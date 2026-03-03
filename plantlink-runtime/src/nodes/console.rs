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

        if let Err(e) = ctx.system_tx.send(json) {
            tracing::warn!(node_id = %ctx.id, "Failed to broadcast log: {}", e);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeConfig;
    use crate::nodes::NodeContext;
    use plantlink_core::{DataValue, MessagePayload};

    fn make_ctx(id: &str) -> (NodeContext, tokio::sync::broadcast::Receiver<String>) {
        NodeContext::for_test(id)
    }

    #[tokio::test]
    async fn test_console_on_input_broadcasts_log() {
        let (ctx, mut rx) = make_ctx("c1");
        let mut node = ConsoleNode::new(&NodeConfig {
            id: "c1".into(),
            type_: "console".into(),
            data: serde_json::json!({}),
        });
        let msg = MessagePayload {
            payload: DataValue::String("hello".into()),
            ..Default::default()
        };
        node.on_input(0, msg, ctx).await.unwrap();
        let broadcast = rx.try_recv().expect("Expected broadcast");
        assert!(
            broadcast.contains("\"type\":\"log\""),
            "Expected type:log, got: {broadcast}"
        );
        assert!(
            broadcast.contains("hello"),
            "Expected payload in log, got: {broadcast}"
        );
    }

    #[tokio::test]
    async fn test_console_formats_node_id() {
        let (ctx, mut rx) = make_ctx("my-node-id");
        let mut node = ConsoleNode::new(&NodeConfig {
            id: "my-node-id".into(),
            type_: "console".into(),
            data: serde_json::json!({}),
        });
        node.on_input(0, MessagePayload::default(), ctx)
            .await
            .unwrap();
        let broadcast = rx.try_recv().expect("Expected broadcast");
        assert!(
            broadcast.contains("my-node-id"),
            "Expected node id in log, got: {broadcast}"
        );
    }
}
