use crate::prelude::PropertyPerm;
use crate::property::PropertyState;
use crate::timestamp::INDITimestamp;
use crate::message::common::vector::INDIVector;
use serde::{Deserialize, Serialize};

/// Number element in a number vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefNumber {
    /// Number name
    #[serde(rename = "@name")]
    pub name: String,
    /// Number label
    #[serde(rename = "@label")]
    pub label: Option<String>,
    /// Number format
    #[serde(rename = "@format")]
    pub format: String,
    /// Number minimum value
    #[serde(rename = "@min")]
    pub min: f64,
    /// Number maximum value
    #[serde(rename = "@max")]
    pub max: f64,
    /// Number step value
    #[serde(rename = "@step")]
    pub step: f64,
    /// Number value
    #[serde(rename = "$text")]
    pub value: f64,
}

impl DefNumber {
    /// Create a new DefNumber
    pub fn new(name: String, label: Option<String>, format: String, min: f64, max: f64, step: f64, value: f64) -> Self {
        Self {
            name,
            label,
            format,
            min,
            max,
            step,
            value,
        }
    }
}

/// Number vector definition that follows the INDI protocol DTD specification.
/// 
/// Uses the same string handling approach as [`crate::message::text::DefTextVector`] - see its documentation
/// for details about the string ownership design.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defNumberVector")]
pub struct DefNumberVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property label
    #[serde(rename = "@label")]
    pub label: Option<String>,
    /// Property group
    #[serde(rename = "@group")]
    pub group: Option<String>,
    /// Property state
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Property permission
    #[serde(rename = "@perm")]
    pub perm: PropertyPerm,
    /// Property timeout
    #[serde(rename = "@timeout")]
    pub timeout: Option<f64>,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<INDITimestamp>,
    /// Optional message
    #[serde(rename = "@message")]
    pub message: Option<String>,
    /// Number elements
    #[serde(rename = "defNumber")]
    pub numbers: Vec<DefNumber>,
}

impl INDIVector for DefNumberVector {
    type Element = DefNumber;

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
        &self.numbers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_vector_trait_implementation() {
        let test_timestamp = "2024-01-01T00:00:00".parse().unwrap();
        let number_vector = DefNumberVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: Some("test_label".to_string()),
            group: Some("test_group".to_string()),
            state: PropertyState::Idle,
            perm: PropertyPerm::Rw,
            timeout: Some(10.0),
            timestamp: Some(test_timestamp),
            message: Some("test_message".to_string()),
            numbers: vec![],
        };

        // Test trait methods
        assert_eq!(number_vector.device(), "test_device");
        assert_eq!(number_vector.name(), "test_name");
        assert_eq!(number_vector.label(), Some("test_label"));
        assert_eq!(number_vector.group(), Some("test_group"));
        assert_eq!(number_vector.state(), PropertyState::Idle);
        assert_eq!(number_vector.perm(), PropertyPerm::Rw);
        assert_eq!(number_vector.timeout(), Some(10.0));
        assert_eq!(number_vector.message(), Some("test_message"));
        assert!(number_vector.elements().is_empty());
        assert_eq!(number_vector.timestamp().unwrap().to_string(), "2024-01-01T00:00:00");
    }

    #[test]
    fn test_def_number_creation() {
        let number = DefNumber::new(
            "test".to_string(),
            Some("Test".to_string()),
            "%.2f".to_string(),
            0.0,
            100.0,
            1.0,
            50.0,
        );

        assert_eq!(number.name, "test");
        assert_eq!(number.label, Some("Test".to_string()));
        assert_eq!(number.format, "%.2f");
        assert_eq!(number.min, 0.0);
        assert_eq!(number.max, 100.0);
        assert_eq!(number.step, 1.0);
        assert_eq!(number.value, 50.0);
    }
}
