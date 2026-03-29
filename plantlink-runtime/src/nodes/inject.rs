//! # Inject Node
//!
//! Provides the primary mechanism for triggering flows manually or on an interval.

use super::NodeContext;
use anyhow::Result;
use plantlink_core::MessagePayload;
use std::time::Duration;

/// A node that emits predefined messages into the flow.
///
/// Can operate in two modes: manual trigger (outputs once, or when told)
/// and interval mode (spawns a background timer to output repeatedly).
pub struct InjectNode {
    payload: String,
    interval_secs: u64,
}

impl InjectNode {
    /// Constructs a new `InjectNode` from its configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Node configuration extracting the `payload` and `interval` fields.
    ///
    /// # Returns
    ///
    /// A new `InjectNode` initialized with the provided arguments.
    pub fn new(config: &crate::NodeConfig) -> Self {
        let payload = config
            .data
            .get("payload")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let interval_secs = config
            .data
            .get("interval")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        Self {
            payload,
            interval_secs,
        }
    }
}

use super::base::SimpleNode;

#[async_trait::async_trait]
impl SimpleNode for InjectNode {
    async fn on_start(&mut self, ctx: &NodeContext) -> Result<()> {
        if self.interval_secs > 0 {
            let interval = Duration::from_secs(self.interval_secs);
            let payload = self.payload.clone();
            let ctx_clone = ctx.clone();

            let cancel = ctx.cancel.clone();

            // Spawn a background task for the timer
            ctx.tracker.spawn(async move {
                let mut timer = tokio::time::interval(interval);
                timer.tick().await; // First tick is immediate
                loop {
                    tokio::select! {
                        () = cancel.cancelled() => {
                            tracing::info!("InjectNode timer: cancelled");
                            break;
                        }
                        _ = timer.tick() => {
                            let msg = MessagePayload {
                                payload: plantlink_core::DataValue::String(payload.clone()),
                                ..Default::default()
                            };
                            if let Err(e) = ctx_clone.send_output(msg).await {
                                tracing::warn!("InjectNode timer: channel closed, stopping: {}", e);
                                ctx_clone.emit_stopped("Channel closed");
                                break;
                            }
                        }
                    }
                }
            });
            ctx.emit_running(&format!("Timer started ({}s interval)", self.interval_secs));
        } else {
            ctx.emit_running("Trigger mode ready");
        }
        Ok(())
    }

    async fn handle(
        &mut self,
        _port: usize,
        _msg: std::sync::Arc<MessagePayload>,
        ctx: &NodeContext,
    ) -> Result<()> {
        // Trigger mode: output the configured payload immediately
        let msg = MessagePayload {
            payload: plantlink_core::DataValue::String(self.payload.clone()),
            ..Default::default()
        };
        ctx.send_output(msg).await?;
        Ok(())
    }

    async fn on_stop(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::InjectNode;
    use crate::NodeConfig;
    use crate::nodes::NodeContext;
    use crate::nodes::OutputMap;
    use crate::nodes::base::SimpleNode;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;
    use tokio::sync::{broadcast, mpsc};
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn test_inject_timer_stops_on_cancellation() {
        let (sys_tx, _) = broadcast::channel(16);
        let cancel = CancellationToken::new();
        let tracker = crate::nodes::TaskTracker::new();
        let ctx = NodeContext::new(
            "inject-test".into(),
            HashMap::new(),
            Arc::new(RwLock::new(HashMap::new())),
            sys_tx,
            cancel.clone(),
            tracker.clone(),
        );
        let mut node = InjectNode::new(&NodeConfig {
            id: "inject-test".into(),
            type_: "inject".into(),
            data: serde_json::json!({"interval": 1, "payload": "test"}),
        });

        node.on_start(&ctx).await.unwrap();

        // Close tracker and cancel
        tracker.close();
        cancel.cancel();

        // Wait for timer to exit via tracker
        tokio::time::timeout(Duration::from_millis(100), tracker.wait())
            .await
            .expect("Timer should have exited after cancellation");
    }

    #[tokio::test]
    async fn test_inject_timer_stops_on_closed_channel() {
        let (tx, rx) = mpsc::channel(16);
        let (sys_tx, _) = broadcast::channel(16);
        let cancel = CancellationToken::new();
        let tracker = crate::nodes::TaskTracker::new();
        let mut outputs: OutputMap = HashMap::new();
        outputs.insert(0, vec![(tx, 0)]);

        let ctx = NodeContext::new(
            "inject-test".into(),
            outputs,
            Arc::new(RwLock::new(HashMap::new())),
            sys_tx,
            cancel,
            tracker.clone(),
        );
        let mut node = InjectNode::new(&NodeConfig {
            id: "inject-test".into(),
            type_: "inject".into(),
            data: serde_json::json!({"interval": 1, "payload": "test"}),
        });

        node.on_start(&ctx).await.unwrap();

        // Close tracker
        tracker.close();

        // Drop receiver to close channel
        drop(rx);

        // Wait for next tick (interval 1s) and check exit via tracker
        tokio::time::timeout(Duration::from_millis(1100), tracker.wait())
            .await
            .expect("Timer should have exited after channel closed");
    }

    #[tokio::test]
    async fn test_inject_node_timer_is_tracked() {
        let (sys_tx, _) = broadcast::channel(16);
        let cancel = CancellationToken::new();
        let tracker = tokio_util::task::TaskTracker::new();
        let ctx = NodeContext::new(
            "inject-test".into(),
            HashMap::new(),
            Arc::new(RwLock::new(HashMap::new())),
            sys_tx,
            cancel.clone(),
            tracker.clone(),
        );
        let mut node = InjectNode::new(&NodeConfig {
            id: "inject-test".into(),
            type_: "inject".into(),
            data: serde_json::json!({"interval": 1, "payload": "test"}),
        });

        node.on_start(&ctx).await.unwrap();

        // Close tracker to signal we are waiting for existing tasks
        tracker.close();

        // If tracked correctly, tracker.wait() will block until timer task exits.
        // Since timer task is currently running (interval 1s), we expect a timeout to fail the assertion
        // if we were in the Red phase (but here it should block).
        // Actually, the plan says: "If tracked correctly, tracker.wait() will block until timer task exits."
        // Let's prove it blocks.
        let wait_fut = tracker.wait();
        tokio::pin!(wait_fut);

        tokio::select! {
            () = &mut wait_fut => {
                panic!("Tracker should be blocking because the timer task is still running!");
            }
            () = tokio::time::sleep(Duration::from_millis(50)) => {
                // Success: tracker is correctly tracking the task and blocking
                tracing::info!("Tracker correctly blocked; task is tracked.");
            }
        }

        // Now cancel and verify it finishes
        cancel.cancel();
        tokio::time::timeout(Duration::from_millis(100), wait_fut)
            .await
            .expect("Tracker should have finished waiting after cancellation");
    }
}
