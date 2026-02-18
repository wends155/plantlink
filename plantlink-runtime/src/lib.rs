//! # PlantLink Runtime
//!
//! The flow execution engine for PlantLink. Manages the lifecycle of all nodes
//! defined in a [`FlowConfig`] and routes messages between them.

// use rhai::{Engine, Scope, AST, Dynamic};
// use plantlink_core::MessagePayload;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

mod nodes;
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
    tx: broadcast::Sender<String>,
    tasks: HashMap<String, tokio::task::JoinHandle<()>>,
    /// Cancellation token for the current flow
    cancel: CancellationToken,
}

impl RuntimeEngine {
    pub fn new(tx: broadcast::Sender<String>) -> Result<Self> {
        // Ensure defaults are registered
        nodes::register_defaults()?;

        Ok(Self {
            tx,
            tasks: HashMap::new(),
            cancel: CancellationToken::new(),
        })
    }

    pub async fn stop_flow(&mut self) -> StopStatus {
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

    pub async fn update_flow(&mut self, flow: FlowConfig) -> Result<()> {
        tracing::info!("Runtime: Updating flow with {} nodes", flow.nodes.len());

        // 1. Stop all existing tasks
        self.stop_flow().await;

        // 2. Map Edges to Outputs and Inputs
        // NodeID -> OutputPortIndex -> Vec<(TargetNodeID, TargetInputPortIndex)>
        let mut wiring: HashMap<String, HashMap<usize, Vec<(String, usize)>>> = HashMap::new();

        for edge in &flow.edges {
            // Parse Handle IDs to Port Indexes (For now default to 0 if not parsable or missing)
            // Convention: "port_0", "output_1", etc. or just index.
            let src_port = parse_port(edge.source_handle.as_deref());
            let tgt_port = parse_port(edge.target_handle.as_deref());

            wiring
                .entry(edge.source.clone())
                .or_default()
                .entry(src_port)
                .or_default()
                .push((edge.target.clone(), tgt_port));
        }

        // 3. Create Channels and Contexts
        let mut node_senders: HashMap<String, OutputMap> = HashMap::new();
        let mut node_receivers: HashMap<String, Vec<(usize, NodeReceiver)>> = HashMap::new();

        // Use a Shared Resource Registry for this flow execution
        let resources =
            std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

        for (source_id, ports) in wiring {
            for (src_port, targets) in ports {
                for (target_id, tgt_port) in targets {
                    let (tx, rx) = mpsc::channel(100);

                    // Store Sender for Source Node
                    node_senders
                        .entry(source_id.clone())
                        .or_default()
                        .entry(src_port)
                        .or_default()
                        .push((tx, tgt_port));

                    // Store Receiver for Target Node
                    node_receivers
                        .entry(target_id.clone())
                        .or_default()
                        .push((tgt_port, rx));
                }
            }
        }

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
            let mut node: Box<dyn NodeBehavior> =
                match nodes::registry::create_node(&config.type_, &config) {
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

                    // Emit Error Status
                    let status = nodes::NodeStatus {
                        node_id: node_id.clone(),
                        state: "error".to_string(),
                        message: format!("Failed to start: {}", e),
                    };
                    #[allow(clippy::collapsible_if)]
                    if let Ok(json) = serde_json::to_string(&serde_json::json!({
                        "type": "status",
                        "data": status
                    })) {
                        if let Err(e) = ctx.system_tx.send(json) {
                            tracing::warn!(node_id = %node_id, "Failed to broadcast error status: {}", e);
                        }
                    }
                }

                // Listen Loop (if we have inputs)
                if !inputs.is_empty() {
                    // Combine all receivers into a single stream
                    // We map each receiver stream to extract just the Msg because StreamMap provides the Port Index (Key)
                    let mut streams = tokio_stream::StreamMap::new();
                    for (port_idx, rx) in inputs {
                        let stream =
                            tokio_stream::wrappers::ReceiverStream::new(rx).map(|(_, msg)| msg);
                        streams.insert(port_idx, stream);
                    }

                    while let Some((port_idx, msg)) = streams.next().await {
                        if let Err(e) = node.on_input(port_idx, msg, ctx.clone()).await {
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

fn parse_port(_handle: Option<&str>) -> usize {
    // Simple heuristic: try to find a digit in the handle string "payload", "output_1" -> 1
    // For now, return 0. (Phase 1 simplicity)
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_status_serialization() {
        let (tx, mut rx) = broadcast::channel(16);
        nodes::send_node_status(&tx, "test-node".to_string(), "running", "Active");
        let msg = rx.try_recv().expect("Message not received");
        assert!(msg.contains("test-node"));
        assert!(msg.contains("running"));
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
        assert!(msg.contains("stopped"));
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
            "Error should name the failing node, got: {}",
            err_msg
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
}
