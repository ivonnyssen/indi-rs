use crate::prelude::PropertyPerm;
use crate::message::common::PropertyState;
use crate::timestamp::INDITimestamp;
use crate::message::common::vector::INDIVector;
use serde::{Deserialize, Serialize};

/// BLOB element in a BLOB vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefBLOB {
    /// BLOB name
    #[serde(rename = "@name")]
    pub name: String,
    /// BLOB label
    #[serde(rename = "@label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// BLOB vector definition that follows the INDI protocol DTD specification.
/// 
/// Uses the same string handling approach as [`crate::message::text::DefTextVector`] - see its documentation
/// for details about the string ownership design.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defBLOBVector")]
pub struct DefBLOBVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Name of the property
    #[serde(rename = "@name")]
    pub name: String,
    /// Label of the property
    #[serde(rename = "@label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Group of the property
    #[serde(rename = "@group", skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// State of the property
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Permission of the property
    #[serde(rename = "@perm")]
    pub perm: PropertyPerm,
    /// Timeout in seconds
    #[serde(rename = "@timeout", skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
    /// Message
    #[serde(rename = "@message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Timestamp
    #[serde(rename = "@timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<INDITimestamp>,
    /// BLOB elements
    #[serde(rename = "defBLOB")]
    pub blobs: Vec<DefBLOB>,
}

impl INDIVector for DefBLOBVector {
    type Element = DefBLOB;

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
        &self.blobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_vector_trait_implementation() {
        let test_timestamp = "2024-01-01T00:00:00".parse().unwrap();
        let blob_vector = DefBLOBVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: Some("test_label".to_string()),
            group: Some("test_group".to_string()),
            state: PropertyState::Idle,
            perm: PropertyPerm::Rw,
            timeout: Some(10.0),
            timestamp: Some(test_timestamp),
            message: Some("test_message".to_string()),
            blobs: vec![],
        };

        // Test trait methods
        assert_eq!(blob_vector.device(), "test_device");
        assert_eq!(blob_vector.name(), "test_name");
        assert_eq!(blob_vector.label(), Some("test_label"));
        assert_eq!(blob_vector.group(), Some("test_group"));
        assert_eq!(blob_vector.state(), PropertyState::Idle);
        assert_eq!(blob_vector.perm(), PropertyPerm::Rw);
        assert_eq!(blob_vector.timeout(), Some(10.0));
        assert_eq!(blob_vector.message(), Some("test_message"));
        assert!(blob_vector.elements().is_empty());
        assert_eq!(blob_vector.timestamp().unwrap().to_string(), "2024-01-01T00:00:00");
    }
}
