//! # `PlantLink` Runtime
//!
//! The flow execution engine for `PlantLink`. Manages the lifecycle of all nodes
//! defined in a [`FlowConfig`] and routes messages between them.

// use rhai::{Engine, Scope, AST, Dynamic};
// use plantlink_core::MessagePayload;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

#[async_trait::async_trait]
pub trait FlowRuntime: Send + Sync {
    async fn update_flow(&mut self, flow: FlowConfig) -> Result<()>;
    async fn stop_flow(&mut self) -> StopStatus;
}

mod nodes;
pub use nodes::SystemEvent;
use nodes::{NodeBehavior, NodeContext, NodeReceiver, OutputMap};
use tokio_stream::StreamExt;

/// Configuration for a single node in a flow.
///
/// # Examples
///
/// ```
/// use plantlink_runtime::NodeConfig;
///
/// let json = r#"{"id": "n1", "type": "console", "data": {"label": "My Node"}}"#;
/// let config: NodeConfig = serde_json::from_str(json).unwrap();
/// assert_eq!(config.id, "n1");
/// assert_eq!(config.type_, "console");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: serde_json::Value,
}

/// Defines a connection between two nodes in a flow.
///
/// # Examples
///
/// ```
/// use plantlink_runtime::EdgeConfig;
///
/// let json = r#"{"id": "e1", "source": "n1", "target": "n2"}"#;
/// let edge: EdgeConfig = serde_json::from_str(json).unwrap();
/// assert_eq!(edge.source, "n1");
/// assert!(edge.source_handle.is_none());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConfig {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "sourceHandle", default)]
    pub source_handle: Option<String>,
    #[serde(rename = "targetHandle", default)]
    pub target_handle: Option<String>,
}

/// A complete flow definition containing nodes and edges.
///
/// # Examples
///
/// ```
/// use plantlink_runtime::{FlowConfig, NodeConfig, EdgeConfig};
///
/// let json = r#"{
///     "nodes": [{"id": "n1", "type": "console", "data": {}}],
///     "edges": [{"id": "e1", "source": "n1", "target": "n2"}]
/// }"#;
/// let flow: FlowConfig = serde_json::from_str(json).unwrap();
/// assert_eq!(flow.nodes.len(), 1);
/// assert_eq!(flow.edges.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConfig {
    pub nodes: Vec<NodeConfig>,
    pub edges: Vec<EdgeConfig>,
}

/// Manages the lifecycle of all nodes in a flow.
///
/// # Examples
///
/// ```no_run
/// use plantlink_runtime::RuntimeEngine;
/// use tokio::sync::broadcast;
///
/// # async fn example() -> anyhow::Result<()> {
/// let (tx, _) = broadcast::channel(100);
/// let mut engine = RuntimeEngine::new(tx)?;
/// // engine.update_flow(flow).await?;
/// // engine.stop_flow().await;
/// # Ok(())
/// # }
/// ```
/// Status returned by `stop_flow()` to report shutdown results.
#[derive(Debug, Clone, Serialize)]
pub struct StopStatus {
    /// Number of tasks that were aborted.
    pub tasks_aborted: usize,
}

pub struct RuntimeEngine {
    tx: broadcast::Sender<nodes::SystemEvent>,
    tasks: HashMap<String, tokio::task::JoinHandle<()>>,
    /// Cancellation token for the current flow
    cancel: CancellationToken,
    registry: nodes::registry::NodeRegistry,
}

impl RuntimeEngine {
    ///
    /// # Errors
    /// Returns an error if the engine cannot be initialized.
    pub fn new(tx: broadcast::Sender<nodes::SystemEvent>) -> Result<Self> {
        let mut registry = nodes::registry::NodeRegistry::new();
        // Ensure defaults are registered
        nodes::register_defaults(&mut registry)?;

        Ok(Self {
            tx,
            tasks: HashMap::new(),
            cancel: CancellationToken::new(),
            registry,
        })
    }
}

#[async_trait::async_trait]
impl FlowRuntime for RuntimeEngine {
    #[allow(clippy::unused_async)]
    async fn stop_flow(&mut self) -> StopStatus {
        tracing::info!("Runtime: Stopping active flow");

        // 1. Signal cancellation to all nodes
        self.cancel.cancel();

        // Emit stopped status for all nodes immediately for better UI feedback
        for node_id in self.tasks.keys() {
            nodes::send_node_status(&self.tx, node_id.clone(), "stopped", "Flow stopped");
        }

        // 2. Abort all tasks
        let tasks_to_abort = self.tasks.len();
        for (_, handle) in self.tasks.drain() {
            handle.abort();
        }

        // 3. Create a fresh token for the next flow
        self.cancel = CancellationToken::new();

        StopStatus {
            tasks_aborted: tasks_to_abort,
        }
    }

