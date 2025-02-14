//! INDI Protocol Property Implementation
//!
//! This module provides the property types and traits for the INDI protocol.
//! Properties represent device characteristics and controls, with different
//! types (Number, Text, Switch, etc.), states (Idle, OK, Busy, Alert),
//! and permissions (RO, WO, RW).

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use crate::error::{Error, Result};
use serde::{Serialize, Deserialize};

/// Property permission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PropertyState {
    /// Property is idle
    #[default]
    Idle,
    /// Property is ok
    Ok,
    /// Property is busy
    Busy,
    /// Property is in alert state
    Alert,
}

impl FromStr for PropertyState {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim() {
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

/// Property value types supported by INDI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    /// Text value
    Text(String),
    /// Number value with optional format string
    Number(f64, Option<String>),
    /// Switch value (On/Off)
    Switch(bool),
    /// Light value representing a state
    Light(PropertyState),
    /// Binary large object (BLOB)
    Blob {
        /// Size of the BLOB in bytes
        size: usize,
        /// Format of the BLOB (e.g., "fits", "raw", etc.)
        format: String,
        /// Binary data
        data: Vec<u8>,
    },
    /// Switch vector value
    SwitchVector(HashMap<String, bool>),
}

impl Default for PropertyValue {
    fn default() -> Self {
        PropertyValue::Text(String::new())
    }
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Text(text) => write!(f, "{}", text),
            PropertyValue::Number(num, None) => write!(f, "{}", num),
            PropertyValue::Number(num, Some(fmt_str)) => write!(f, "{} {}", num, fmt_str),
            PropertyValue::Switch(state) => write!(f, "{}", if *state { "On" } else { "Off" }),
            PropertyValue::Light(state) => write!(f, "{}", state),
            PropertyValue::Blob { format, size, .. } => {
                write!(f, "[BLOB format={} size={}]", format, size)
            }
            PropertyValue::SwitchVector(switches) => {
                let mut entries: Vec<_> = switches.iter().collect();
                entries.sort_by(|(a, _), (b, _)| a.cmp(b));
                let mut result = String::new();
                for (name, state) in entries {
                    if !result.is_empty() {
                        result.push(',');
                    }
                    result.push_str(&format!("{}={}", name, if *state { "On" } else { "Off" }));
                }
                write!(f, "{}", result)
            }
        }
    }
}

/// INDI property
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "property")]
pub struct Property {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property value
    #[serde(flatten)]
    pub value: PropertyValue,
    /// Property state
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Property permissions
    #[serde(rename = "@perm")]
    pub perm: PropertyPerm,
    /// Property label (optional)
    #[serde(rename = "label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Property group (optional)
    #[serde(rename = "group", skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
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
            value,
            state,
            perm,
            label: None,
            group: None,
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
            value,
            state,
            perm,
            label: None,
            group: None,
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
            value: PropertyValue::default(),
            state,
            perm,
            label: None,
            group: None,
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
        use quick_xml::se::Serializer;
        let mut writer = String::new();
        let ser = Serializer::new(&mut writer);
        self.serialize(ser)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        Ok(writer)
    }
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
        let text = PropertyValue::Text("test".to_string());
        let num = PropertyValue::Number(42.0, None);
        let num_fmt = PropertyValue::Number(42.0, Some("m/s".to_string()));
        let switch_on = PropertyValue::Switch(true);
        let switch_off = PropertyValue::Switch(false);
        let light = PropertyValue::Light(PropertyState::Ok);
        let blob = PropertyValue::Blob {
            size: 100,
            format: "fits".to_string(),
            data: vec![0; 100],
        };
        let switch_vector = PropertyValue::SwitchVector(HashMap::from([("switch1".to_string(), true), ("switch2".to_string(), false)]));

        assert_eq!(text.to_string(), "test");
        assert_eq!(num.to_string(), "42");
        assert_eq!(num_fmt.to_string(), "42 m/s");
        assert_eq!(switch_on.to_string(), "On");
        assert_eq!(switch_off.to_string(), "Off");
        assert_eq!(light.to_string(), "Ok");
        assert_eq!(blob.to_string(), "[BLOB format=fits size=100]");
        assert_eq!(switch_vector.to_string(), "switch1=On,switch2=Off");
    }
}
