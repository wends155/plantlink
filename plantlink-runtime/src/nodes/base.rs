//! Base Node Framework
//!
//! This module provides a simplified framework for implementing nodes.
//! By implementing the `SimpleNode` trait and wrapping it in a `BaseNodeAdapter`,
//! developers can focus on message processing without managing the full
//! `NodeBehavior` async trait and resource lifecycle.

use super::{NodeBehavior, NodeContext};
use anyhow::Result;
use plantlink_core::MessagePayload;

/// A simplified trait for nodes that primarily process messages.
///
/// Implemented by nodes that don't need low-level control over their
/// concurrent lifecycle. The methods are called by `BaseNodeAdapter`
/// which handles error reporting and logging.
#[async_trait::async_trait]
pub trait SimpleNode: Send + Sync {
    /// Called when the node is started. Can return initial state or error.
    async fn on_start(&mut self, _ctx: &NodeContext) -> Result<()> {
        Ok(())
    }

    /// Handle an incoming message.
    ///
    /// This is the primary processing hook. It is called by the adapter's
    /// `receive` implementation. Logic should be non-blocking where possible.
    ///
    /// # Port Routing
    /// - `port`: The index of the input port receiving the message.
    /// - `msg`: An `Arc`-wrapped `MessagePayload`.
    async fn handle(
        &mut self,
        port: usize,
        msg: std::sync::Arc<plantlink_core::MessagePayload>,
        ctx: &NodeContext,
    ) -> Result<()>;

    /// Called when the node is shut down.
    async fn on_stop(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A wrapper that adapts a `SimpleNode` into a full `NodeBehavior`.
///
/// The adapter provides several automatic features for `SimpleNode` implementations:
/// 1. **Status Reporting**: Broadcasts "running" status on start.
/// 2. **Error Capture**: Catches errors from `handle` and broadcasts them as "error" status.
/// 3. **Logging**: Automatically logs received errors to the system channel.
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
        ctx.emit_running("Running");
        self.inner.on_start(&ctx).await
    }

    async fn receive(
        &mut self,
        port: usize,
        msg: std::sync::Arc<MessagePayload>,
        ctx: &NodeContext,
    ) -> Result<()> {
        // We could wrap this in a catch_unwind or specific error handling logging
        if let Err(e) = self.inner.handle(port, msg, ctx).await {
            // Automatic Error Reporting via System Channel
            ctx.emit_error(&e.to_string());

            // Also log
            ctx.emit_log(format!("Node [{}]: Error: {}", ctx.id, e));

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
    use super::{BaseNodeAdapter, NodeBehavior, NodeContext, SimpleNode};
    use anyhow::Result;
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
            _msg: std::sync::Arc<MessagePayload>,
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
    async fn test_adapter_receive_error_broadcasts_status() {
        let node = DummySimpleNode::new(true);
        let mut adapter = BaseNodeAdapter::new(node);
        let (ctx, mut sys_rx) = NodeContext::for_test("base-err");
        let result = adapter
            .receive(0, std::sync::Arc::new(MessagePayload::default()), &ctx)
            .await;
        assert!(result.is_err(), "Expected error from failing handle");
        // Drain and look for "error" status in broadcast
        let mut found = false;
        while let Ok(msg) = sys_rx.try_recv() {
            if let crate::nodes::SystemEvent::Status { data } = msg
                && data.state == "error"
            {
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
 
    #[tokio::test]
    async fn test_adapter_start_emits_running() {
        let node = DummySimpleNode::new(false);
        let mut adapter = BaseNodeAdapter::new(node);
        let (ctx, mut sys_rx) = NodeContext::for_test("base-running");
 
        adapter.start(ctx).await.unwrap();
 
        let mut found = false;
        while let Ok(msg) = sys_rx.try_recv() {
            if let crate::nodes::SystemEvent::Status { data } = msg
                && data.state == "running"
            {
                found = true;
            }
        }
        assert!(found, "Expected 'running' status broadcast from adapter start");
    }
}
