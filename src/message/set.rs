use crate::message::new::{OneBlob, OneLight, OneNumber, OneSwitch, OneText};
use serde::{Deserialize, Serialize};

/// Set text vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetTextVector {
    /// Device name
    pub device: String,
    /// Property name
    pub name: String,
    /// Text elements
    pub texts: Vec<OneText>,
}

/// Set number vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetNumberVector {
    /// Device name
    pub device: String,
    /// Property name
    pub name: String,
    /// Number elements
    pub numbers: Vec<OneNumber>,
}

/// Set switch vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSwitchVector {
    /// Device name
    pub device: String,
    /// Property name
    pub name: String,
    /// Switch elements
    pub switches: Vec<OneSwitch>,
}

/// Set light vector message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLightVector {
    /// Device name
    pub device: String,
    /// Property name
    pub name: String,
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
    /// BLOB elements
    pub blobs: Vec<OneBlob>,
}
