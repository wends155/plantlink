//! Structured error types for the `PlantLink` core library.
//!
//! This module defines the [`PlantLinkError`] enum, which consolidates all
//! protocol and operational failures into a single, matchable type.

use std::error::Error as StdError;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct SimpleError(pub String);

/// The primary error type for all `plantlink-core` operations.
///
/// This enum is `#[non_exhaustive]` to allow for future protocol additions
/// without breaking downstream consumers.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PlantLinkError {
    /// Failed to establish a connection to a broker or device.
    #[error("connection failed: {0}")]
    Connection(#[source] Arc<dyn StdError + Send + Sync>),

    /// Failed to publish a message.
    #[error("publish failed: {0}")]
    Publish(#[source] Arc<dyn StdError + Send + Sync>),

    /// Failed to subscribe to a topic or subject.
    #[error("subscribe failed: {0}")]
    Subscribe(#[source] Arc<dyn StdError + Send + Sync>),

    /// An error occurred during a Modbus operation.
    #[error("modbus operation failed: {0}")]
    Modbus(#[source] Arc<dyn StdError + Send + Sync>),

    #[error("not implemented: {0}")]
    NotImplemented(#[source] Arc<dyn StdError + Send + Sync>),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{PlantLinkError, SimpleError};
    use std::sync::Arc;

    #[test]
    fn test_error_display() {
        let err = PlantLinkError::Connection(Arc::new(SimpleError("timeout".into())));
        assert_eq!(err.to_string(), "connection failed: timeout");

        let err = PlantLinkError::Publish(Arc::new(SimpleError("disconnected".into())));
        assert_eq!(err.to_string(), "publish failed: disconnected");

        let err = PlantLinkError::Subscribe(Arc::new(SimpleError("no access".into())));
        assert_eq!(err.to_string(), "subscribe failed: no access");

        let err = PlantLinkError::Modbus(Arc::new(SimpleError("crc error".into())));
        assert_eq!(err.to_string(), "modbus operation failed: crc error");

        let err = PlantLinkError::NotImplemented(Arc::new(SimpleError("MQTT subscribe".into())));
        assert_eq!(err.to_string(), "not implemented: MQTT subscribe");
    }

    #[test]
    fn test_error_converts_to_anyhow() {
        let err = PlantLinkError::Connection(Arc::new(SimpleError("failed".into())));
        let anyhow_err: anyhow::Error = err.into();
        assert_eq!(anyhow_err.to_string(), "connection failed: failed");
    }

    #[test]
    fn test_error_is_std_error() {
        use std::sync::Arc;
        fn is_std_error<T: std::error::Error>(_: &T) {}
        let err = PlantLinkError::Connection(Arc::new(super::SimpleError("failed".into())));
        is_std_error(&err);
    }
}
