use crate::error::{Error, Result};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Message handling for the INDI protocol
pub mod basic;
/// Message definitions for the INDI protocol
pub mod definition;
/// Message types for creating new properties
pub mod new;
/// Message types for setting property values
pub mod set;
/// Message types for BLOB data
pub mod blob;
/// Tests for the message module
#[cfg(test)]
mod tests;

/// Raw message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The raw XML content of the message
    pub content: String,
}

impl Message {
    /// Create a new message
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

/// INDI message type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    /// Get properties request
    GetProperties(basic::GetProperties),
    /// General message
    Message(Message),
    /// Enable BLOB transfer
    EnableBLOB(blob::EnableBlob),
    /// Define text vector
    DefTextVector(definition::DefTextVector),
    /// Define number vector
    DefNumberVector(definition::DefNumberVector),
    /// Define switch vector
    DefSwitchVector(definition::DefSwitchVector),
    /// Define BLOB vector
    DefBLOBVector(definition::DefBLOBVector),
    /// New text vector
    NewTextVector(new::NewTextVector),
    /// New number vector
    NewNumberVector(new::NewNumberVector),
    /// New switch vector
    NewSwitchVector(new::NewSwitchVector),
    /// New BLOB vector
    NewBLOBVector(blob::NewBLOBVector),
    /// Set text vector
    SetTextVector(set::SetTextVector),
    /// Set number vector
    SetNumberVector(set::SetNumberVector),
    /// Set switch vector
    SetSwitchVector(set::SetSwitchVector),
    /// Set BLOB vector
    SetBLOBVector(blob::SetBLOBVector),
}

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

impl MessageType {
    /// Convert message to XML string
    pub fn to_xml(&self) -> Result<String> {
        to_string(&self).map_err(|e| Error::SerializationError(e.to_string()))
    }

    /// Parse a message from bytes asynchronously
    pub async fn from_bytes(bytes: &[u8]) -> Result<Self> {
        from_str(std::str::from_utf8(bytes).unwrap()).map_err(Error::XmlDe)
    }
}

impl FromStr for MessageType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        from_str(s).map_err(Error::XmlDe)
    }
}
