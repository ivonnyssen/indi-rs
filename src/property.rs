//! INDI Protocol Property Implementation
//!
//! This module provides the property types and traits for the INDI protocol.
//! Properties represent device characteristics and controls, with different
//! types (Number, Text, Switch, etc.), states (Idle, OK, Busy, Alert),
//! and permissions (RO, WO, RW).

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

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

/// Switch state for INDI switch properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchState {
    /// Switch is On
    #[serde(rename = "On")]
    On,
    /// Switch is Off
    #[serde(rename = "Off")]
    Off,
}

impl FromStr for SwitchState {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim() {
            "On" => Ok(SwitchState::On),
            "Off" => Ok(SwitchState::Off),
            _ => Err(Error::Property(format!("Invalid switch state: {}", s))),
        }
    }
}

impl fmt::Display for SwitchState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwitchState::On => write!(f, "On"),
            SwitchState::Off => write!(f, "Off"),
        }
    }
}

impl From<bool> for SwitchState {
    fn from(value: bool) -> Self {
        if value {
            SwitchState::On
        } else {
            SwitchState::Off
        }
    }
}

impl From<SwitchState> for bool {
    fn from(value: SwitchState) -> Self {
        match value {
            SwitchState::On => true,
            SwitchState::Off => false,
        }
    }
}

/// Rule for INDI switch properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SwitchRule {
    /// Only one switch can be On at a time
    #[serde(rename = "OneOfMany")]
    OneOfMany,
    /// At most one switch can be On, all can be Off
    #[serde(rename = "AtMostOne")]
    AtMostOne,
    /// Any number of switches can be On
    #[serde(rename = "AnyOfMany")]
    AnyOfMany,
}

impl FromStr for SwitchRule {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim() {
            "OneOfMany" => Ok(SwitchRule::OneOfMany),
            "AtMostOne" => Ok(SwitchRule::AtMostOne),
            "AnyOfMany" => Ok(SwitchRule::AnyOfMany),
            _ => Err(Error::Property(format!("Invalid switch rule: {}", s))),
        }
    }
}

