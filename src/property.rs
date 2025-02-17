//! INDI Protocol Property Implementation
//!
//! This module provides the property types and traits for the INDI protocol.
//! Properties represent device characteristics and controls, with different
//! types (Number, Text, Switch, etc.), states (Idle, OK, Busy, Alert),
//! and permissions (RO, WO, RW).

use crate::error::{Error, Result};
use quick_xml::se::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Property permission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyPerm {
    /// Read-only property
    Ro,
    /// Write-only property
    Wo,
    /// Read-write property
    Rw,
}

impl FromStr for PropertyPerm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ro" => Ok(PropertyPerm::Ro),
            "wo" => Ok(PropertyPerm::Wo),
            "rw" => Ok(PropertyPerm::Rw),
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
            PropertyPerm::Ro => write!(f, "ro"),
            PropertyPerm::Wo => write!(f, "wo"),
            PropertyPerm::Rw => write!(f, "rw"),
        }
    }
}

/// Property state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyState {
    /// Property is idle
    Idle,
    /// Property is being updated
    Ok,
    /// Property is busy
    Busy,
    /// Property has an alert
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

/// Switch state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwitchState {
    /// Switch is off
    Off,
    /// Switch is on
    On,
}

impl FromStr for SwitchState {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Off" => Ok(SwitchState::Off),
            "On" => Ok(SwitchState::On),
            _ => Err(Error::InvalidSwitchState(s.to_string())),
        }
    }
}

impl fmt::Display for SwitchState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwitchState::Off => write!(f, "Off"),
            SwitchState::On => write!(f, "On"),
        }
    }
}

/// Switch rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SwitchRule {
    /// Only one switch can be On at a time
    OneOfMany,
    /// At most one switch can be On, all can be Off
    AtMostOne,
    /// Any number of switches can be On
    AnyOfMany,
}

/// Property value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    /// Text value
    Text(String),
    /// Number value with optional format
    Number(f64, Option<String>),
    /// Switch value
    Switch(SwitchState),
    /// Light value
    Light(PropertyState),
    /// BLOB value
    Blob(Vec<u8>),
    /// Switch vector value
    SwitchVector(HashMap<String, SwitchState>),
    /// Text vector value
    TextVector(HashMap<String, String>),
    /// Number vector value
    NumberVector(HashMap<String, f64>),
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
            PropertyValue::Switch(state) => write!(f, "{}", state),
            PropertyValue::Light(state) => write!(f, "{}", state),
            PropertyValue::Blob(_) => write!(f, "[BLOB]"),
            PropertyValue::SwitchVector(switches) => {
                let mut entries: Vec<_> = switches.iter().collect();
                entries.sort_by(|(a, _), (b, _)| a.cmp(b));
                let mut result = String::new();
                for (name, state) in entries {
                    if !result.is_empty() {
                        result.push(',');
                    }
                    result.push_str(&format!("{}={}", name, state));
                }
                write!(f, "{}", result)
            }
            PropertyValue::TextVector(texts) => {
                let mut entries: Vec<_> = texts.iter().collect();
                entries.sort_by(|(a, _), (b, _)| a.cmp(b));
                let mut result = String::new();
                for (name, text) in entries {
                    if !result.is_empty() {
                        result.push(',');
                    }
                    result.push_str(&format!("{}={}", name, text));
                }
                write!(f, "{}", result)
            }
            PropertyValue::NumberVector(numbers) => {
                let mut entries: Vec<_> = numbers.iter().collect();
                entries.sort_by(|(a, _), (b, _)| a.cmp(b));
                let mut result = String::new();
                for (name, num) in entries {
                    if !result.is_empty() {
                        result.push(',');
                    }
                    result.push_str(&format!("{}={}", name, num));
                }
                write!(f, "{}", result)
            }
        }
    }
}