    ///
    /// # Errors
    /// Returns an error if the new flow cannot be deployed.
    #[allow(clippy::if_not_else)]
    async fn update_flow(&mut self, flow: FlowConfig) -> Result<()> {
        tracing::info!("Runtime: Updating flow with {} nodes", flow.nodes.len());

        // 1. Stop all existing tasks
        self.stop_flow().await;

        // 2. Map Edges to Outputs and Inputs
        let wiring = build_wiring(&flow.edges);

        // 3. Create Channels and Contexts
        let (mut node_senders, mut node_receivers) = create_channels(wiring);

        // Use a Shared Resource Registry for this flow execution
        let resources =
            std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

        // 4. Instantiate and Start Nodes
        let mut failed_nodes: Vec<String> = Vec::new();

        for config in flow.nodes {
            let node_id = config.id.clone();
            let outputs = node_senders.remove(&node_id).unwrap_or_default();
            let ctx = NodeContext::new(
                node_id.clone(),
                outputs,
                resources.clone(),
                self.tx.clone(),
                self.cancel.child_token(),
            );

            // Create specific node instance dynamically from registry
            let mut node: Box<dyn NodeBehavior> = match self.registry.create(&config.type_, &config)
            {
                Ok(n) => n,
                Err(e) => {
                    tracing::warn!("Failed to create node {}: {}", config.type_, e);
                    failed_nodes.push(config.id.clone());
                    continue;
                }
            };

            // Get receivers for this node
            let inputs = node_receivers.remove(&node_id).unwrap_or_default();

            // Spawn Actor Task
            let task = tokio::spawn(async move {
                // Initialize Node
                if let Err(e) = node.start(ctx.clone()).await {
                    tracing::error!("Node {} failed to start: {}", node_id, e);
                    ctx.emit_error(&format!("Failed to start: {e}"));
                }

                // Listen Loop (if we have inputs)
                if !inputs.is_empty() {
                    // Combine all receivers into a single stream
                    let mut streams = tokio_stream::StreamMap::new();
                    for (port_idx, rx) in inputs {
                        let stream =
                            tokio_stream::wrappers::ReceiverStream::new(rx).map(|(_, msg)| msg);
                        streams.insert(port_idx, stream);
                    }

                    while let Some((port_idx, msg)) = streams.next().await {
                        if let Err(e) = node.receive(port_idx, msg, &ctx).await {
                            tracing::error!("Node {} error on input: {}", node_id, e);
                        }
                    }
                } else {
                    // Keep alive
                    futures::future::pending::<()>().await;
                }

                // Cleanup
                if let Err(e) = node.stop().await {
                    tracing::warn!("Node {} error on stop: {}", node_id, e);
                }
                ctx.emit_stopped("Node stopped");
            });

            self.tasks.insert(config.id, task);
        }

        if !failed_nodes.is_empty() {
            bail!(
                "Failed to create {} node(s): {}",
                failed_nodes.len(),
                failed_nodes.join(", ")
            );
        }

        Ok(())
    }
}

#[allow(clippy::type_complexity)]
fn build_wiring(edges: &[EdgeConfig]) -> HashMap<String, HashMap<usize, Vec<(String, usize)>>> {
    let mut wiring: HashMap<String, HashMap<usize, Vec<(String, usize)>>> = HashMap::new();

    for edge in edges {
        let src_port = parse_port(edge.source_handle.as_deref());
        let tgt_port = parse_port(edge.target_handle.as_deref());

        wiring
            .entry(edge.source.clone())
            .or_default()
            .entry(src_port)
            .or_default()
            .push((edge.target.clone(), tgt_port));
    }

    wiring
}

#[allow(clippy::type_complexity)]
fn create_channels(
    wiring: HashMap<String, HashMap<usize, Vec<(String, usize)>>>,
) -> (
    HashMap<String, OutputMap>,
    HashMap<String, Vec<(usize, NodeReceiver)>>,
) {
    let mut node_senders: HashMap<String, OutputMap> = HashMap::new();
    let mut node_receivers: HashMap<String, Vec<(usize, NodeReceiver)>> = HashMap::new();

    for (source_id, ports) in wiring {
        for (src_port, targets) in ports {
            for (target_id, tgt_port) in targets {
                let (tx, rx) = tokio::sync::mpsc::channel(100);

                node_senders
                    .entry(source_id.clone())
                    .or_default()
                    .entry(src_port)
                    .or_default()
                    .push((tx, tgt_port));

                node_receivers
                    .entry(target_id.clone())
                    .or_default()
                    .push((tgt_port, rx));
            }
        }
    }

    (node_senders, node_receivers)
}

