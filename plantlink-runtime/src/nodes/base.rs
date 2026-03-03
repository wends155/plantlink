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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::NodeContext;
    use plantlink_core::MessagePayload;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct DummySimpleNode {
        started: Arc<Mutex<bool>>,
        stopped: Arc<Mutex<bool>>,
        fail_on_handle: bool,
    }

    impl DummySimpleNode {
        fn new(fail_on_handle: bool) -> Self {
            Self {
                started: Arc::new(Mutex::new(false)),
                stopped: Arc::new(Mutex::new(false)),
                fail_on_handle,
            }
        }
    }

    #[async_trait::async_trait]
    impl SimpleNode for DummySimpleNode {
        async fn on_start(&mut self, _ctx: &NodeContext) -> Result<()> {
            *self.started.lock().unwrap() = true;
            Ok(())
        }

        async fn handle(
            &mut self,
            _port: usize,
            _msg: MessagePayload,
            _ctx: &NodeContext,
        ) -> Result<()> {
            if self.fail_on_handle {
                anyhow::bail!("handle error");
            }
            Ok(())
        }

        async fn on_stop(&mut self) -> Result<()> {
            *self.stopped.lock().unwrap() = true;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_adapter_delegates_start() {
        let node = DummySimpleNode::new(false);
        let started = Arc::clone(&node.started);
        let mut adapter = BaseNodeAdapter::new(node);
        let (ctx, _) = NodeContext::for_test("base-test");
        adapter.start(ctx).await.unwrap();
        assert!(*started.lock().unwrap(), "Expected on_start to be called");
    }

    #[tokio::test]
    async fn test_adapter_on_input_error_broadcasts_status() {
        let node = DummySimpleNode::new(true);
        let mut adapter = BaseNodeAdapter::new(node);
        let (ctx, mut sys_rx) = NodeContext::for_test("base-err");
        let result = adapter.on_input(0, MessagePayload::default(), ctx).await;
        assert!(result.is_err(), "Expected error from failing handle");
        // Drain and look for "error" status in broadcast
        let mut found = false;
        while let Ok(msg) = sys_rx.try_recv() {
            if msg.contains("error") {
                found = true;
            }
        }
        assert!(found, "Expected 'error' status broadcast from adapter");
    }

    #[tokio::test]
    async fn test_adapter_delegates_stop() {
        let node = DummySimpleNode::new(false);
        let stopped = Arc::clone(&node.stopped);
        let mut adapter = BaseNodeAdapter::new(node);
        adapter.stop().await.unwrap();
        assert!(*stopped.lock().unwrap(), "Expected on_stop to be called");
    }
}
