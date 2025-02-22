use crate::message::new::{OneBlob, OneLight, OneNumber, OneSwitch, OneText};
use crate::property::{PropertyState, SwitchState};
use crate::timestamp::INDITimestamp;
use serde::{Deserialize, Serialize};

/// Set text vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTextVector {
    /// Device name
    pub device: String,
    /// Property name
    pub name: String,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: INDITimestamp,
    /// Text elements
    pub texts: Vec<OneText>,
}

/// Set number vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "setNumberVector")]
pub struct SetNumberVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property state (optional)
    #[serde(rename = "@state", skip_serializing_if = "Option::is_none")]
    pub state: Option<PropertyState>,
    /// Worse-case time to affect a change (optional)
    #[serde(rename = "@timeout", skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
    /// Property timestamp (optional)
    #[serde(rename = "@timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<INDITimestamp>,
    /// Commentary message (optional)
    #[serde(rename = "@message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Number elements (at least one required)
    #[serde(rename = "oneNumber")]
    pub numbers: Vec<OneNumber>,
}

/// Set switch vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "setSwitchVector")]
pub struct SetSwitchVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property state (optional)
    #[serde(rename = "@state", skip_serializing_if = "Option::is_none")]
    pub state: Option<PropertyState>,
    /// Worse-case time to affect a change (optional)
    #[serde(rename = "@timeout", skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
    /// Property timestamp (optional)
    #[serde(rename = "@timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<INDITimestamp>,
    /// Commentary message (optional)
    #[serde(rename = "@message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Switch elements (at least one required)
    #[serde(rename = "oneSwitch")]
    pub switches: Vec<OneSwitch>,
}

/// Set light vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLightVector {
    /// Device name
    pub device: String,
    /// Property name
    pub name: String,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: INDITimestamp,
    /// Light elements
    pub lights: Vec<OneLight>,
}

/// Set blob vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetBlobVector {
    /// Device name
    pub device: String,
    /// Property name
    pub name: String,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: INDITimestamp,
    /// BLOB elements
    pub blobs: Vec<OneBlob>,
}
