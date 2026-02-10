use super::{NodeBehavior, NodeContext};
use anyhow::Result;
use futures::StreamExt;
use plantlink_core::{DataValue, MessagePayload, nats::NatsDriver};

// --- NATS Broker Node ---
pub struct NatsBrokerNode {
    url: String,
    conn_id: String,
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
            conn_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[async_trait::async_trait]
impl NodeBehavior for NatsBrokerNode {
    async fn start(&mut self, ctx: NodeContext) -> Result<()> {
        tracing::info!("NatsBroker starting, connecting to {}", self.url);

        match NatsDriver::connect(&self.url).await {
            Ok(driver) => {
                // Register connection in shared resources
                {
                    let mut resources = ctx.resources.write().await;
                    resources.insert(self.conn_id.clone(), Box::new(driver));
                }

                // Emit success status
                ctx.emit_running(&format!("Connected to {}", self.url));

                // Broadcast the Connection ID to downstream nodes
                let msg = MessagePayload {
                    payload: DataValue::String(self.conn_id.clone()),
                    ..Default::default()
                };
                ctx.send_output(msg).await;

                Ok(())
            }
            Err(e) => {
                // Emit error status
                ctx.emit_error(&format!("Connection failed: {}", e));
                Err(e)
            }
        }
    }
}

// --- NATS Sub Node ---
pub struct NatsSubNode {
    subject: String,
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
        Self {
            subject,
            sub_handle: None,
        }
    }
}

#[async_trait::async_trait]
impl NodeBehavior for NatsSubNode {
    async fn on_input(
        &mut self,
        _port_idx: usize,
        msg: MessagePayload,
        ctx: NodeContext,
    ) -> Result<()> {
        // Expect Connection ID in payload
        let conn_id = match msg.payload {
            DataValue::String(s) => s,
            _ => return Ok(()), // Ignore invalid payloads
        };

        // Retrieve Driver
        let driver = {
            let resources = ctx.resources.read().await;
            if let Some(any) = resources.get(&conn_id) {
                if let Some(driver) = any.downcast_ref::<NatsDriver>() {
                    driver.clone()
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        };

        // Subscribe
        let mut subscriber = match driver.subscribe(&self.subject).await {
            Ok(s) => s,
            Err(e) => {
                ctx.emit_error(&format!("Subscribe failed: {}", e));
                return Err(e);
            }
        };
        // let subject = self.subject.clone(); // Not used

        // Spawn listener
        // Move ctx into task to send outputs
        let handle = tokio::spawn(async move {
            while let Some(nats_msg) = subscriber.next().await {
                let payload_str = String::from_utf8_lossy(&nats_msg.payload).to_string();
                let out_msg = MessagePayload {
                    payload: DataValue::String(payload_str), // For now assume string
                    topic: Some(nats_msg.subject.to_string()),
                    ..Default::default()
                };
                ctx.send_output(out_msg).await;
            }
        });

        // Store handle to abort later if needed (though on_input might be called multiple times?)
        // If called multiple times (multiple brokers?), we spawn multiple subs?
        // Ideally we should stop previous sub if any.
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

// --- NATS Pub Node ---
pub struct NatsPubNode {
    subject: String,
    active_conn_id: Option<String>,
}

impl NatsPubNode {
    pub fn new(config: &crate::NodeConfig) -> Self {
        let subject = config
            .data
            .get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Self {
            subject,
            active_conn_id: None,
        }
    }
}

#[async_trait::async_trait]
impl NodeBehavior for NatsPubNode {
    async fn on_input(
        &mut self,
        port_idx: usize,
        msg: MessagePayload,
        ctx: NodeContext,
    ) -> Result<()> {
        if port_idx == 0 {
            // Unpack Connection ID (Port 0)
            if let DataValue::String(id) = msg.payload {
                self.active_conn_id = Some(id);
            }
        } else if port_idx == 1 {
            // Data Payload (Port 1)
            if let Some(conn_id) = &self.active_conn_id {
                // Get Driver
                let driver = {
                    let resources = ctx.resources.read().await;
                    match resources
                        .get(conn_id)
                        .and_then(|a| a.downcast_ref::<NatsDriver>())
                    {
                        Some(d) => d.clone(),
                        None => return Ok(()),
                    }
                };

                // Publish
                let payload_bytes = match msg.payload {
                    DataValue::String(s) => s.into(),
                    DataValue::Json(v) => v.to_string().into(),
                    _ => bytes::Bytes::from(""),
                };

                // Use config subject, or msg topic if config is empty?
                let target_subject = if !self.subject.is_empty() {
                    &self.subject
                } else if let Some(t) = &msg.topic {
                    t
                } else {
                    return Ok(());
                };

                if let Err(e) = driver.publish(target_subject, payload_bytes).await {
                    ctx.emit_error(&format!("Publish failed: {}", e));
                }
            } else {
                tracing::warn!("NatsPub: No active connection");
            }
        }
        Ok(())
    }
}
