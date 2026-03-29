//! # NATS Driver
//!
//! This module provides the [`NatsDriver`] implementation of the [`PubSubClient`] trait.
//! It uses the `async-nats` crate for asynchronous communication with NATS servers.

use crate::traits::{PubSubClient, PubSubMessage};
use crate::PlantLinkError;
use futures::{StreamExt, stream::BoxStream};
use async_nats::Client;

/// Manages a NATS client connection with publish/subscribe capabilities.
///
/// # Examples
///
/// ```no_run
/// use plantlink_core::nats::NatsDriver;
/// use plantlink_core::traits::PubSubClient;
///
/// # async fn example() -> Result<(), plantlink_core::PlantLinkError> {
/// let driver = NatsDriver::connect("nats://localhost:4222").await?;
/// driver.publish("events.sensor", bytes::Bytes::from("hello")).await?;
/// let mut sub = driver.subscribe("events.>").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct NatsDriver {
    client: Client,
}

impl NatsDriver {
    /// Connects to a NATS server at the specified URL.
    ///
    /// # Errors
    /// Returns a [`PlantLinkError::Connection`] if the NATS server is unreachable.
    #[tracing::instrument(err)]
    pub async fn connect(url: &str) -> Result<Self, PlantLinkError> {
        let client = async_nats::connect(url)
            .await
            .map_err(|e| PlantLinkError::Connection(e.to_string()))?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl PubSubClient for NatsDriver {
    #[tracing::instrument(skip(self, payload), err)]
    async fn publish(&self, subject: &str, payload: bytes::Bytes) -> Result<(), PlantLinkError> {
        self.client
            .publish(subject.to_string(), payload)
            .await
            .map_err(|e| PlantLinkError::Publish(e.to_string()))?;
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    async fn subscribe(&self, subject: &str) -> Result<BoxStream<'static, PubSubMessage>, PlantLinkError> {
        let subscriber = self
            .client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| PlantLinkError::Subscribe(e.to_string()))?;

        let stream = subscriber.map(|nats_msg| PubSubMessage {
            topic: nats_msg.subject.to_string(),
            payload: nats_msg.payload,
        });

        Ok(stream.boxed())
    }
}
