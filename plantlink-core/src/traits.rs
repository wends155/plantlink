//! # Protocol Traits
//!
//! This module defines the core trait interfaces for protocol drivers.
//! All drivers returned by the factory or registered in the runtime must
//! implement these traits to be compatible with the flow engine.

use crate::PlantLinkError;
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;

/// A message received from a `PubSub` subject.
#[derive(Debug, Clone)]
pub struct PubSubMessage {
    pub topic: String,
    pub payload: Bytes,
}

/// Interface for `PubSub` protocol drivers (MQTT, NATS).
#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
#[async_trait]
pub trait PubSubClient: Send + Sync + 'static {
    /// Publishes a payload to the specified topic.
    async fn publish(&self, topic: &str, payload: Bytes) -> Result<(), PlantLinkError>;

    /// Subscribes to the specified topic and returns a stream of messages.
    async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<BoxStream<'static, PubSubMessage>, PlantLinkError>;
}

/// Interface for Modbus protocol drivers (TCP, RTU).
#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
#[async_trait]
pub trait ModbusClient: Send + Sync + 'static {
    /// Reads the specified number of coils starting from the given address.
    async fn read_coils(&self, addr: u16, cnt: u16) -> Result<Vec<bool>, PlantLinkError>;
}
