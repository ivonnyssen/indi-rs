//! INDI Protocol Property Implementation
//!
//! This module provides the property types and traits for the INDI protocol.
//! Properties represent device characteristics and controls, with different
//! types (Number, Text, Switch, etc.), states (Idle, OK, Busy, Alert),
//! and permissions (RO, WO, RW).

use serde::Serialize;
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// Property permission
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyPerm {
    /// Read-only permission
    #[default]
    #[serde(rename = "ro")]
    ReadOnly,
    /// Write-only permission
    #[serde(rename = "wo")]
    WriteOnly,
    /// Read-write permission
    #[serde(rename = "rw")]
    ReadWrite,
}

impl FromStr for PropertyPerm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ro" => Ok(PropertyPerm::ReadOnly),
            "wo" => Ok(PropertyPerm::WriteOnly),
            "rw" => Ok(PropertyPerm::ReadWrite),
            _ => Err(Error::Property(format!(
                "Invalid property permission: {}",
                s
            ))),
        }
    }
}

impl fmt::Display for PropertyPerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyPerm::ReadOnly => write!(f, "ro"),
            PropertyPerm::WriteOnly => write!(f, "wo"),
            PropertyPerm::ReadWrite => write!(f, "rw"),
        }
    }
}

/// Property state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyState {
    /// Device/property is idle
    #[serde(rename = "Idle")]
    Idle,
    /// Device/property is operating correctly
    #[serde(rename = "Ok")]
    Ok,
    /// Device/property is busy
    #[serde(rename = "Busy")]
    Busy,
    /// Device/property has an error condition
    #[serde(rename = "Alert")]
    Alert,
}

impl FromStr for PropertyState {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Idle" => Ok(PropertyState::Idle),
            "Ok" => Ok(PropertyState::Ok),
            "Busy" => Ok(PropertyState::Busy),
            "Alert" => Ok(PropertyState::Alert),
            _ => Err(Error::Property(format!("Invalid property state: {}", s))),
        }
    }
}

impl fmt::Display for PropertyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyState::Idle => write!(f, "Idle"),
            PropertyState::Ok => write!(f, "Ok"),
            PropertyState::Busy => write!(f, "Busy"),
            PropertyState::Alert => write!(f, "Alert"),
        }
    }
}

/// Property value types
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PropertyValue {
    /// Switch value
    #[serde(rename = "oneSwitch")]
    Switch(bool),
    /// Text value
    #[serde(rename = "oneText")]
    Text(String),
    /// Number value with optional format
    #[serde(rename = "oneNumber")]
    Number(f64, Option<String>),
    /// Light value
    #[serde(rename = "oneLight")]
    Light(PropertyState),
    /// BLOB value
    #[serde(rename = "oneBLOB")]
    Blob {
        /// Format of the BLOB data (e.g., ".fits", ".raw", etc.)
        format: String,
        /// Binary data of the BLOB
        #[serde(serialize_with = "serialize_base64")]
        data: Vec<u8>,
        /// Size of the binary data in bytes
        size: usize,
    },
}

impl Default for PropertyValue {
    fn default() -> Self {
        Self::Text(String::default())
    }
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Text(s) => write!(f, "{}", s),
            PropertyValue::Number(n, Some(_fmt_str)) => write!(f, "{n}"),
            PropertyValue::Number(n, None) => write!(f, "{}", n),
            PropertyValue::Switch(b) => write!(f, "{}", if *b { "On" } else { "Off" }),
            PropertyValue::Light(s) => write!(f, "{}", s),
            PropertyValue::Blob {
                data: _,
                format,
                size,
            } => write!(f, "{} bytes ({})", size, format),
        }
    }
}

/// Property definition
#[derive(Debug, Clone, Serialize)]
#[serde(rename = "property")]
pub struct Property {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property label (optional)
    #[serde(rename = "label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Property group (optional)
    #[serde(rename = "group", skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Property value
    #[serde(flatten)]
    pub value: PropertyValue,
    /// Property state
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Property permissions
    #[serde(rename = "@perm")]
    pub perm: PropertyPerm,
    /// Property timeout (optional)
    #[serde(rename = "@timeout", skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    /// Child elements (optional)
    #[serde(rename = "elements", skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<Property>>,
}

impl Property {
    /// Create a new property
    pub fn new(
        device: String,
        name: String,
        value: PropertyValue,
        state: PropertyState,
        perm: PropertyPerm,
    ) -> Self {
        Self {
            device,
            name,
            label: None,
            group: None,
            value,
            state,
            perm,
            timeout: None,
            elements: None,
        }
    }

