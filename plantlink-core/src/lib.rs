//! # `PlantLink` Core
//!
//! Shared data types and protocol drivers for the `PlantLink`
//! flow-based `IoT` runtime.
//!
//! This crate provides:
//! - [`DataValue`] — A universal value type for node payloads.
//! - [`MessagePayload`] — The standard message envelope passed between nodes.
//! - Protocol drivers: [`mqtt::MqttDriver`], [`nats::NatsDriver`], [`modbus::ModbusTcpClient`].

use serde::{Deserialize, Serialize};

// ... (existing imports)
use std::fmt;

/// Universal value type for all node payloads.
///
/// Uses `#[serde(untagged)]` deserialization, so variant order matters.
/// `Null` must precede `Json` to prevent `null` from being captured as `Json(Value::Null)`.
///
/// # Examples
///
/// ```
/// use plantlink_core::DataValue;
///
/// // Primitive types deserialize to their specific variants
/// let val: DataValue = serde_json::from_str("42").unwrap();
/// assert!(matches!(val, DataValue::Integer(42)));
///
/// // JSON objects are captured by the Json variant
/// let val: DataValue = serde_json::from_str(r#"{"key": "value"}"#).unwrap();
/// assert!(matches!(val, DataValue::Json(_)));
///
/// // Display implementation
/// assert_eq!(DataValue::Boolean(true).to_string(), "true");
/// assert_eq!(DataValue::Null.to_string(), "null");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Null,
    // Json must be last to avoid aggressively capturing other types
    Json(serde_json::Value),
}

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataValue::Boolean(v) => write!(f, "{v}"),
            DataValue::Integer(v) => write!(f, "{v}"),
            DataValue::Float(v) => write!(f, "{v}"),
            DataValue::String(v) => write!(f, "{v}"),
            DataValue::Bytes(v) => write!(f, "{v:?}"),
            DataValue::Json(v) => write!(f, "{v}"),
            DataValue::Null => write!(f, "null"),
        }
    }
}

/// The standard message envelope passed between nodes.
///
/// Each message has a unique `id`, a `timestamp` in milliseconds since epoch,
/// and an arbitrary `meta` field for additional context.
///
/// # Examples
///
/// ```
/// use plantlink_core::{MessagePayload, DataValue};
///
/// let msg = MessagePayload::default();
/// assert!(matches!(msg.payload, DataValue::Null));
/// assert!(!msg.id.is_empty());
///
/// // Round-trip serialization
/// let json = serde_json::to_string(&msg).unwrap();
/// let deserialized: MessagePayload = serde_json::from_str(&json).unwrap();
/// assert_eq!(msg.id, deserialized.id);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub id: String,
    pub topic: Option<String>,
    pub payload: DataValue,
    pub timestamp: u64,
    pub meta: serde_json::Value,
}

impl Default for MessagePayload {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            topic: None,
            payload: DataValue::Null,
            timestamp: chrono::Utc::now().timestamp_millis().cast_unsigned(),
            meta: serde_json::json!({}),
        }
    }
}

pub mod modbus;
pub mod mqtt;
pub mod nats;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_payload_serialization() {
        let payload = MessagePayload::default();
        let json = serde_json::to_string(&payload).expect("Serialization failed");
        assert!(json.contains(&payload.id));
    }

    #[test]
    fn test_data_value_boolean_not_captured_as_json() {
        let json_str = "true";
        let val: DataValue = serde_json::from_str(json_str).unwrap();
        assert!(matches!(val, DataValue::Boolean(true)));
    }

    #[test]
    fn test_data_value_integer_not_captured_as_json() {
        let json_str = "42";
        let val: DataValue = serde_json::from_str(json_str).unwrap();
        assert!(matches!(val, DataValue::Integer(42)));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_data_value_float_not_captured_as_json() {
        let json_str = "3.14";
        let val: DataValue = serde_json::from_str(json_str).unwrap();
        assert!(matches!(val, DataValue::Float(f) if (f - 3.14).abs() < f64::EPSILON));
    }
    #[test]
    fn test_data_value_string_not_captured_as_json() {
        let json_str = "\"hello\"";
        let val: DataValue = serde_json::from_str(json_str).unwrap();
        assert!(matches!(val, DataValue::String(ref s) if s == "hello"));
    }

    #[test]
    fn test_data_value_null_deserialization() {
        let json_str = "null";
        let val: DataValue = serde_json::from_str(json_str).unwrap();
        assert!(matches!(val, DataValue::Null));
    }

    #[test]
    fn test_data_value_json_object() {
        let json_str = r#"{"key": "value"}"#;
        let val: DataValue = serde_json::from_str(json_str).unwrap();
        assert!(matches!(val, DataValue::Json(_)));
    }

    #[test]
    fn test_payload_roundtrip_deserialization() {
        let original = MessagePayload::default();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: MessagePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_data_value_display() {
        assert_eq!(DataValue::Boolean(true).to_string(), "true");
        assert_eq!(DataValue::Integer(42).to_string(), "42");
        assert_eq!(DataValue::Null.to_string(), "null");
        assert_eq!(DataValue::String("hi".into()).to_string(), "hi");
    }
}
