use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;

#[derive(Debug, Clone)]
pub struct PubSubMessage {
    pub topic: String,
    pub payload: Bytes,
}

#[async_trait]
#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
pub trait PubSubClient: Send + Sync + 'static {
    async fn publish(&self, topic: &str, payload: Bytes) -> Result<()>;
    async fn subscribe(&self, topic: &str) -> Result<BoxStream<'static, PubSubMessage>>;
}

#[async_trait]
#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
pub trait ModbusClient: Send + Sync + 'static {
    async fn read_coils(&mut self, addr: u16, cnt: u16) -> Result<Vec<bool>>;
}
