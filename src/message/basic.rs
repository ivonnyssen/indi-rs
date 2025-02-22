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
