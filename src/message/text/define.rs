use crate::prelude::PropertyPerm;
use crate::message::common::PropertyState;
use crate::timestamp::INDITimestamp;
use crate::message::common::vector::INDIVector;
use serde::{Deserialize, Serialize};

/// Define one member of a text vector
/// 
/// This struct represents a single text element within a text vector as defined by the INDI protocol.
/// Each text element has a required name, an optional label (defaulting to name if absent),
/// and a text value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defText")]
pub struct DefText {
    /// Name of this text element
    #[serde(rename = "@name")]
    pub name: String,
    /// GUI label, or use name by default
    #[serde(rename = "@label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Text value
    #[serde(rename = "$text")]
    pub value: String,
}

/// Define a property that holds one or more text elements.
/// 
/// # String Handling
/// 
/// This struct uses owned `String` types for all string fields. This is a deliberate design choice:
/// 
/// - Owned `String` is required for XML serialization/deserialization
/// - It ensures the data lives as long as the struct itself
/// - It provides clear ownership semantics for the XML attributes
/// 
/// While this struct stores owned strings, it implements the `INDIVector` trait which provides
/// efficient read-only access through `&str` references. This is achieved by using `as_deref()`
/// in the trait implementation to convert from `&String` to `&str`.
/// 
/// This approach combines the benefits of:
/// - Owned data for XML serialization (`String`)
/// - Efficient read access through the trait interface (`&str`)
/// 
/// According to the INDI protocol specification, this vector can hold one or more text elements
/// and includes various attributes for device identification, GUI presentation, state tracking,
/// and client control.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defTextVector")]
pub struct DefTextVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// GUI label, use name by default
    #[serde(rename = "@label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Property group membership, blank by default
    #[serde(rename = "@group", skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Current state of Property
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Ostensible Client controlability
    #[serde(rename = "@perm")]
    pub perm: PropertyPerm,
    /// Worse-case time to affect, 0 default, N/A for ro
    #[serde(rename = "@timeout", skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
    /// Moment when these data were valid
    #[serde(rename = "@timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<INDITimestamp>,
    /// Commentary
    #[serde(rename = "@message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Text elements
    #[serde(rename = "defText")]
    pub texts: Vec<DefText>,
}

impl INDIVector for DefTextVector {
    type Element = DefText;

    fn device(&self) -> &str {
        &self.device
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    fn state(&self) -> PropertyState {
        self.state
    }

    fn perm(&self) -> PropertyPerm {
        self.perm
    }

    fn timeout(&self) -> Option<f64> {
        self.timeout
    }

    fn timestamp(&self) -> Option<&INDITimestamp> {
        self.timestamp.as_ref()
    }

    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn elements(&self) -> &[Self::Element] {
        &self.texts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_def_text() {
        let text = DefText {
            name: "test_text".to_string(),
            label: Some("Test Text".to_string()),
            value: "test value".to_string(),
        };

        assert_eq!(text.name, "test_text");
        assert_eq!(text.label.unwrap(), "Test Text");
        assert_eq!(text.value, "test value");
    }

    #[test]
    fn test_def_text_vector() {
        let vector = DefTextVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: Some("Test Label".to_string()),
            group: Some("Test Group".to_string()),
            state: PropertyState::Ok,
            perm: PropertyPerm::Rw,
            timeout: None,
            timestamp: None,
            message: None,
            texts: vec![DefText {
                name: "text1".to_string(),
                label: Some("Text 1".to_string()),
                value: "test value".to_string(),
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.label.unwrap(), "Test Label");
        assert_eq!(vector.group.unwrap(), "Test Group");
        assert_eq!(vector.state, PropertyState::Ok);
        assert_eq!(vector.perm, PropertyPerm::Rw);
        assert!(vector.timeout.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
        assert_eq!(vector.texts.len(), 1);
        assert_eq!(&vector.texts[0].name, "text1");
        assert_eq!(vector.texts[0].label.as_ref().unwrap(), "Text 1");
        assert_eq!(&vector.texts[0].value, "test value");
    }

    #[test]
    fn test_def_text_vector_with_all_fields() {
        let timestamp = INDITimestamp::from_str("2024-01-01T12:34:56.7").unwrap();
        let vector = DefTextVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: Some("Test Label".to_string()),
            group: Some("Test Group".to_string()),
            state: PropertyState::Ok,
            perm: PropertyPerm::Rw,
            timeout: Some(1000.0),
            timestamp: Some(timestamp.clone()),
            message: Some("Test message".to_string()),
            texts: vec![DefText {
                name: "text1".to_string(),
                label: Some("Text 1".to_string()),
                value: "test value".to_string(),
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.label.unwrap(), "Test Label");
        assert_eq!(vector.group.unwrap(), "Test Group");
        assert_eq!(vector.state, PropertyState::Ok);
        assert_eq!(vector.perm, PropertyPerm::Rw);
        assert_eq!(vector.timeout.unwrap(), 1000.0);
        assert_eq!(vector.timestamp.unwrap().to_string(), "2024-01-01T12:34:56.7");
        assert_eq!(vector.message.unwrap(), "Test message");
        assert_eq!(vector.texts.len(), 1);
        assert_eq!(&vector.texts[0].name, "text1");
        assert_eq!(vector.texts[0].label.as_ref().unwrap(), "Text 1");
        assert_eq!(&vector.texts[0].value, "test value");
    }
}
