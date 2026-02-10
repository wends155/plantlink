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
            .and_then(|v| v.as_u64())
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

            // Spawn a background task for the timer
            let handle = tokio::spawn(async move {
                let mut timer = tokio::time::interval(interval);
                timer.tick().await; // First tick is immediate
                loop {
                    timer.tick().await;
                    let msg = MessagePayload {
                        payload: plantlink_core::DataValue::String(payload.clone()),
                        ..Default::default()
                    };
                    ctx_clone.send_output(msg).await;
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
        _msg: MessagePayload,
        ctx: &NodeContext,
    ) -> Result<()> {
        // Trigger mode: output the configured payload immediately
        let msg = MessagePayload {
            payload: plantlink_core::DataValue::String(self.payload.clone()),
            ..Default::default()
        };
        ctx.send_output(msg).await;
        Ok(())
    }

    async fn on_stop(&mut self) -> Result<()> {
        if let Some(handle) = self.timer_handle.take() {
            handle.abort();
        }
        Ok(())
    }
}
