use serde::{Deserialize, Serialize};
use crate::timestamp::INDITimestamp;
use super::one::OneNumber;

/// New number vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "newNumberVector")]
pub struct NewNumberVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<INDITimestamp>,
    /// Number elements
    #[serde(rename = "oneNumber")]
    pub numbers: Vec<OneNumber>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_new_number_vector() {
        let vector = NewNumberVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            timestamp: None,
            numbers: vec![OneNumber {
                name: "number1".to_string(),
                value: 42.0,
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert!(vector.timestamp.is_none());
        assert_eq!(vector.numbers.len(), 1);
        assert_eq!(vector.numbers[0].name, "number1");
        assert_eq!(vector.numbers[0].value, 42.0);
    }

    #[test]
    fn test_new_number_vector_with_timestamp() {
        let timestamp = INDITimestamp::from_str("2024-01-01T12:34:56.7").unwrap();
        let vector = NewNumberVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            timestamp: Some(timestamp.clone()),
            numbers: vec![OneNumber {
                name: "number1".to_string(),
                value: 42.0,
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.timestamp.unwrap().to_string(), "2024-01-01T12:34:56.7");
        assert_eq!(vector.numbers.len(), 1);
    }
}
