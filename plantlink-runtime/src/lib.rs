// use rhai::{Engine, Scope, AST, Dynamic};
// use plantlink_core::MessagePayload;
// use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;

mod nodes;
use nodes::{NodeBehavior, NodeContext, NodeReceiver, OutputMap};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: serde_json::Value,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConfig {
    pub nodes: Vec<NodeConfig>,
    pub edges: Vec<EdgeConfig>,
}

pub struct RuntimeEngine {
    tx: broadcast::Sender<String>,
    tasks: HashMap<String, tokio::task::JoinHandle<()>>,
}

impl RuntimeEngine {
    pub fn new(tx: broadcast::Sender<String>) -> Self {
        // Ensure defaults are registered
        nodes::register_defaults();

        Self {
            tx,
            tasks: HashMap::new(),
        }
    }

    pub async fn stop_flow(&mut self) {
        tracing::info!(
            "Runtime: Stopping flow. Aborting {} tasks.",
            self.tasks.len()
        );

        // Emit stopped status for all nodes
        for node_id in self.tasks.keys() {
            nodes::send_node_status(&self.tx, node_id.clone(), "stopped", "Flow stopped");
        }

        // Then abort tasks
        for (_, handle) in self.tasks.drain() {
            handle.abort();
        }
    }

    pub async fn update_flow(&mut self, flow: FlowConfig) {
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
        for config in flow.nodes {
            let node_id = config.id.clone();
            let outputs = node_senders.remove(&node_id).unwrap_or_default();
            let ctx =
                NodeContext::new(node_id.clone(), outputs, resources.clone(), self.tx.clone());

            // Create specific node instance dynamically from registry
            let mut node: Box<dyn NodeBehavior> =
                match nodes::registry::create_node(&config.type_, &config) {
                    Ok(n) => n,
                    Err(e) => {
                        tracing::warn!("Failed to create node {}: {}", config.type_, e);
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
}