/// Property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    /// Device name
    pub device: String,
    /// Property name
    pub name: String,
    /// Property value
    pub value: PropertyValue,
    /// Property state
    pub state: PropertyState,
    /// Property permission
    pub perm: PropertyPerm,
    /// Property timestamp
    pub timestamp: String,
    /// Property label (optional)
    pub label: Option<String>,
    /// Property group (optional)
    pub group: Option<String>,
    /// Property timeout (optional)
    pub timeout: Option<u32>,
    /// Child elements (optional)
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
        timestamp: String,
    ) -> Self {
        Self {
            device,
            name,
            value,
            state,
            perm,
            timestamp,
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
        timestamp: String,
    ) -> Self {
        Self {
            device,
            name: element_name,
            value,
            state,
            perm,
            timestamp,
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
        timestamp: String,
    ) -> Self {
        Self {
            device,
            name: String::default(),
            value: PropertyValue::default(),
            state,
            perm,
            timestamp,
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
        matches!(self.perm, PropertyPerm::Ro | PropertyPerm::Rw)
    }

    /// Returns true if the property is writable
    pub fn is_writable(&self) -> bool {
        use crate::property::PropertyPerm;
        matches!(self.perm, PropertyPerm::Wo | PropertyPerm::Rw)
    }

    /// Convert property to XML string
    pub fn to_xml(&self) -> Result<String> {
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
    use std::collections::HashMap;

    #[test]
    fn test_property_creation() {
        let prop = Property::new(
            "test_device".to_string(),
            "test_prop".to_string(),
            PropertyValue::Text("test".to_string()),
            PropertyState::Ok,
            PropertyPerm::Rw,
            timestamp::generate(),
        )
        .with_label("Test Property".to_string())
        .with_group("Main".to_string())
        .with_timeout(1000);

        assert_eq!(prop.device, "test_device");
        assert_eq!(prop.name, "test_prop");
        assert_eq!(prop.label.unwrap(), "Test Property");
        assert_eq!(prop.group.unwrap(), "Main");
        assert_eq!(prop.state, PropertyState::Ok);
        assert_eq!(prop.perm, PropertyPerm::Rw);
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
            PropertyPerm::Ro,
            timestamp::generate(),
        );
        assert!(ro_prop.is_readable());
        assert!(!ro_prop.is_writable());

        let wo_prop = Property::new(
            "test".to_string(),
            "test".to_string(),
            PropertyValue::Text("test".to_string()),
            PropertyState::Ok,
            PropertyPerm::Wo,
            timestamp::generate(),
        );
        assert!(!wo_prop.is_readable());
        assert!(wo_prop.is_writable());

        let rw_prop = Property::new(
            "test".to_string(),
            "test".to_string(),
            PropertyValue::Text("test".to_string()),
            PropertyState::Ok,
            PropertyPerm::Rw,
            timestamp::generate(),
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
    fn test_switch_state() {
        // Test FromStr implementation
        assert_eq!(SwitchState::from_str("On").unwrap(), SwitchState::On);
        assert_eq!(SwitchState::from_str("Off").unwrap(), SwitchState::Off);
        assert!(SwitchState::from_str("Invalid").is_err());

        // Test Display implementation
        assert_eq!(SwitchState::On.to_string(), "On");
        assert_eq!(SwitchState::Off.to_string(), "Off");
    }

    #[test]
    fn test_property_value_display() {
        let text = PropertyValue::Text("test".to_string());
        let num = PropertyValue::Number(42.0, None);
        let num_fmt = PropertyValue::Number(42.0, Some("m/s".to_string()));
        let switch_on = PropertyValue::Switch(SwitchState::On);
        let switch_off = PropertyValue::Switch(SwitchState::Off);
        let light = PropertyValue::Light(PropertyState::Ok);
        let blob = PropertyValue::Blob(vec![0; 100]);
        let switch_vector = PropertyValue::SwitchVector(HashMap::from([
            ("switch1".to_string(), SwitchState::On),
            ("switch2".to_string(), SwitchState::Off),
        ]));
        let text_vector = PropertyValue::TextVector(HashMap::from([
            ("text1".to_string(), "text1".to_string()),
            ("text2".to_string(), "text2".to_string()),
        ]));
        let number_vector = PropertyValue::NumberVector(HashMap::from([
            ("number1".to_string(), 42.0),
            ("number2".to_string(), 24.0),
        ]));

        assert_eq!(text.to_string(), "test");
        assert_eq!(num.to_string(), "42");
        assert_eq!(num_fmt.to_string(), "42 m/s");
        assert_eq!(switch_on.to_string(), "On");
        assert_eq!(switch_off.to_string(), "Off");
        assert_eq!(light.to_string(), "Ok");
        assert_eq!(blob.to_string(), "[BLOB]");
        assert_eq!(switch_vector.to_string(), "switch1=On,switch2=Off");
        assert_eq!(text_vector.to_string(), "text1=text1,text2=text2");
        assert_eq!(number_vector.to_string(), "number1=42,number2=24");
    }
}

/// Timestamp format validation and generation
pub mod timestamp {
    use crate::error::{Error, Result};
    use chrono::{DateTime, Utc};

    /// Validate timestamp format
    pub fn validate(timestamp: &str) -> Result<()> {
        DateTime::parse_from_rfc3339(timestamp).map_err(|e| Error::ParseError(e.to_string()))?;
        Ok(())
    }

    /// Generate current timestamp
    pub fn generate() -> String {
        Utc::now().to_rfc3339()
    }
}
