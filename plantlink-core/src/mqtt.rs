use crate::traits::{PubSubClient, PubSubMessage};
use anyhow::Result;
use futures::stream::BoxStream;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;

/// Manages a persistent MQTT client connection with automatic reconnection.
///
/// Uses exponential backoff (1s–60s) for the event loop retry strategy.
///
/// # Examples
///
/// ```no_run
/// use plantlink_core::mqtt::MqttDriver;
/// use plantlink_core::traits::PubSubClient;
///
/// # async fn example() -> anyhow::Result<()> {
/// let driver = MqttDriver::connect("plant-01", "localhost", 1883).await?;
/// driver.publish("sensors/temp", bytes::Bytes::from("hello")).await?;
/// # Ok(())
/// # }
/// ```
pub struct MqttDriver {
    client: AsyncClient,
}

impl MqttDriver {
    ///
    /// # Errors
    /// Returns an error if the MQTT options are invalid.
    #[allow(clippy::unused_async)]
    #[tracing::instrument(skip(id, host, port), err)]
    pub async fn connect(id: &str, host: &str, port: u16) -> Result<Self> {
        let mut mqttoptions = MqttOptions::new(id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // Spawn event loop handler with exponential backoff retry
        tokio::spawn(async move {
            let mut backoff = Duration::from_secs(1);
            let max_backoff = Duration::from_secs(60);

            loop {
                match eventloop.poll().await {
                    Ok(_event) => {
                        // Reset backoff on successful poll/reconnection
                        backoff = Duration::from_secs(1);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "MQTT event loop error: {}. Retrying in {:?}...",
                            e,
                            backoff
                        );
                        tokio::time::sleep(backoff).await;
                        // Exponential backoff capped at max_backoff
                        backoff = (backoff * 2).min(max_backoff);
                    }
                }
            }
        });

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl PubSubClient for MqttDriver {
    #[tracing::instrument(skip(self, payload), err)]
    async fn publish(&self, topic: &str, payload: bytes::Bytes) -> Result<()> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload.to_vec())
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    async fn subscribe(&self, _topic: &str) -> Result<BoxStream<'static, PubSubMessage>> {
        Err(anyhow::anyhow!("MQTT subscribe not implemented yet"))
    }
}
