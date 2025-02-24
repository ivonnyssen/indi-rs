use serde::{Deserialize, Serialize};

/// BLOB enable values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BLOBEnable {
    /// Never send BLOB data (default)
    Never,
    /// Send BLOB data along with other messages
    Also,
    /// Only send BLOB data
    Only,
}

/// Command to control whether setBLOBs should be sent to this channel from a given Device.
/// They can be turned off completely by setting Never (the default), allowed to be intermixed
/// with other INDI commands by setting Also or made the only command by setting Only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "enableBLOB")]
pub struct EnableBlob {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Name of BLOB Property, or all if absent
    #[serde(rename = "@name")]
    pub name: Option<String>,
    /// BLOB enable value
    #[serde(rename = "$text")]
    pub enable: BLOBEnable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_enable_with_name() {
        let enable = EnableBlob {
            device: "test_device".to_string(),
            name: Some("test_blob".to_string()),
            enable: BLOBEnable::Also,
        };

        assert_eq!(enable.device, "test_device");
        assert_eq!(enable.name, Some("test_blob".to_string()));
        assert_eq!(enable.enable, BLOBEnable::Also);
    }

    #[test]
    fn test_blob_enable_without_name() {
        let enable = EnableBlob {
            device: "test_device".to_string(),
            name: None,
            enable: BLOBEnable::Never,
        };

        assert_eq!(enable.device, "test_device");
        assert_eq!(enable.name, None);
        assert_eq!(enable.enable, BLOBEnable::Never);
    }
}
