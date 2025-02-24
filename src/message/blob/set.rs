use crate::prelude::PropertyState;
use crate::timestamp::INDITimestamp;
use serde::{Deserialize, Serialize};
use super::one::OneBLOB;

/// BLOB vector that follows the INDI protocol DTD specification.
/// Used to report a BLOB's current value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "setBLOBVector")]
pub struct SetBLOBVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Name of the property
    #[serde(rename = "@name")]
    pub name: String,
    /// State of the property
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Timestamp
    #[serde(rename = "@timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<INDITimestamp>,
    /// Message
    #[serde(rename = "@message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// BLOB elements
    #[serde(rename = "oneBLOB")]
    pub blobs: Vec<OneBLOB>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_blob_vector() {
        let test_timestamp = "2024-01-01T00:00:00".parse().unwrap();
        let blob = OneBLOB {
            name: "test_blob".to_string(),
            size: 100,
            format: ".fits".to_string(),
            data: "base64encodeddata".to_string(),
        };
        
        let blob_vector = SetBLOBVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            state: PropertyState::Ok,
            timestamp: Some(test_timestamp),
            message: Some("test_message".to_string()),
            blobs: vec![blob],
        };

        assert_eq!(blob_vector.device, "test_device");
        assert_eq!(blob_vector.name, "test_name");
        assert_eq!(blob_vector.state, PropertyState::Ok);
        assert_eq!(blob_vector.timestamp.unwrap().to_string(), "2024-01-01T00:00:00");
        assert_eq!(blob_vector.message, Some("test_message".to_string()));
        assert_eq!(blob_vector.blobs.len(), 1);
        
        let blob = &blob_vector.blobs[0];
        assert_eq!(blob.name, "test_blob");
        assert_eq!(blob.size, 100);
        assert_eq!(blob.format, ".fits");
        assert_eq!(blob.data, "base64encodeddata");
    }
}
