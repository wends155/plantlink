use crate::traits::{PubSubClient, PubSubMessage};
use anyhow::{Context, Result};
use async_nats::Client;
use futures::{StreamExt, stream::BoxStream};

#[derive(Clone)]
/// Manages a NATS client connection with publish/subscribe capabilities.
///
/// # Examples
///
/// ```no_run
/// use plantlink_core::nats::NatsDriver;
/// use plantlink_core::traits::PubSubClient;
///
/// # async fn example() -> anyhow::Result<()> {
/// let driver = NatsDriver::connect("nats://localhost:4222").await?;
/// driver.publish("events.sensor", bytes::Bytes::from("hello")).await?;
/// let mut sub = driver.subscribe("events.>").await?;
/// # Ok(())
/// # }
/// ```
pub struct NatsDriver {
    client: Client,
}

impl NatsDriver {
    ///
    /// # Errors
    /// Returns an error if connecting to the NATS server fails.
    #[tracing::instrument(err)]
    pub async fn connect(url: &str) -> Result<Self> {
        let client = async_nats::connect(url)
            .await
            .context("Failed to connect to NATS")?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl PubSubClient for NatsDriver {
    #[tracing::instrument(skip(self, payload), err)]
    async fn publish(&self, subject: &str, payload: bytes::Bytes) -> Result<()> {
        self.client
            .publish(subject.to_string(), payload)
            .await
            .context("Failed to publish")?;
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    async fn subscribe(&self, subject: &str) -> Result<BoxStream<'static, PubSubMessage>> {
        let subscriber = self
            .client
            .subscribe(subject.to_string())
            .await
            .context("Failed to subscribe")?;

        let stream = subscriber.map(|nats_msg| PubSubMessage {
            topic: nats_msg.subject.to_string(),
            payload: nats_msg.payload,
        });

        Ok(stream.boxed())
    }
}
