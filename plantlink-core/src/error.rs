//! Structured error types for the PlantLink core library.
//!
//! This module defines the [`PlantLinkError`] enum, which consolidates all
//! protocol and operational failures into a single, matchable type.

use std::fmt::Debug;

/// The primary error type for all `plantlink-core` operations.
///
/// This enum is `#[non_exhaustive]` to allow for future protocol additions
/// without breaking downstream consumers.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PlantLinkError {
    /// Failed to establish a connection to a broker or device.
    #[error("connection failed: {0}")]
    Connection(String),

    /// Failed to publish a message.
    #[error("publish failed: {0}")]
    Publish(String),

    /// Failed to subscribe to a topic or subject.
    #[error("subscribe failed: {0}")]
    Subscribe(String),

    /// An error occurred during a Modbus operation.
    #[error("modbus operation failed: {0}")]
    Modbus(String),

    /// The requested feature or protocol capability is not yet implemented.
    #[error("not implemented: {0}")]
    NotImplemented(String),
}
