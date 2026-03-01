use super::{NodeBehavior, NodeContext};
use anyhow::Result;
use plantlink_core::MessagePayload;

/// A simplified trait for nodes that don't need full control over their lifecycle
/// and just want to process messages.
#[async_trait::async_trait]
pub trait SimpleNode: Send + Sync {
    /// Called when the node is started. Can return initial state or error.
    async fn on_start(&mut self, _ctx: &NodeContext) -> Result<()> {
        Ok(())
    }

    /// Handle an incoming message. Return a Result.
    /// If you need to send output, use `ctx.send_output()`.
    async fn handle(&mut self, port: usize, msg: MessagePayload, ctx: &NodeContext) -> Result<()>;

    async fn on_stop(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A wrapper that adapts a `SimpleNode` into a full `NodeBehavior`
#[derive(Clone)]
pub struct BaseNodeAdapter<T: SimpleNode> {
    inner: T,
}

impl<T: SimpleNode> BaseNodeAdapter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl<T: SimpleNode + 'static> NodeBehavior for BaseNodeAdapter<T> {
    async fn start(&mut self, ctx: NodeContext) -> Result<()> {
        // Here we could add automatic status reporting "Running"
        self.inner.on_start(&ctx).await
    }

    async fn on_input(&mut self, port: usize, msg: MessagePayload, ctx: NodeContext) -> Result<()> {
        // We could wrap this in a catch_unwind or specific error handling logging
        if let Err(e) = self.inner.handle(port, msg, &ctx).await {
            // Automatic Error Reporting via System Channel
            let status = crate::nodes::NodeStatus {
                node_id: ctx.id.clone(),
                state: "error".to_string(),
                message: e.to_string(),
            };
            #[allow(clippy::collapsible_if)]
            if let Ok(json) = serde_json::to_string(&serde_json::json!({
                "type": "status",
                "data": status
            })) {
                if let Err(e) = ctx.system_tx.send(json) {
                    tracing::warn!(node_id = %ctx.id, "Failed to broadcast node status: {}", e);
                }
            }
            // Also log
            let log_msg = format!("Node [{}]: Error: {}", ctx.id, e);
            let json_log = serde_json::json!({ "type": "log", "message": log_msg }).to_string();
            if let Err(e) = ctx.system_tx.send(json_log) {
                tracing::warn!(node_id = %ctx.id, "Failed to broadcast log: {}", e);
            }

            return Err(e);
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.inner.on_stop().await
    }
}