impl fmt::Display for SwitchRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwitchRule::OneOfMany => write!(f, "OneOfMany"),
            SwitchRule::AtMostOne => write!(f, "AtMostOne"),
            SwitchRule::AnyOfMany => write!(f, "AnyOfMany"),
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
    Switch(SwitchState),
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
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
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
        timestamp: String,
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
            timestamp,
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
            label: None,
            group: None,
            timeout: None,
            timestamp,
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
            label: None,
            group: None,
            timeout: None,
            timestamp,
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

    /// Validates switch states according to the given rule
    pub fn validate_switch_states(&self, rule: SwitchRule) -> Result<()> {
        if let PropertyValue::SwitchVector(switches) = &self.value {
            let on_count = switches
                .values()
                .filter(|&&state| state == SwitchState::On)
                .count();

            match rule {
                SwitchRule::OneOfMany if on_count != 1 => Err(Error::Property(format!(
                    "OneOfMany rule requires exactly one switch to be On, found {}",
                    on_count
                ))),
                SwitchRule::AtMostOne if on_count > 1 => Err(Error::Property(format!(
                    "AtMostOne rule allows at most one switch to be On, found {}",
                    on_count
                ))),
                _ => Ok(()),
            }
        } else {
            Err(Error::Property("Not a switch vector property".to_string()))
        }
    }

    /// Validates timestamp format
    pub fn validate_timestamp(&self) -> Result<()> {
        timestamp::validate(&self.timestamp)
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

/// Timestamp format validation and generation
pub mod timestamp {
    use crate::error::{Error, Result};
    use chrono::{Local, NaiveDateTime};

    /// Format string for INDI timestamps (YYYY-MM-DDTHH:MM:SS.sss)
    const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%3f";

    /// Validates if a timestamp string follows the INDI protocol format
    pub fn validate(timestamp: &str) -> Result<()> {
        NaiveDateTime::parse_from_str(timestamp, TIMESTAMP_FORMAT)
            .map_err(|e| Error::Property(format!("Invalid timestamp format: {}", e)))
            .map(|_| ())
    }

    /// Generates a current timestamp in INDI protocol format
    pub fn now() -> String {
        Local::now()
            .naive_local()
            .format(TIMESTAMP_FORMAT)
            .to_string()
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
            PropertyPerm::ReadWrite,
            timestamp::now(),
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
            timestamp::now(),
        );
        assert!(ro_prop.is_readable());
        assert!(!ro_prop.is_writable());

        let wo_prop = Property::new(
            "test".to_string(),
            "test".to_string(),
            PropertyValue::Text("test".to_string()),
            PropertyState::Ok,
            PropertyPerm::WriteOnly,
            timestamp::now(),
        );
        assert!(!wo_prop.is_readable());
        assert!(wo_prop.is_writable());

        let rw_prop = Property::new(
            "test".to_string(),
            "test".to_string(),
            PropertyValue::Text("test".to_string()),
            PropertyState::Ok,
            PropertyPerm::ReadWrite,
            timestamp::now(),
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

        // Test bool conversion
        assert!(bool::from(SwitchState::On));
        assert!(!bool::from(SwitchState::Off));
        assert_eq!(SwitchState::from(true), SwitchState::On);
        assert_eq!(SwitchState::from(false), SwitchState::Off);
    }

    #[test]
    fn test_property_value_display() {
        let text = PropertyValue::Text("test".to_string());
        let num = PropertyValue::Number(42.0, None);
        let num_fmt = PropertyValue::Number(42.0, Some("m/s".to_string()));
        let switch_on = PropertyValue::Switch(SwitchState::On);
        let switch_off = PropertyValue::Switch(SwitchState::Off);
        let light = PropertyValue::Light(PropertyState::Ok);
        let blob = PropertyValue::Blob {
            size: 100,
            format: "fits".to_string(),
            data: vec![0; 100],
        };
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
        assert_eq!(blob.to_string(), "[BLOB format=fits size=100]");
        assert_eq!(switch_vector.to_string(), "switch1=On,switch2=Off");
        assert_eq!(text_vector.to_string(), "text1=text1,text2=text2");
        assert_eq!(number_vector.to_string(), "number1=42,number2=24");
    }

    #[test]
    fn test_switch_rules() {
        // Test OneOfMany rule
        let mut switches = HashMap::new();
        switches.insert("switch1".to_string(), SwitchState::On);
        switches.insert("switch2".to_string(), SwitchState::Off);
        let prop = Property::new(
            "device1".to_string(),
            "prop1".to_string(),
            PropertyValue::SwitchVector(switches),
            PropertyState::Ok,
            PropertyPerm::ReadWrite,
            timestamp::now(),
        );
        assert!(prop.validate_switch_states(SwitchRule::OneOfMany).is_ok());

        // Test OneOfMany rule violation
        let mut switches = HashMap::new();
        switches.insert("switch1".to_string(), SwitchState::On);
        switches.insert("switch2".to_string(), SwitchState::On);
        let prop = Property::new(
            "device1".to_string(),
            "prop1".to_string(),
            PropertyValue::SwitchVector(switches),
            PropertyState::Ok,
            PropertyPerm::ReadWrite,
            timestamp::now(),
        );
        assert!(prop.validate_switch_states(SwitchRule::OneOfMany).is_err());

        // Test AtMostOne rule
        let mut switches = HashMap::new();
        switches.insert("switch1".to_string(), SwitchState::Off);
        switches.insert("switch2".to_string(), SwitchState::Off);
        let prop = Property::new(
            "device1".to_string(),
            "prop1".to_string(),
            PropertyValue::SwitchVector(switches),
            PropertyState::Ok,
            PropertyPerm::ReadWrite,
            timestamp::now(),
        );
        assert!(prop.validate_switch_states(SwitchRule::AtMostOne).is_ok());
    }

    #[test]
    fn test_timestamp_validation() {
        // Test valid timestamp
        let timestamp = "2024-02-15T10:30:00.000";
        assert!(timestamp::validate(timestamp).is_ok());

        // Test another valid timestamp
        let timestamp = "2025-02-14T00:42:55.000";
        assert!(timestamp::validate(timestamp).is_ok());

        // Test invalid timestamp
        let timestamp = "invalid";
        assert!(timestamp::validate(timestamp).is_err());

        // Test timestamp without milliseconds
        let timestamp = "2024-02-15T10:30:00";
        assert!(timestamp::validate(timestamp).is_err());

        // Test timestamp generation
        let now = timestamp::now();
        assert!(timestamp::validate(&now).is_ok());
    }
}
