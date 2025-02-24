use crate::property::PropertyState;
use crate::timestamp::INDITimestamp;
use crate::message::common::vector::INDIVector;
use serde::{Deserialize, Serialize};

/// Light element in a light vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefLight {
    /// Light name
    #[serde(rename = "@name")]
    pub name: String,
    /// Light label
    #[serde(rename = "@label")]
    pub label: Option<String>,
    /// Light state
    #[serde(rename = "$text")]
    pub state: PropertyState,
}

/// Light vector definition that follows the INDI protocol DTD specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defLightVector")]
pub struct DefLightVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Name of the property
    #[serde(rename = "@name")]
    pub name: String,
    /// Label of the property
    #[serde(rename = "@label")]
    pub label: Option<String>,
    /// Group of the property
    #[serde(rename = "@group")]
    pub group: Option<String>,
    /// State of the property
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<INDITimestamp>,
    /// Message
    #[serde(rename = "@message")]
    pub message: Option<String>,
    /// Light elements
    #[serde(rename = "defLight")]
    pub lights: Vec<DefLight>,
}

impl INDIVector for DefLightVector {
    type Element = DefLight;

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

    fn perm(&self) -> crate::prelude::PropertyPerm {
        // Light vectors are read-only by definition
        crate::prelude::PropertyPerm::Ro
    }

    fn timeout(&self) -> Option<f64> {
        None
    }

    fn timestamp(&self) -> Option<&INDITimestamp> {
        self.timestamp.as_ref()
    }

    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn elements(&self) -> &[Self::Element] {
        &self.lights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_vector_trait_implementation() {
        let test_timestamp = "2024-01-01T00:00:00".parse().unwrap();
        let light_vector = DefLightVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: Some("test_label".to_string()),
            group: Some("test_group".to_string()),
            state: PropertyState::Idle,
            timestamp: Some(test_timestamp),
            message: Some("test_message".to_string()),
            lights: vec![],
        };

        // Test trait methods
        assert_eq!(light_vector.device(), "test_device");
        assert_eq!(light_vector.name(), "test_name");
        assert_eq!(light_vector.label(), Some("test_label"));
        assert_eq!(light_vector.group(), Some("test_group"));
        assert_eq!(light_vector.state(), PropertyState::Idle);
        assert_eq!(light_vector.message(), Some("test_message"));
        assert!(light_vector.elements().is_empty());
        assert_eq!(light_vector.timestamp().unwrap().to_string(), "2024-01-01T00:00:00");
    }
}
