use serde::{Deserialize, Serialize};

// ... (existing imports)
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    // Json must be last to avoid aggressively capturing other types
    Json(serde_json::Value),
    Null,
}

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataValue::Boolean(v) => write!(f, "{}", v),
            DataValue::Integer(v) => write!(f, "{}", v),
            DataValue::Float(v) => write!(f, "{}", v),
            DataValue::String(v) => write!(f, "{}", v),
            DataValue::Bytes(v) => write!(f, "{:?}", v),
            DataValue::Json(v) => write!(f, "{}", v),
            DataValue::Null => write!(f, "null"),
        }
    }
}

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
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
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
}