fn parse_port(_handle: Option<&str>) -> usize {
    // Simple heuristic: try to find a digit in the handle string "payload", "output_1" -> 1
    // For now, return 0. (Phase 1 simplicity)
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) struct MockRuntime {
        pub deployed: bool,
        pub stopped: bool,
    }

    #[async_trait::async_trait]
    impl FlowRuntime for MockRuntime {
        async fn update_flow(&mut self, _flow: FlowConfig) -> Result<()> {
            self.deployed = true;
            Ok(())
        }
        async fn stop_flow(&mut self) -> StopStatus {
            self.stopped = true;
            StopStatus { tasks_aborted: 0 }
        }
    }

    #[tokio::test]
    async fn test_mock_runtime_implements_flow_runtime() {
        let mut rt: Box<dyn FlowRuntime> = Box::new(MockRuntime {
            deployed: false,
            stopped: false,
        });
        let flow = FlowConfig {
            nodes: vec![],
            edges: vec![],
        };
        rt.update_flow(flow).await.unwrap();
        rt.stop_flow().await;
        // The mock records state successfully.
    }

    #[test]
    fn test_status_serialization() {
        let (tx, mut rx) = broadcast::channel(16);
        nodes::send_node_status(&tx, "test-node".to_string(), "running", "Active");
        let msg = rx.try_recv().expect("Message not received");
        if let nodes::SystemEvent::Status { data } = msg {
            assert_eq!(data.node_id, "test-node");
            assert_eq!(data.state, "running");
        } else {
            panic!("Expected SystemEvent::Status, got {msg:?}");
        }
    }

    #[test]
    fn test_flow_config_deserialization() {
        let json = r#"{
            "nodes": [{"id": "n1", "type": "console", "data": {}}],
            "edges": [{"id": "e1", "source": "n1", "target": "n2"}]
        }"#;
        let config: FlowConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.nodes.len(), 1);
        assert_eq!(config.nodes[0].type_, "console");
        assert_eq!(config.edges.len(), 1);
    }

    #[test]
    fn test_edge_config_optional_handles() {
        let json = r#"{"id": "e1", "source": "n1", "target": "n2"}"#;
        let edge: EdgeConfig = serde_json::from_str(json).unwrap();
        assert!(edge.source_handle.is_none());
        assert!(edge.target_handle.is_none());
    }

    #[tokio::test]
    async fn test_stop_flow_emits_stopped() {
        let (tx, mut rx) = broadcast::channel(16);
        let mut engine = RuntimeEngine::new(tx).unwrap();
        // Deploy a minimal flow with a single console node
        let flow = FlowConfig {
            nodes: vec![NodeConfig {
                id: "n1".into(),
                type_: "console".into(),
                data: serde_json::json!({}),
            }],
            edges: vec![],
        };
        engine.update_flow(flow).await.unwrap();
        // Drain initial status broadcasts
        while rx.try_recv().is_ok() {}
        engine.stop_flow().await;
        // Should receive at least one "stopped" status
        let msg = rx.try_recv().expect("Expected stopped status");
        if let nodes::SystemEvent::Status { data } = msg {
            assert_eq!(data.state, "stopped");
        } else {
            panic!("Expected SystemEvent::Status, got {msg:?}");
        }
    }

    #[test]
    fn test_new_returns_ok() {
        let (tx, _rx) = broadcast::channel(16);
        let engine = RuntimeEngine::new(tx);
        assert!(engine.is_ok(), "RuntimeEngine::new should succeed");
    }

    #[tokio::test]
    async fn test_update_flow_error_on_invalid_node_type() {
        let (tx, _rx) = broadcast::channel(16);
        let mut engine = RuntimeEngine::new(tx).unwrap();
        let flow = FlowConfig {
            nodes: vec![NodeConfig {
                id: "bad-node".into(),
                type_: "nonexistent-node-type".into(),
                data: serde_json::json!({}),
            }],
            edges: vec![],
        };
        let result = engine.update_flow(flow).await;
        assert!(
            result.is_err(),
            "update_flow should fail for unknown node types"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("bad-node"),
            "Error should name the failing node, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_stop_flow_returns_correct_count() {
        let (tx, _rx) = broadcast::channel(16);
        let mut engine = RuntimeEngine::new(tx).unwrap();

        // Deploy a flow with 2 valid nodes
        let flow = FlowConfig {
            nodes: vec![
                NodeConfig {
                    id: "n1".into(),
                    type_: "console".into(),
                    data: serde_json::json!({}),
                },
                NodeConfig {
                    id: "n2".into(),
                    type_: "console".into(),
                    data: serde_json::json!({}),
                },
            ],
            edges: vec![],
        };
        engine.update_flow(flow).await.unwrap();
        let status = engine.stop_flow().await;
        assert_eq!(status.tasks_aborted, 2, "Should report 2 aborted tasks");
    }

    // ─── Steps 12-17: Wiring & engine lifecycle tests ───────────────────────────

    #[test]
    fn test_build_wiring_single_edge() {
        let edges = vec![EdgeConfig {
            id: "e1".into(),
            source: "n1".into(),
            target: "n2".into(),
            source_handle: None,
            target_handle: None,
        }];
        let wiring = build_wiring(&edges);
        assert!(wiring.contains_key("n1"), "Expected n1 in wiring");
        let n1_ports = &wiring["n1"];
        let targets = n1_ports.get(&0).expect("Expected port 0");
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].0, "n2");
    }

    #[test]
    fn test_build_wiring_multi_edge() {
        let edges = vec![
            EdgeConfig {
                id: "e1".into(),
                source: "n1".into(),
                target: "n2".into(),
                source_handle: None,
                target_handle: None,
            },
            EdgeConfig {
                id: "e2".into(),
                source: "n1".into(),
                target: "n3".into(),
                source_handle: None,
                target_handle: None,
            },
        ];
        let wiring = build_wiring(&edges);
        let targets = &wiring["n1"][&0];
        let target_ids: Vec<_> = targets.iter().map(|(id, _)| id.as_str()).collect();
        assert!(target_ids.contains(&"n2"), "Expected n2 in targets");
        assert!(target_ids.contains(&"n3"), "Expected n3 in targets");
    }

    #[test]
    fn test_build_wiring_empty() {
        let wiring = build_wiring(&[]);
        assert!(wiring.is_empty(), "Expected empty wiring from empty edges");
    }

    #[test]
    fn test_create_channels_produces_paired_senders_receivers() {
        let edges = vec![EdgeConfig {
            id: "e1".into(),
            source: "n1".into(),
            target: "n2".into(),
            source_handle: None,
            target_handle: None,
        }];
        let wiring = build_wiring(&edges);
        let (senders, receivers) = create_channels(wiring);
        assert!(senders.contains_key("n1"), "Expected sender for n1");
        assert!(receivers.contains_key("n2"), "Expected receiver for n2");
        assert_eq!(receivers["n2"].len(), 1, "Expected 1 receiver entry");
    }

    #[tokio::test]
    async fn test_update_flow_replaces_previous() {
        let (tx, _rx) = broadcast::channel(32);
        let mut engine = RuntimeEngine::new(tx).unwrap();

        // Deploy flow A: 2 nodes
        let flow_a = FlowConfig {
            nodes: vec![
                NodeConfig {
                    id: "a1".into(),
                    type_: "console".into(),
                    data: serde_json::json!({}),
                },
                NodeConfig {
                    id: "a2".into(),
                    type_: "console".into(),
                    data: serde_json::json!({}),
                },
            ],
            edges: vec![],
        };
        engine.update_flow(flow_a).await.unwrap();

        // Replace with flow B: 1 node
        let flow_b = FlowConfig {
            nodes: vec![NodeConfig {
                id: "b1".into(),
                type_: "console".into(),
                data: serde_json::json!({}),
            }],
            edges: vec![],
        };
        engine.update_flow(flow_b).await.unwrap();

        // Only flow B's task should be alive
        let status = engine.stop_flow().await;
        assert_eq!(
            status.tasks_aborted, 1,
            "Expected only 1 task from flow B, not flow A"
        );
    }

    #[tokio::test]
    async fn test_mock_runtime_state_tracking() {
        let mut rt = MockRuntime {
            deployed: false,
            stopped: false,
        };
        assert!(!rt.deployed, "Should start undeployed");
        assert!(!rt.stopped, "Should start unstopped");

        rt.update_flow(FlowConfig {
            nodes: vec![],
            edges: vec![],
        })
        .await
        .unwrap();
        assert!(rt.deployed, "Should be deployed after update_flow");

        rt.stop_flow().await;
        assert!(rt.stopped, "Should be stopped after stop_flow");
    }
}
