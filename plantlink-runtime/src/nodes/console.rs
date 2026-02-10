use super::{NodeBehavior, NodeContext};
use anyhow::Result;
use plantlink_core::MessagePayload;

pub struct ConsoleNode;

impl ConsoleNode {
    pub fn new(_config: &crate::NodeConfig) -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl NodeBehavior for ConsoleNode {
    async fn on_input(
        &mut self,
        _port: usize,
        msg: MessagePayload,
        ctx: NodeContext,
    ) -> Result<()> {
        let payload_str = msg.payload.to_string();
        let log_msg = format!("Console [{}]: {}", ctx.id, payload_str);

        // Broadcast to WebSocket via System Channel
        let json = serde_json::json!({
            "type": "log",
            "message": log_msg
        })
        .to_string();

        let _ = ctx.system_tx.send(json);
        Ok(())
    }
}
