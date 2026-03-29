//! NATS Protocol Nodes
//!
//! This module provides nodes for interacting with NATS messaging brokers.
//! It implements a decoupled resource model where a `NatsBrokerNode` manages
//! the connection and provides it to `NatsSubNode` and `NatsPubNode` via a
//! shared resource registry indexed by `broker_id`.

use super::{NodeBehavior, NodeContext};
use anyhow::Result;
use futures::StreamExt;
use plantlink_core::traits::PubSubClient;
use plantlink_core::{DataValue, MessagePayload};
use std::sync::Arc;

/// A node that manages a connection to a NATS broker.
///
/// This node connects to a server and registers the resulting `PubSubClient`
/// in the shared resource registry. This allows other nodes to reference
/// the connection by the `id` of this broker node.
///
/// # Configuration (`data`)
/// - `url`: The NATS server URL (e.g., `nats://localhost:4222`).
pub struct NatsBrokerNode {
    url: String,
    id: String,
}

impl NatsBrokerNode {
    pub fn new(config: &crate::NodeConfig) -> Self {
        let url = config
            .data
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("nats://localhost:4222")
            .to_string();
        Self {
            url,
            id: config.id.clone(),
        }
    }
}

#[async_trait::async_trait]
impl NodeBehavior for NatsBrokerNode {
    async fn start(&mut self, ctx: NodeContext) -> Result<()> {
        tracing::info!("NatsBroker starting, connecting to {}", self.url);

        match plantlink_core::nats::NatsDriver::connect(&self.url).await {
            Ok(driver) => {
                // Register connection in shared resources using our own ID
                {
                    let mut resources = ctx.resources.write().await;
                    resources.insert(
                        self.id.clone(),
                        Box::new(Arc::new(driver) as Arc<dyn PubSubClient>),
                    );
                }

                ctx.emit_running(&format!("Connected to {}", self.url));
                Ok(())
            }
            Err(e) => {
                ctx.emit_error(&format!("Connection failed: {e}"));
                Err(anyhow::anyhow!("Connection failed: {e}"))
            }
        }
    }
}

/// A node that subscribes to a NATS subject.
///
/// This node retrieves a NATS client from the shared resource registry
/// using its configured `broker_id` and spawns a background task to
/// listener for messages.
///
/// # Configuration (`data`)
/// - `subject`: The NATS subject to subscribe to (e.g., `sensors.temp`).
/// - `broker`: The ID of the `NatsBrokerNode` providing the connection.
///
/// # Input Ports
/// - `0`: Accepts a `DataValue::String` containing a new `broker_id` to
///   dynamically re-assign the node's broker relationship.
pub struct NatsSubNode {
    subject: String,
    broker_id: String,
    sub_handle: Option<tokio::task::JoinHandle<()>>,
}

impl NatsSubNode {
    pub fn new(config: &crate::NodeConfig) -> Self {
        let subject = config
            .data
            .get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let broker_id = config
            .data
            .get("broker")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Self {
            subject,
            broker_id,
            sub_handle: None,
        }
    }
}

