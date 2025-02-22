use crate::property::{PropertyState, PropertyPerm};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

/// One BLOB element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneBLOB {
    /// BLOB name
    #[serde(rename = "@name")]
    pub name: String,
    /// BLOB size
    #[serde(rename = "@size")]
    pub size: usize,
    /// BLOB format as a file suffix (e.g., ".fits", ".z", ".fits.z")
    #[serde(rename = "@format")]
    pub format: String,
    /// BLOB data (base64 encoded)
    #[serde(rename = "$text")]
    pub data: String,
}

impl OneBLOB {
    /// Create a new BLOB element
    /// 
    /// # Arguments
    /// * `name` - Name of the BLOB element
    /// * `format` - Format as a file suffix (e.g., ".fits", ".z", ".fits.z")
    /// * `data` - Raw binary data to be base64 encoded
    pub fn new(name: String, format: String, data: Vec<u8>) -> Self {
        Self {
            name,
            size: data.len(),
            format,
            data: general_purpose::STANDARD.encode(&data),
        }
    }

    /// Get the decoded BLOB data
    pub fn get_data(&self) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(&self.data)
    }
}

/// BLOB enable values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BLOBEnable {
    /// Never send setBLOB messages (default)
    Never,
    /// Allow setBLOB messages to be intermixed with other commands
    Also,
    /// Only send setBLOB messages
    Only,
}

/// Enable BLOB message - controls whether setBLOBs should be sent to this channel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "enableBLOB")]
pub struct EnableBlob {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name (optional)
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// BLOB enable value
    #[serde(rename = "$text")]
    pub value: BLOBEnable,
}

/// Set BLOB vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "setBLOBVector")]
pub struct SetBLOBVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property state
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    /// BLOB elements
    #[serde(rename = "oneBLOB")]
    pub blobs: Vec<OneBLOB>,
}

/// New BLOB vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "newBLOBVector")]
pub struct NewBLOBVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    /// BLOB elements
    #[serde(rename = "oneBLOB")]
    pub blobs: Vec<OneBLOB>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageType;
    use crate::property::PropertyState;

    #[test]
    fn test_enable_blob_message() {
        let xml = r#"<enableBLOB device="CCD" name="CCD1">Never</enableBLOB>"#;
        let parsed: MessageType = xml.parse().unwrap();
        match parsed {
            MessageType::EnableBLOB(v) => {
                assert_eq!(v.device, "CCD");
                assert_eq!(v.name, Some("CCD1".to_string()));
                assert_eq!(v.value, BLOBEnable::Never);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_enable_blob() {
        let xml = r#"<enableBLOB device="CCD">Also</enableBLOB>"#;
        let parsed: MessageType = xml.parse().unwrap();
        match parsed {
            MessageType::EnableBLOB(v) => {
                assert_eq!(v.device, "CCD");
                assert_eq!(v.name, None);
                assert_eq!(v.value, BLOBEnable::Also);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_blob_message_serialization() {
        let test_data = vec![1, 2, 3, 4, 5];
        let blob = OneBLOB::new(
            "test_blob".to_string(),
            ".fits".to_string(),
            test_data.clone(),
        );

        let blob_vector = SetBLOBVector {
            device: "TestDevice".to_string(),
            name: "TestBLOB".to_string(),
            state: PropertyState::Ok,
            timestamp: "2024-02-21T19:30:00".to_string(),
            blobs: vec![blob],
        };

        let xml = quick_xml::se::to_string(&blob_vector).unwrap();
        assert!(xml.contains("setBLOBVector"));
        assert!(xml.contains("device=\"TestDevice\""));
        assert!(xml.contains("name=\"TestBLOB\""));
        assert!(xml.contains("state=\"Ok\""));
        assert!(xml.contains("oneBLOB"));
        assert!(xml.contains("format=\".fits\""));

        let decoded: SetBLOBVector = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(decoded.device, "TestDevice");
        assert_eq!(decoded.blobs[0].get_data().unwrap(), test_data);
    }

    #[test]
    fn test_blob_encoding_decoding() {
        let test_data = vec![1, 2, 3, 4, 5];
        let blob = OneBLOB::new(
            "test_blob".to_string(),
            ".fits".to_string(),
            test_data.clone(),
        );

        assert_eq!(blob.size, test_data.len());
        assert_eq!(blob.get_data().unwrap(), test_data);
    }
}
