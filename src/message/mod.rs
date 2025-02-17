use crate::error::{Error, Result};
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

/// Raw message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The raw XML content of the message
    pub content: String,
}

/// INDI message type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    /// Get properties request
    GetProperties(GetProperties),
    /// General message
    Message(Message),
    /// Enable BLOB transfer
    EnableBLOB(EnableBLOB),
    /// Define text vector
    DefTextVector(definition::DefTextVector),
    /// Define number vector
    DefNumberVector(definition::DefNumberVector),
    /// Define switch vector
    DefSwitchVector(definition::DefSwitchVector),
    /// New text vector
    NewTextVector(new::NewTextVector),
    /// New number vector
    NewNumberVector(new::NewNumberVector),
    /// New switch vector
    NewSwitchVector(new::NewSwitchVector),
    /// Set text vector
    SetTextVector(set::SetTextVector),
    /// Set number vector
    SetNumberVector(set::SetNumberVector),
    /// Set switch vector
    SetSwitchVector(set::SetSwitchVector),
}

/// Get properties message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProperties {
    /// Protocol version
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Device name (optional)
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Property name (optional)
    pub name: Option<String>,
}

/// Enable BLOB message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableBLOB {
    /// Device name
    pub device: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Property name (optional)
    pub name: Option<String>,
    /// BLOB enable value
    pub value: String,
}

impl MessageType {
    /// Convert message to XML string
    pub fn to_xml(&self) -> Result<String> {
        to_string(&self).map_err(|e| Error::SerializationError(e.to_string()))
    }
}

impl FromStr for MessageType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        quick_xml::de::from_str(s).map_err(Error::XmlDe)
    }
}
