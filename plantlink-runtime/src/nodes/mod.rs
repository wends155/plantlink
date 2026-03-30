//! Node Framework and Registry
//!
//! This module defines the core trait `NodeBehavior` that all node types
//! must implement. It also provides the global `NodeRegistry` for dynamic
//! node creation from configuration strings.

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
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub type NodeSender = mpsc::Sender<(usize, Arc<MessagePayload>)>;
pub type NodeReceiver = mpsc::Receiver<(usize, Arc<MessagePayload>)>;
pub type PortLinks = Vec<(NodeSender, usize)>;
pub type OutputMap = HashMap<usize, PortLinks>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub state: String, // "running", "error", "stopped"
    pub message: String,
}

/// Events broadcast across the system-wide event bus
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SystemEvent {
    Status { data: NodeStatus },
    Log { message: String },
}

pub fn send_node_status(
    tx: &broadcast::Sender<SystemEvent>,
    node_id: String,
    state: &str,
    message: &str,
) {
    let status = NodeStatus {
        node_id,
        state: state.to_string(),
        message: message.to_string(),
    };
    let event = SystemEvent::Status { data: status };
    if let Err(e) = tx.send(event) {
        tracing::warn!("Failed to broadcast node status: {}", e);
    }
}

pub fn register_defaults(registry: &mut registry::NodeRegistry) -> anyhow::Result<()> {
    registry.register("inject", |cfg| {
        Ok(
            Box::new(base::BaseNodeAdapter::new(inject::InjectNode::new(cfg)))
                as Box<dyn NodeBehavior>,
        )
    })?;
    registry.register("console", |cfg| {
        Ok(Box::new(console::ConsoleNode::new(cfg)) as Box<dyn NodeBehavior>)
    })?;
    registry.register("nats-broker", |cfg| {
        Ok(Box::new(nats::NatsBrokerNode::new(cfg)) as Box<dyn NodeBehavior>)
    })?;
    registry.register("nats-sub", |cfg| {
        Ok(Box::new(nats::NatsSubNode::new(cfg)) as Box<dyn NodeBehavior>)
    })?;
    registry.register("nats-pub", |cfg| {
        Ok(Box::new(nats::NatsPubNode::new(cfg)) as Box<dyn NodeBehavior>)
    })?;
    registry.register("rhai", |cfg| {
        rhai::RhaiNode::new(cfg).map(|n| Box::new(n) as Box<dyn NodeBehavior>)
    })?;
    registry.register("function", |cfg| {
        rhai::RhaiNode::new(cfg).map(|n| Box::new(n) as Box<dyn NodeBehavior>)
    })?;
    registry.register("rhai-function", |cfg| {
        rhai::RhaiNode::new(cfg).map(|n| Box::new(n) as Box<dyn NodeBehavior>)
    })?;
    Ok(())
}

/// Capabilities and routing provided to a node instance.
///
/// Each node receives a `NodeContext` to interact with the broader flow:
/// 1. **Output Routing**: Multi-casting messages to downstreams.
/// 2. **Shared Resources**: Accessing shared driver instances.
/// 3. **Observability**: Sending status updates and logs.
/// 4. **Cancellation**: Responding to flow termination via `cancel`.
#[derive(Clone)]
pub struct NodeContext {
    pub id: String,
    /// Map of Output Port Index -> List of (Channel Sender, Target Input Port Index)
    outputs: OutputMap,
    /// Shared Resource Registry (Connection Objects, etc.)
    pub resources: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    /// System Broadcast Channel (For Logs and Status)
    pub system_tx: broadcast::Sender<SystemEvent>,
    /// Cancellation Token for cooperative shutdown
    pub cancel: CancellationToken,
    /// Task Tracker for structured concurrency
    pub tracker: TaskTracker,
}

impl NodeContext {
    pub fn new(
        id: String,
        outputs: OutputMap,
        resources: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
        system_tx: broadcast::Sender<SystemEvent>,
        cancel: CancellationToken,
        tracker: TaskTracker,
    ) -> Self {
        Self {
            id,
            outputs,
            resources,
            system_tx,
            cancel,
            tracker,
        }
    }

    /// Send a message to the default output port (0)
    pub async fn send_output(&self, msg: MessagePayload) -> Result<()> {
        self.send_output_port(0, msg).await
    }

