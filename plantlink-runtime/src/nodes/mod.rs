pub mod inject;
pub mod console;
pub mod nats;
pub mod rhai;
pub mod registry;
pub mod base;

use async_trait::async_trait;
use plantlink_core::MessagePayload;
use anyhow::Result;
use tokio::sync::mpsc;
use std::collections::HashMap;

use std::sync::Arc;
use tokio::sync::RwLock;
use std::any::Any;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub state: String, // "running", "error", "stopped"
    pub message: String,
}

pub fn register_defaults() {
    registry::register_node("inject", |cfg| Box::new(base::BaseNodeAdapter::new(inject::InjectNode::new(cfg))));
    registry::register_node("console", |cfg| Box::new(console::ConsoleNode::new(cfg)));
    registry::register_node("nats-broker", |cfg| Box::new(nats::NatsBrokerNode::new(cfg)));
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
    outputs: HashMap<usize, Vec<(mpsc::Sender<(usize, MessagePayload)>, usize)>>,
    /// Shared Resource Registry (Connection Objects, etc.)
    pub resources: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    /// System Broadcast Channel (For Logs and Status)
    pub system_tx: broadcast::Sender<String>,
}

impl NodeContext {
    pub fn new(
        id: String, 
        outputs: HashMap<usize, Vec<(mpsc::Sender<(usize, MessagePayload)>, usize)>>,
        resources: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
        system_tx: broadcast::Sender<String>,
    ) -> Self {
        Self { id, outputs, resources, system_tx }
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
}

/// The behavior every node must implement
#[async_trait]
pub trait NodeBehavior: Send + Sync {
    async fn start(&mut self, _ctx: NodeContext) -> Result<()> {
        Ok(())
    }

    async fn on_input(&mut self, _port: usize, _msg: MessagePayload, _ctx: NodeContext) -> Result<()> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}