    /// Create a new property with value
    pub fn new_with_value(
        device: String,
        _name: String, // Parent property name, unused in this context
        element_name: String,
        value: PropertyValue,
        state: PropertyState,
        perm: PropertyPerm,
    ) -> Self {
        Self {
            device,
            name: element_name,
            label: None,
            group: None,
            value,
            state,
            perm,
            timeout: None,
            elements: None,
        }
    }

    /// Create a new property with elements
    pub fn new_with_elements(
        device: String,
        _name: String,
        elements: Vec<Property>,
        state: PropertyState,
        perm: PropertyPerm,
    ) -> Self {
        Self {
            device,
            name: String::default(),
            label: None,
            group: None,
            value: PropertyValue::default(),
            state,
            perm,
            timeout: None,
            elements: Some(elements),
        }
    }

    /// Sets the property label
    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Sets the property group
    pub fn with_group(mut self, group: String) -> Self {
        self.group = Some(group);
        self
    }

    /// Sets the property timeout
    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Returns true if the property is readable
    pub fn is_readable(&self) -> bool {
        use crate::property::PropertyPerm;
        matches!(self.perm, PropertyPerm::ReadOnly | PropertyPerm::ReadWrite)
    }

    /// Returns true if the property is writable
    pub fn is_writable(&self) -> bool {
        use crate::property::PropertyPerm;
        matches!(self.perm, PropertyPerm::WriteOnly | PropertyPerm::ReadWrite)
    }

    /// Serializes the property to XML
    pub fn to_xml(&self) -> Result<String> {
        use quick_xml::se;
        use serde::Serialize;
        let mut writer = String::new();
        let ser = se::Serializer::new(&mut writer);
        self.serialize(ser)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        Ok(writer)
    }
}

fn serialize_base64<S>(data: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    serializer.serialize_str(&STANDARD.encode(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_creation() {
        let prop = Property::new(
            "test_device".to_string(),
            "test_prop".to_string(),
            PropertyValue::Text("test".to_string()),
            PropertyState::Ok,
            PropertyPerm::ReadWrite,
        )
        .with_label("Test Property".to_string())
        .with_group("Main".to_string())
        .with_timeout(1000);

        assert_eq!(prop.device, "test_device");
        assert_eq!(prop.name, "test_prop");
        assert_eq!(prop.label.unwrap(), "Test Property");
        assert_eq!(prop.group.unwrap(), "Main");
        assert_eq!(prop.state, PropertyState::Ok);
        assert_eq!(prop.perm, PropertyPerm::ReadWrite);
        assert_eq!(prop.timeout.unwrap(), 1000);
        assert!(matches!(prop.value, PropertyValue::Text(_)));
    }

    #[test]
    fn test_property_permissions() {
        let ro_prop = Property::new(
            "test".to_string(),
            "test".to_string(),
            PropertyValue::Text("test".to_string()),
            PropertyState::Ok,
            PropertyPerm::ReadOnly,
        );
        assert!(ro_prop.is_readable());
        assert!(!ro_prop.is_writable());

        let wo_prop = Property::new(
            "test".to_string(),
            "test".to_string(),
            PropertyValue::Text("test".to_string()),
            PropertyState::Ok,
            PropertyPerm::WriteOnly,
        );
        assert!(!wo_prop.is_readable());
        assert!(wo_prop.is_writable());

        let rw_prop = Property::new(
            "test".to_string(),
            "test".to_string(),
            PropertyValue::Text("test".to_string()),
            PropertyState::Ok,
            PropertyPerm::ReadWrite,
        );
        assert!(rw_prop.is_readable());
        assert!(rw_prop.is_writable());
    }

    #[test]
    fn test_property_states() {
        assert_eq!(
            "Idle".parse::<PropertyState>().unwrap(),
            PropertyState::Idle
        );
        assert_eq!("Ok".parse::<PropertyState>().unwrap(), PropertyState::Ok);
        assert_eq!(
            "Busy".parse::<PropertyState>().unwrap(),
            PropertyState::Busy
        );
        assert_eq!(
            "Alert".parse::<PropertyState>().unwrap(),
            PropertyState::Alert
        );
        assert!("Invalid".parse::<PropertyState>().is_err());
    }

    #[test]
    fn test_property_value_display() {
        assert_eq!(PropertyValue::Text("test".to_string()).to_string(), "test");
        assert_eq!(PropertyValue::Number(42.0, None).to_string(), "42");
        assert_eq!(PropertyValue::Switch(true).to_string(), "On");
        assert_eq!(PropertyValue::Light(PropertyState::Ok).to_string(), "Ok");
        assert_eq!(
            PropertyValue::Blob {
                data: vec![1, 2, 3],
                format: ".fits".to_string(),
                size: 3
            }
            .to_string(),
            "3 bytes (.fits)"
        );
    }
}
