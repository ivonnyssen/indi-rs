use serde::{Deserialize, Serialize};

/// BLOB enable values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BLOBEnable {
    /// Never send BLOB data
    Never,
    /// Send BLOB data along with other messages
    Also,
    /// Only send BLOB data
    Only,
}

/// Enable BLOB message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableBlob {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// BLOB enable value
    #[serde(rename = "$text")]
    pub enable: BLOBEnable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_enable_serialization() {
        let enable = EnableBlob {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            enable: BLOBEnable::Also,
        };

        assert_eq!(enable.device, "test_device");
        assert_eq!(enable.name, "test_name");
        assert_eq!(enable.enable, BLOBEnable::Also);
    }
}
