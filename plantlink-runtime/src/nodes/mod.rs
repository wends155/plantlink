pub mod base;
pub mod console;
pub mod inject;
pub mod nats;
pub mod registry;
pub mod rhai;

use anyhow::Result;
use async_trait::async_trait;
use plantlink_core::MessagePayload;
use std::collections::HashMap;
use tokio::sync::mpsc;

use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

pub type NodeSender = mpsc::Sender<(usize, MessagePayload)>;
pub type NodeReceiver = mpsc::Receiver<(usize, MessagePayload)>;
pub type PortLinks = Vec<(NodeSender, usize)>;
pub type OutputMap = HashMap<usize, PortLinks>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub state: String, // "running", "error", "stopped"
    pub message: String,
}

pub fn send_node_status(
    tx: &broadcast::Sender<String>,
    node_id: String,
    state: &str,
    message: &str,
) {
    let status = NodeStatus {
        node_id,
        state: state.to_string(),
        message: message.to_string(),
    };
    if let Ok(json) = serde_json::to_string(&serde_json::json!({
        "type": "status",
        "data": status
    })) {
        let _ = tx.send(json);
    }
}

pub fn register_defaults() {
    registry::register_node("inject", |cfg| {
        Box::new(base::BaseNodeAdapter::new(inject::InjectNode::new(cfg)))
    });
    registry::register_node("console", |cfg| Box::new(console::ConsoleNode::new(cfg)));
    registry::register_node("nats-broker", |cfg| {
        Box::new(nats::NatsBrokerNode::new(cfg))
    });
    registry::register_node("nats-sub", |cfg| Box::new(nats::NatsSubNode::new(cfg)));
    registry::register_node("nats-pub", |cfg| Box::new(nats::NatsPubNode::new(cfg)));
    registry::register_node("rhai", |cfg| Box::new(rhai::RhaiNode::new(cfg)));
    registry::register_node("function", |cfg| Box::new(rhai::RhaiNode::new(cfg)));
    registry::register_node("rhai-function", |cfg| Box::new(rhai::RhaiNode::new(cfg)));
}

/// Context passed to the node during execution
#[derive(Clone)]
pub struct NodeContext {
    pub id: String,
    /// Map of Output Port Index -> List of (Channel Sender, Target Input Port Index)
    outputs: OutputMap,
    /// Shared Resource Registry (Connection Objects, etc.)
    pub resources: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    /// System Broadcast Channel (For Logs and Status)
    pub system_tx: broadcast::Sender<String>,
}

impl NodeContext {
    pub fn new(
        id: String,
        outputs: OutputMap,
        resources: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
        system_tx: broadcast::Sender<String>,
    ) -> Self {
        Self {
            id,
            outputs,
            resources,
            system_tx,
        }
    }

    /// Send a message to the default output port (0)
    pub async fn send_output(&self, msg: MessagePayload) {
        self.send_output_port(0, msg).await;
    }

    /// Send a message to a specific output port
    pub async fn send_output_port(&self, port: usize, msg: MessagePayload) {
        if let Some(links) = self.outputs.get(&port) {
            for (sender, target_input_port) in links {
                // Send (TargetPort, Payload) to the channel
                let _ = sender.send((*target_input_port, msg.clone())).await;
            }
        }
    }

    /// Emit a "running" status message
    pub fn emit_running(&self, message: &str) {
        self.emit_status("running", message);
    }

    /// Emit an "error" status message
    pub fn emit_error(&self, message: &str) {
        self.emit_status("error", message);
    }

    /// Emit a "stopped" status message
    pub fn emit_stopped(&self, message: &str) {
        self.emit_status("stopped", message);
    }

    fn emit_status(&self, state: &str, message: &str) {
        send_node_status(&self.system_tx, self.id.clone(), state, message);
    }
}

/// The behavior every node must implement
#[async_trait]
pub trait NodeBehavior: Send + Sync {
    async fn start(&mut self, _ctx: NodeContext) -> Result<()> {
        Ok(())
    }

    async fn on_input(
        &mut self,
        _port: usize,
        _msg: MessagePayload,
        _ctx: NodeContext,
    ) -> Result<()> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[test]
    fn test_node_context_emit_stopped() {
        let (tx, mut rx) = broadcast::channel(16);
        let ctx = NodeContext::new(
            "test-node".to_string(),
            HashMap::new(),
            Arc::new(RwLock::new(HashMap::new())),
            tx,
        );
        ctx.emit_stopped("Manual stop");
        let msg = rx.try_recv().expect("Message not received");
        assert!(msg.contains("test-node"));
        assert!(msg.contains("stopped"));
        assert!(msg.contains("Manual stop"));
    }
}
