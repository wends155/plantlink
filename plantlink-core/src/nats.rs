use anyhow::{Context, Result};
use async_nats::Client;

#[derive(Clone)]
/// Manages a NATS client connection with publish/subscribe capabilities.
///
/// # Examples
///
/// ```no_run
/// use plantlink_core::nats::NatsDriver;
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
    pub async fn connect(url: &str) -> Result<Self> {
        let client = async_nats::connect(url)
            .await
            .context("Failed to connect to NATS")?;
        Ok(Self { client })
    }

    pub async fn publish(&self, subject: &str, payload: bytes::Bytes) -> Result<()> {
        self.client
            .publish(subject.to_string(), payload)
            .await
            .context("Failed to publish")?;
        Ok(())
    }

    pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber> {
        self.client
            .subscribe(subject.to_string())
            .await
            .context("Failed to subscribe")
    }
}