#[async_trait::async_trait]
impl NodeBehavior for NatsSubNode {
    async fn receive(
        &mut self,
        port_idx: usize,
        msg: Arc<MessagePayload>,
        ctx: &NodeContext,
    ) -> Result<()> {
        if let (0, DataValue::String(id)) = (port_idx, &msg.payload) {
            self.broker_id.clone_from(id);
        }

        // Retrieve Driver
        let driver = {
            let resources = ctx.resources.read().await;
            match resources
                .get(&self.broker_id)
                .and_then(|a| a.downcast_ref::<Arc<dyn PubSubClient>>())
            {
                Some(d) => d.clone(),
                None => return Ok(()),
            }
        };

        // Subscribe
        let mut subscriber = match driver.subscribe(&self.subject).await {
            Ok(s) => s,
            Err(e) => {
                ctx.emit_error(&format!("Subscribe failed: {e}"));
                return Err(anyhow::anyhow!("Subscribe failed: {e}"));
            }
        };

        // Spawn listener
        let ctx = ctx.clone();
        // ast-grep-ignore: deferred to structured-spawn plan
        let handle = tokio::spawn(async move {
            while let Some(nats_msg) = subscriber.next().await {
                let payload_str = String::from_utf8_lossy(&nats_msg.payload).to_string();
                let out_msg = MessagePayload {
                    payload: DataValue::String(payload_str),
                    topic: Some(nats_msg.topic.clone()),
                    ..Default::default()
                };
                if let Err(e) = ctx.send_output(out_msg).await {
                    tracing::warn!("NatsSub: failed to forward message: {}", e);
                }
            }
        });

        if let Some(h) = self.sub_handle.replace(handle) {
            h.abort();
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(h) = self.sub_handle.take() {
            h.abort();
        }
        Ok(())
    }
}

/// A node that publishes messages to a NATS subject.
///
/// This node retrieves a NATS client from the shared resource registry
/// and publishes incoming payloads. If the node's `subject` is empty,
/// it will attempt to use the `topic` field from the incoming `MessagePayload`.
///
/// # Configuration (`data`)
/// - `subject`: Fixed NATS subject. If empty, uses the message topic.
/// - `broker`: The ID of the `NatsBrokerNode` providing the connection.
///
/// # Input Ports
/// - `0`: Accepts a `DataValue::String` to dynamically update the `broker_id`.
/// - `1`: Receives payloads to be published to NATS.
pub struct NatsPubNode {
    subject: String,
    broker_id: String,
}

impl NatsPubNode {
    pub fn new(config: &crate::NodeConfig) -> Self {
        let subject = config
            .data
            .get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let broker_id = config
            .data
            .get("broker")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Self { subject, broker_id }
    }
}

#[async_trait::async_trait]
impl NodeBehavior for NatsPubNode {
    async fn receive(
        &mut self,
        port_idx: usize,
        msg: Arc<MessagePayload>,
        ctx: &NodeContext,
    ) -> Result<()> {
        if port_idx == 0 {
            // Unpack Connection ID
            if let DataValue::String(id) = &msg.payload {
                self.broker_id.clone_from(id);
            }
            return Ok(());
        }

        if self.broker_id.is_empty() {
            tracing::warn!("NatsPub: No active connection");
        } else {
            // Get Driver
            let driver = {
                let resources = ctx.resources.read().await;
                match resources
                    .get(&self.broker_id)
                    .and_then(|a| a.downcast_ref::<Arc<dyn PubSubClient>>())
                {
                    Some(d) => d.clone(),
                    None => return Ok(()),
                }
            };

            // Publish
            let payload_bytes = match &msg.payload {
                DataValue::String(s) => bytes::Bytes::copy_from_slice(s.as_bytes()),
                DataValue::Json(v) => bytes::Bytes::from(v.to_string()),
                _ => bytes::Bytes::new(),
            };

            let target_subject = if !self.subject.is_empty() {
                &self.subject
            } else if let Some(t) = &msg.topic {
                t
            } else {
                return Ok(());
            };

            if let Err(e) = driver.publish(target_subject, payload_bytes).await {
                ctx.emit_error(&format!("Publish failed: {e}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{NatsBrokerNode, NatsPubNode, NatsSubNode, NodeBehavior, NodeContext};
    use crate::NodeConfig;
    use plantlink_core::{DataValue, MessagePayload};
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn test_broker_config_parsing() {
        let cfg = NodeConfig {
            id: "b1".into(),
            type_: "nats-broker".into(),
            data: json!({ "url": "nats://test:4222" }),
        };
        let node = NatsBrokerNode::new(&cfg);
        assert_eq!(node.url, "nats://test:4222");
        assert_eq!(node.id, "b1");
    }

    #[test]
    fn test_sub_config_parsing() {
        let cfg = NodeConfig {
            id: "s1".into(),
            type_: "nats-sub".into(),
            data: json!({ "subject": "test.*", "broker": "b1" }),
        };
        let node = NatsSubNode::new(&cfg);
        assert_eq!(node.subject, "test.*");
        assert_eq!(node.broker_id, "b1");
    }

    #[test]
    fn test_pub_config_parsing() {
        let cfg = NodeConfig {
            id: "p1".into(),
            type_: "nats-pub".into(),
            data: json!({ "subject": "test.out", "broker": "b1" }),
        };
        let node = NatsPubNode::new(&cfg);
        assert_eq!(node.subject, "test.out");
        assert_eq!(node.broker_id, "b1");
    }

    #[tokio::test]
    async fn test_sub_dynamic_broker_assignment() {
        let cfg = NodeConfig {
            id: "s1".into(),
            type_: "nats-sub".into(),
            data: json!({ "subject": "test", "broker": "old" }),
        };
        let mut node = NatsSubNode::new(&cfg);
        let msg = Arc::new(MessagePayload {
            payload: DataValue::String("new-broker".into()),
            ..Default::default()
        });
        let (ctx, _) = NodeContext::for_test("s1");
        // ast-grep-ignore
        node.receive(0, msg, &ctx).await.unwrap();
        assert_eq!(node.broker_id, "new-broker");
    }

    #[tokio::test]
    async fn test_pub_dynamic_broker_assignment() {
        let cfg = NodeConfig {
            id: "p1".into(),
            type_: "nats-pub".into(),
            data: json!({ "subject": "test", "broker": "old" }),
        };
        let mut node = NatsPubNode::new(&cfg);
        let msg = Arc::new(MessagePayload {
            payload: DataValue::String("new-broker".into()),
            ..Default::default()
        });
        let (ctx, _) = NodeContext::for_test("p1");
        // ast-grep-ignore
        node.receive(0, msg, &ctx).await.unwrap();
        assert_eq!(node.broker_id, "new-broker");
    }
}
