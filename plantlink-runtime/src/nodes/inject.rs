use super::NodeContext;
use anyhow::Result;
use plantlink_core::MessagePayload;
use std::time::Duration;
use tokio::task::JoinHandle;

pub struct InjectNode {
    payload: String,
    interval_secs: u64,
    timer_handle: Option<JoinHandle<()>>,
}

impl InjectNode {
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
            timer_handle: None,
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
            let handle = tokio::spawn(async move {
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
            self.timer_handle = Some(handle);
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
        if let Some(handle) = self.timer_handle.take() {
            handle.abort();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeConfig;
    use crate::nodes::OutputMap;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::sync::{broadcast, mpsc};
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn test_inject_timer_stops_on_cancellation() {
        let (sys_tx, _) = broadcast::channel(16);
        let cancel = CancellationToken::new();
        let ctx = NodeContext::new(
            "inject-test".into(),
            HashMap::new(),
            Arc::new(RwLock::new(HashMap::new())),
            sys_tx,
            cancel.clone(),
        );
        let mut node = InjectNode::new(&NodeConfig {
            id: "inject-test".into(),
            type_: "inject".into(),
            data: serde_json::json!({"interval": 1, "payload": "test"}),
        });
        node.on_start(&ctx).await.unwrap();

        // Cancel and wait for timer to exit
        cancel.cancel();
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(
            node.timer_handle.as_ref().unwrap().is_finished(),
            "Timer should have exited after cancellation"
        );
    }

    #[tokio::test]
    async fn test_inject_timer_stops_on_closed_channel() {
        let (tx, rx) = mpsc::channel(16);
        let (sys_tx, _) = broadcast::channel(16);
        let cancel = CancellationToken::new();
        let mut outputs: OutputMap = HashMap::new();
        outputs.insert(0, vec![(tx, 0)]);

        let ctx = NodeContext::new(
            "inject-test".into(),
            outputs,
            Arc::new(RwLock::new(HashMap::new())),
            sys_tx,
            cancel,
        );
        let mut node = InjectNode::new(&NodeConfig {
            id: "inject-test".into(),
            type_: "inject".into(),
            data: serde_json::json!({"interval": 1, "payload": "test"}),
        });
        node.on_start(&ctx).await.unwrap();

        // Drop receiver to close channel
        drop(rx);

        // Wait for next tick (interval 1s)
        tokio::time::sleep(Duration::from_millis(1100)).await;

        assert!(
            node.timer_handle.as_ref().unwrap().is_finished(),
            "Timer should have exited after channel closed"
        );
    }
}
