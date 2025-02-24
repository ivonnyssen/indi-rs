use serde::{Deserialize, Serialize};
use crate::message::common::PropertyState;
use crate::timestamp::INDITimestamp;
use super::one::OneNumber;

/// Set number vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "setNumberVector")]
pub struct SetNumberVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property state
    #[serde(rename = "@state")]
    pub state: Option<PropertyState>,
    /// Property timeout - worse-case time to affect a change
    #[serde(rename = "@timeout")]
    pub timeout: Option<f64>,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<INDITimestamp>,
    /// Optional message/commentary
    #[serde(rename = "@message")]
    pub message: Option<String>,
    /// Number elements
    #[serde(rename = "oneNumber")]
    pub numbers: Vec<OneNumber>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_set_number_vector() {
        let vector = SetNumberVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            state: None,
            timeout: None,
            timestamp: None,
            message: None,
            numbers: vec![OneNumber {
                name: "number1".to_string(),
                value: 42.0,
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert!(vector.state.is_none());
        assert!(vector.timeout.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
        assert_eq!(vector.numbers.len(), 1);
        assert_eq!(vector.numbers[0].name, "number1");
        assert_eq!(vector.numbers[0].value, 42.0);
    }

    #[test]
    fn test_set_number_vector_with_all_fields() {
        let timestamp = INDITimestamp::from_str("2024-01-01T12:34:56.7").unwrap();
        let vector = SetNumberVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            state: Some(PropertyState::Ok),
            timeout: Some(1000.0),
            timestamp: Some(timestamp.clone()),
            message: Some("Test message".to_string()),
            numbers: vec![OneNumber {
                name: "number1".to_string(),
                value: 42.0,
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.state.unwrap(), PropertyState::Ok);
        assert_eq!(vector.timeout.unwrap(), 1000.0);
        assert_eq!(vector.timestamp.unwrap().to_string(), "2024-01-01T12:34:56.7");
        assert_eq!(vector.message.unwrap(), "Test message");
        assert_eq!(vector.numbers.len(), 1);
    }
}
