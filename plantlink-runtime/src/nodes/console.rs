use super::{NodeBehavior, NodeContext, SystemEvent};
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
    async fn receive(
        &mut self,
        _port: usize,
        msg: std::sync::Arc<MessagePayload>,
        ctx: &NodeContext,
    ) -> Result<()> {
        let payload_str = msg.payload.to_string();
        let log_msg = format!("Console [{}]: {}", ctx.id, payload_str);

        // Broadcast to WebSocket via System Channel
        let event = SystemEvent::Log { message: log_msg };

        if let Err(e) = ctx.system_tx.send(event) {
            tracing::warn!(node_id = %ctx.id, "Failed to broadcast log: {}", e);
        }
        Ok(())
    }
}

// ast-grep-ignore
#[cfg(test)]
mod tests {
    use super::ConsoleNode;
    use crate::NodeConfig;
    use crate::nodes::{NodeBehavior, NodeContext};
    use plantlink_core::{DataValue, MessagePayload};

    fn make_ctx(
        id: &str,
    ) -> (
        NodeContext,
        tokio::sync::broadcast::Receiver<crate::nodes::SystemEvent>,
    ) {
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
        node.receive(0, std::sync::Arc::new(msg), &ctx)
            .await
            // ast-grep-ignore
            .unwrap();
        // ast-grep-ignore
        let broadcast = rx.try_recv().expect("Expected broadcast");
        if let crate::nodes::SystemEvent::Log { message } = broadcast {
            assert!(
                message.contains("hello"),
                "Expected payload in log, got: {message}"
            );
        } else {
            panic!("Expected SystemEvent::Log, got {broadcast:?}");
        }
    }

    #[tokio::test]
    async fn test_console_formats_node_id() {
        let (ctx, mut rx) = make_ctx("my-node-id");
        let mut node = ConsoleNode::new(&NodeConfig {
            id: "my-node-id".into(),
            type_: "console".into(),
            data: serde_json::json!({}),
        });
        node.receive(0, std::sync::Arc::new(MessagePayload::default()), &ctx)
            .await
            // ast-grep-ignore
            .unwrap();
        // ast-grep-ignore
        let broadcast = rx.try_recv().expect("Expected broadcast");
        if let crate::nodes::SystemEvent::Log { message } = broadcast {
            assert!(
                message.contains("my-node-id"),
                "Expected node id in log, got: {message}"
            );
        } else {
            panic!("Expected SystemEvent::Log, got {broadcast:?}");
        }
    }
}
