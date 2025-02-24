use crate::timestamp::INDITimestamp;
use serde::{Deserialize, Serialize};
use super::one::OneBLOB;

/// BLOB vector that follows the INDI protocol DTD specification.
/// Used to send new target BLOBs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "newBLOBVector")]
pub struct NewBLOBVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Name of the property
    #[serde(rename = "@name")]
    pub name: String,
    /// Timestamp
    #[serde(rename = "@timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<INDITimestamp>,
    /// BLOB elements
    #[serde(rename = "oneBLOB")]
    pub blobs: Vec<OneBLOB>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blob_vector() {
        let test_timestamp = "2024-01-01T00:00:00".parse().unwrap();
        let blob = OneBLOB {
            name: "test_blob".to_string(),
            size: 100,
            format: ".fits".to_string(),
            data: "base64encodeddata".to_string(),
        };
        
        let blob_vector = NewBLOBVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            timestamp: Some(test_timestamp),
            blobs: vec![blob],
        };

        assert_eq!(blob_vector.device, "test_device");
        assert_eq!(blob_vector.name, "test_name");
        assert_eq!(blob_vector.timestamp.unwrap().to_string(), "2024-01-01T00:00:00");
        assert_eq!(blob_vector.blobs.len(), 1);
        
        let blob = &blob_vector.blobs[0];
        assert_eq!(blob.name, "test_blob");
        assert_eq!(blob.size, 100);
        assert_eq!(blob.format, ".fits");
        assert_eq!(blob.data, "base64encodeddata");
    }
}
