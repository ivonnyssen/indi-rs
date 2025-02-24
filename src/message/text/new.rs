use crate::timestamp::INDITimestamp;
use super::one::OneText;
use serde::{Deserialize, Serialize};

/// New text vector command
/// 
/// According to the INDI protocol specification, this command informs the Device
/// of new target values for a Property. After sending, the Client must set its
/// local state for the Property to Busy, leaving it up to the Device to change
/// it when it sees fit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "newTextVector")]
pub struct NewTextVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Moment when this message was generated
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<INDITimestamp>,
    /// Text elements
    #[serde(rename = "oneText")]
    pub texts: Vec<OneText>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_new_text_vector() {
        let vector = NewTextVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            timestamp: None,
            texts: vec![OneText {
                name: "text1".to_string(),
                value: "value1".to_string(),
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert!(vector.timestamp.is_none());
        assert_eq!(vector.texts.len(), 1);
        assert_eq!(&vector.texts[0].name, "text1");
        assert_eq!(&vector.texts[0].value, "value1");
    }

    #[test]
    fn test_new_text_vector_with_timestamp() {
        let timestamp = INDITimestamp::from_str("2024-01-01T12:34:56.7").unwrap();
        let vector = NewTextVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            timestamp: Some(timestamp.clone()),
            texts: vec![OneText {
                name: "text1".to_string(),
                value: "value1".to_string(),
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.timestamp.unwrap().to_string(), "2024-01-01T12:34:56.7");
        assert_eq!(vector.texts.len(), 1);
        assert_eq!(&vector.texts[0].name, "text1");
        assert_eq!(&vector.texts[0].value, "value1");
    }
}
