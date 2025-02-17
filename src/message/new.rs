use crate::property::{PropertyState, SwitchState};
use serde::{Deserialize, Serialize};

/// Switch element in a new switch vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "oneSwitch")]
pub struct OneSwitch {
    /// Switch name
    #[serde(rename = "@name")]
    pub name: String,
    /// Switch state
    #[serde(rename = "$text")]
    pub value: SwitchState,
}

/// New switch vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "newSwitchVector")]
pub struct NewSwitchVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    /// Switch elements
    #[serde(rename = "oneSwitch")]
    pub elements: Vec<OneSwitch>,
}

/// New text vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "newTextVector")]
pub struct NewTextVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    /// Text elements
    #[serde(rename = "oneText")]
    pub elements: Vec<OneText>,
}

/// New number vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "newNumberVector")]
pub struct NewNumberVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    /// Number elements
    #[serde(rename = "oneNumber")]
    pub elements: Vec<OneNumber>,
}

/// Text element in a new text vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "oneText")]
pub struct OneText {
    /// Text name
    #[serde(rename = "@name")]
    pub name: String,
    /// Text value
    #[serde(rename = "$text")]
    pub value: String,
}

/// Number element in a new number vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "oneNumber")]
pub struct OneNumber {
    /// Number name
    #[serde(rename = "@name")]
    pub name: String,
    /// Number value
    #[serde(rename = "$text")]
    pub value: String,
}

/// Light element in a new light vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "oneLight")]
pub struct OneLight {
    /// Light name
    #[serde(rename = "@name")]
    pub name: String,
    /// Light value
    #[serde(rename = "$text")]
    pub value: PropertyState,
}

/// BLOB element in a new BLOB vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "oneBLOB")]
pub struct OneBlob {
    /// BLOB name
    #[serde(rename = "@name")]
    pub name: String,
    /// BLOB size
    #[serde(rename = "@size")]
    pub size: usize,
    /// BLOB format
    #[serde(rename = "@format")]
    pub format: String,
    /// BLOB value
    #[serde(rename = "$text")]
    pub value: Vec<u8>,
}
