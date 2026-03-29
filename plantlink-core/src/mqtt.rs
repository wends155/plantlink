//! # MQTT Driver
//!
//! This module provides the [`MqttDriver`] implementation of the [`PubSubClient`] trait.
//! It handles connection lifecycle, automatic reconnection, and message publishing.

use crate::PlantLinkError;
use crate::traits::{PubSubClient, PubSubMessage};
use futures::stream::BoxStream;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

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
/// # async fn example() -> Result<(), plantlink_core::PlantLinkError> {
/// let driver = MqttDriver::connect("plant-01", "localhost", 1883).await?;
/// driver.publish("sensors/temp", bytes::Bytes::from("hello")).await?;
/// # Ok(())
/// # }
/// ```
pub struct MqttDriver {
    client: AsyncClient,
    cancel: CancellationToken,
    #[allow(dead_code)]
    task_handle: JoinHandle<()>,
}

impl MqttDriver {
    /// Connects to an MQTT broker at the specified host and port.
    ///
    /// # Errors
    /// Returns a [`PlantLinkError::Connection`] if the driver fails to initialize.
    #[allow(clippy::unused_async)]
    #[tracing::instrument(skip(id, host, port), err)]
    pub async fn connect(id: &str, host: &str, port: u16) -> Result<Self, PlantLinkError> {
        let mut mqttoptions = MqttOptions::new(id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        let cancel = CancellationToken::new();
        let loop_cancel = cancel.clone();

        // ast-grep-ignore: raw-tokio-spawn
        let task_handle = tokio::spawn(async move {
            let mut backoff = Duration::from_secs(1);
            let max_backoff = Duration::from_secs(60);

            loop {
                tokio::select! {
                    () = loop_cancel.cancelled() => {
                        tracing::info!("MQTT event loop shutting down");
                        break;
                    }
                    poll_res = eventloop.poll() => {
                        match poll_res {
                            Ok(_event) => {
                                backoff = Duration::from_secs(1);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "MQTT event loop error: {}. Retrying in {:?}...",
                                    e,
                                    backoff
                                );
                                tokio::time::sleep(backoff).await;
                                backoff = (backoff * 2).min(max_backoff);
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            client,
            cancel,
            task_handle,
        })
    }

    /// Signals the background event loop to shut down.
    pub fn shutdown(&self) {
        self.cancel.cancel();
    }
}

impl Drop for MqttDriver {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

#[async_trait::async_trait]
impl PubSubClient for MqttDriver {
    #[tracing::instrument(skip(self, payload), err)]
    async fn publish(&self, topic: &str, payload: bytes::Bytes) -> Result<(), PlantLinkError> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload.to_vec())
            .await
            .map_err(|e| PlantLinkError::Publish(e.to_string()))?;
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    async fn subscribe(
        &self,
        _topic: &str,
    ) -> Result<BoxStream<'static, PubSubMessage>, PlantLinkError> {
        Err(PlantLinkError::NotImplemented("MQTT subscribe".into()))
    }
}