    /// Send a message to a specific output port
    pub async fn send_output_port(&self, port: usize, msg: MessagePayload) -> Result<()> {
        if let Some(links) = self.outputs.get(&port) {
            let arc_msg = Arc::new(msg);
            let mut failures = 0;
            for (sender, target_input_port) in links {
                // Send (TargetPort, Payload) to the channel
                if let Err(e) = sender.send((*target_input_port, arc_msg.clone())).await {
                    tracing::warn!(
                        node_id = %self.id,
                        port,
                        "Failed to send to downstream node (channel closed): {}",
                        e
                    );
                    failures += 1;
                }
            }
            if failures > 0 {
                anyhow::bail!(
                    "Node {}: {}/{} downstream sends failed on port {}",
                    self.id,
                    failures,
                    links.len(),
                    port
                );
            }
        }
        Ok(())
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

    /// Emit a log message to the system event bus
    pub fn emit_log(&self, message: impl Into<String>) {
        let event = SystemEvent::Log {
            message: message.into(),
        };
        let _ = self.system_tx.send(event);
    }

    /// Convenience constructor for unit tests with minimal boilerplate.
    #[cfg(test)]
    pub fn for_test(id: &str) -> (Self, broadcast::Receiver<SystemEvent>) {
        let (tx, rx) = broadcast::channel(16);
        let ctx = Self::new(
            id.to_string(),
            HashMap::new(),
            Arc::new(RwLock::new(HashMap::new())),
            tx,
            CancellationToken::new(),
            TaskTracker::new(),
        );
        (ctx, rx)
    }
}

/// The behavior trait every node must implement.
///
/// Nodes are concurrent units of logic that process messages. They
/// are managed by the `RuntimeEngine` and communicate via `NodeContext`.
#[async_trait]
pub trait NodeBehavior: Send + Sync {
    /// Initialize the node and start internal background tasks.
    async fn start(&mut self, _ctx: NodeContext) -> Result<()> {
        Ok(())
    }

    /// Primary data handler for nodes.
    ///
    /// This is the preferred way to receive data. It uses `Arc<MessagePayload>`
    /// to avoid deep-cloning payloads when fan-out is high.
    ///
    /// # Arguments
    /// - `port`: Input port index.
    /// - `msg`: `Arc`-wrapped `MessagePayload`.
    /// - `ctx`: The node's runtime context.
    async fn receive(
        &mut self,
        port: usize,
        msg: Arc<MessagePayload>,
        ctx: &NodeContext,
    ) -> Result<()> {
        // Default implementation shims to deprecated on_input for backward compatibility
        #[allow(deprecated)]
        // NOTE: This shim still clones but receive callers do not.
        self.on_input(port, (*msg).clone(), ctx.clone()).await
    }

    /// [DEPRECATED] Use `receive` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use `receive` instead to avoid cloning overhead"
    )]
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
    use super::{NodeContext, OutputMap, SystemEvent};
    use plantlink_core::MessagePayload;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::sync::broadcast;
    use tokio_util::sync::CancellationToken;

    #[test]
    fn test_node_context_emit_stopped() {
        let (ctx, mut rx) = NodeContext::for_test("test-node");
        ctx.emit_stopped("Manual stop");
        let msg = rx.try_recv().expect("Message not received");
        if let SystemEvent::Status { data } = msg {
            assert_eq!(data.node_id, "test-node");
            assert_eq!(data.state, "stopped");
            assert_eq!(data.message, "Manual stop");
        } else {
            panic!("Expected SystemEvent::Status, got {msg:?}");
        }
    }

    #[tokio::test]
    async fn test_send_output_delivers_to_downstream() {
        use tokio::sync::mpsc;
        let (tx, mut rx) = mpsc::channel(16);
        let (sys_tx, _) = broadcast::channel(16);
        let mut outputs: OutputMap = HashMap::new();
        outputs.insert(0, vec![(tx, 0)]);

        let ctx = NodeContext::new(
            "test-node".to_string(),
            outputs,
            Arc::new(RwLock::new(HashMap::new())),
            sys_tx,
            CancellationToken::new(),
            tokio_util::task::TaskTracker::new(),
        );

        let msg = MessagePayload::default();
        let msg_id = msg.id;

        ctx.send_output(msg).await.unwrap();

        let (port, received) = rx.recv().await.unwrap();
        assert_eq!(port, 0);
        assert_eq!(received.id, msg_id);
    }

    #[tokio::test]
    async fn test_send_output_no_links_is_ok() {
        let (ctx, _) = NodeContext::for_test("test-node");
        // No outputs configured — should succeed silently
        let result = ctx.send_output(MessagePayload::default()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_emit_running_broadcasts() {
        let (ctx, mut rx) = NodeContext::for_test("run-node");
        ctx.emit_running("All good");

        let msg = rx.try_recv().unwrap();
        if let SystemEvent::Status { data } = msg {
            assert_eq!(data.state, "running");
            assert_eq!(data.message, "All good");
        } else {
            panic!("Expected SystemEvent::Status, got {msg:?}");
        }
    }

    #[test]
    fn test_emit_error_broadcasts() {
        let (ctx, mut rx) = NodeContext::for_test("err-node");
        ctx.emit_error("Something failed");

        let msg = rx.try_recv().unwrap();
        if let SystemEvent::Status { data } = msg {
            assert_eq!(data.state, "error");
            assert_eq!(data.message, "Something failed");
        } else {
            panic!("Expected SystemEvent::Status, got {msg:?}");
        }
    }

    #[test]
    fn test_emit_log_broadcasts() {
        let (ctx, mut rx) = NodeContext::for_test("log-node");
        ctx.emit_log("Diagnostic info");

        let msg = rx.try_recv().unwrap();
        if let SystemEvent::Log { message } = msg {
            assert_eq!(message, "Diagnostic info");
        } else {
            panic!("Expected SystemEvent::Log, got {msg:?}");
        }
    }
}
