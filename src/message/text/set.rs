use crate::message::common::PropertyState;
use crate::timestamp::INDITimestamp;
use super::one::OneText;
use serde::{Deserialize, Serialize};

/// Set text vector command
/// 
/// According to the INDI protocol specification, this command sends a new set of values
/// for a Text vector, with optional new timeout, state, timestamp, and message.
/// The state attribute indicates whether the values are being set (Busy), have been
/// set (Ok), or the attempt to set them failed (Alert).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "setTextVector")]
pub struct SetTextVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property state, no change if absent
    #[serde(rename = "@state")]
    pub state: Option<PropertyState>,
    /// Worse-case time to affect a change
    #[serde(rename = "@timeout")]
    pub timeout: Option<f64>,
    /// Moment when these data were valid
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<INDITimestamp>,
    /// Commentary
    #[serde(rename = "@message")]
    pub message: Option<String>,
    /// Text elements
    #[serde(rename = "oneText")]
    pub texts: Vec<OneText>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_set_text_vector() {
        let vector = SetTextVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            state: None,
            timeout: None,
            timestamp: None,
            message: None,
            texts: vec![OneText {
                name: "text1".to_string(),
                value: "value1".to_string(),
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert!(vector.state.is_none());
        assert!(vector.timeout.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
        assert_eq!(vector.texts.len(), 1);
        assert_eq!(&vector.texts[0].name, "text1");
        assert_eq!(&vector.texts[0].value, "value1");
    }

    #[test]
    fn test_set_text_vector_with_all_fields() {
        let timestamp = INDITimestamp::from_str("2024-01-01T12:34:56.7").unwrap();
        let vector = SetTextVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            state: Some(PropertyState::Busy),
            timeout: Some(1000.0),
            timestamp: Some(timestamp.clone()),
            message: Some("Processing".to_string()),
            texts: vec![OneText {
                name: "text1".to_string(),
                value: "value1".to_string(),
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.state.unwrap(), PropertyState::Busy);
        assert_eq!(vector.timeout.unwrap(), 1000.0);
        assert_eq!(vector.timestamp.unwrap().to_string(), "2024-01-01T12:34:56.7");
        assert_eq!(vector.message.unwrap(), "Processing");
        assert_eq!(vector.texts.len(), 1);
        assert_eq!(&vector.texts[0].name, "text1");
        assert_eq!(&vector.texts[0].value, "value1");
    }
}
