use serde::{Deserialize, Serialize};

/// Properties request message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "getProperties")]
pub struct GetProperties {
    /// Version of the INDI protocol
    #[serde(rename = "@version")]
    pub version: String,
    /// Device to get properties for
    #[serde(rename = "@device", skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    /// Name of the property to get
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Set property message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "setProperty")]
pub struct SetProperty {
    /// Raw XML content of the message
    #[serde(rename = "$value")]
    pub content: String,
}

/// General message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "message")]
pub struct Message {
    /// Raw XML content of the message
    #[serde(rename = "$value")]
    pub content: String,
}

/// Delete property message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "delProperty")]
pub struct DelProperty {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
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
