use crate::error::{Error, Result};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::timestamp::INDITimestamp;

use crate::message::{
    blob::{DefBLOBVector, EnableBlob, SetBLOBVector},
    light::DefLightVector,
    number::{DefNumberVector, SetNumberVector},
    switch::{DefSwitchVector, SetSwitchVector},
    text::{DefTextVector, SetTextVector},
};

use super::basic::GetProperties;

/// A message associated with a device or entire system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "message")]
pub struct Message {
    /// Device name (if absent, message is considered site-wide)
    #[serde(rename = "device", skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    
    /// Timestamp when this message was generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<INDITimestamp>,
    
    /// Message text/commentary
    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl Message {
    /// Create a new message
    pub fn new(message: String) -> Self {
        Self {
            device: None,
            timestamp: None,
            message: Some(message),
        }
    }

    /// Create a new device-specific message
    pub fn new_for_device(device: String, message: String) -> Self {
        Self {
            device: Some(device),
            timestamp: None,
            message: Some(message),
        }
    }

    /// Set the timestamp
    pub fn with_timestamp(mut self, timestamp: INDITimestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

/// INDI message type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    /// Get properties request
    GetProperties(GetProperties),
    /// General message
    Message(Message),
    /// Enable BLOB transfer
    EnableBLOB(EnableBlob),
    /// Define text vector
    DefTextVector(DefTextVector),
    /// Define number vector
    DefNumberVector(DefNumberVector),
    /// Define switch vector
    DefSwitchVector(DefSwitchVector),
    /// Define BLOB vector
    DefBLOBVector(DefBLOBVector),
    /// Define light vector
    DefLightVector(DefLightVector),
    /// Set text vector
    SetTextVector(SetTextVector),
    /// Set number vector
    SetNumberVector(SetNumberVector),
    /// Set switch vector
    SetSwitchVector(SetSwitchVector),
    /// Set BLOB vector
    SetBLOBVector(SetBLOBVector),
}

impl MessageType {
    /// Convert message to XML string
    pub fn to_xml(&self) -> Result<String> {
        to_string(&self).map_err(|e| Error::SerializationError(e.to_string()))
    }

    /// Parse a message from bytes asynchronously
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let s = std::str::from_utf8(bytes).map_err(|e| Error::ParseError(e.to_string()))?;
        Self::from_str(s)
    }
}

impl FromStr for MessageType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        from_str(s).map_err(|e| Error::ParseError(e.to_string()))
    }
}
