//! INDI Protocol Property Implementation
//! 
//! This module provides the property types and traits for the INDI protocol.
//! Properties represent device characteristics and controls, with different
//! types (Number, Text, Switch, etc.), states (Idle, OK, Busy, Alert),
//! and permissions (RO, WO, RW).

use std::str::FromStr;
use std::fmt;
use crate::error::Error;
use crate::Result;

/// Property permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyPerm {
    /// Read-only property
    RO,
    /// Write-only property
    WO,
    /// Read-write property
    RW,
}

impl FromStr for PropertyPerm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "RO" => Ok(PropertyPerm::RO),
            "WO" => Ok(PropertyPerm::WO),
            "RW" => Ok(PropertyPerm::RW),
            _ => Err(Error::Property(format!("Invalid property permission: {}", s))),
        }
    }
}

impl fmt::Display for PropertyPerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyPerm::RO => write!(f, "ro"),
            PropertyPerm::WO => write!(f, "wo"),
            PropertyPerm::RW => write!(f, "rw"),
        }
    }
}

/// Property state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyState {
    /// Property is idle
    Idle,
    /// Property is ok
    Ok,
    /// Property is busy
    Busy,
    /// Property has an alert
    Alert,
}

impl FromStr for PropertyState {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "idle" => Ok(PropertyState::Idle),
            "ok" => Ok(PropertyState::Ok),
            "busy" => Ok(PropertyState::Busy),
            "alert" => Ok(PropertyState::Alert),
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
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// Text value
    Text(String),
    /// Number value with format
    Number(f64, Option<String>),
    /// Switch value (on/off)
    Switch(bool),
    /// Light value (represents device status)
    Light(PropertyState),
    /// BLOB value (Binary Large OBject)
    Blob(Vec<u8>),
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Text(s) => write!(f, "{}", s),
            PropertyValue::Number(n, Some(_fmt_str)) => write!(f, "{n}"),
            PropertyValue::Number(n, None) => write!(f, "{}", n),
            PropertyValue::Switch(b) => write!(f, "{}", if *b { "On" } else { "Off" }),
            PropertyValue::Light(s) => write!(f, "{}", s),
            PropertyValue::Blob(b) => write!(f, "{} bytes", b.len()),
        }
    }
}

/// Property definition
#[derive(Debug, Clone)]
pub struct Property {
    /// Device name
    pub device: String,
    /// Property name
    pub name: String,
    /// Property label (human-readable name)
    pub label: Option<String>,
    /// Property group
    pub group: Option<String>,
    /// Property state
    pub state: PropertyState,
    /// Property permission
    pub perm: PropertyPerm,
    /// Property timeout in seconds (0 for no timeout)
    pub timeout: u32,
    /// Property value
    pub value: PropertyValue,
}

impl Property {
    /// Creates a new property
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
            state,
            perm,
            timeout: 0,
            value,
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
        self.timeout = timeout;
        self
    }

    /// Returns true if the property is readable
    pub fn is_readable(&self) -> bool {
        matches!(self.perm, PropertyPerm::RO | PropertyPerm::RW)
    }

    /// Returns true if the property is writable
    pub fn is_writable(&self) -> bool {
        matches!(self.perm, PropertyPerm::WO | PropertyPerm::RW)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_permissions() {
        assert_eq!("ro".parse::<PropertyPerm>().unwrap(), PropertyPerm::RO);
        assert_eq!("wo".parse::<PropertyPerm>().unwrap(), PropertyPerm::WO);
        assert_eq!("rw".parse::<PropertyPerm>().unwrap(), PropertyPerm::RW);
        assert!("invalid".parse::<PropertyPerm>().is_err());
    }

    #[test]
    fn test_property_states() {
        assert_eq!("idle".parse::<PropertyState>().unwrap(), PropertyState::Idle);
        assert_eq!("ok".parse::<PropertyState>().unwrap(), PropertyState::Ok);
        assert_eq!("busy".parse::<PropertyState>().unwrap(), PropertyState::Busy);
        assert_eq!("alert".parse::<PropertyState>().unwrap(), PropertyState::Alert);
        assert!("invalid".parse::<PropertyState>().is_err());
    }

    #[test]
    fn test_property_value_display() {
        assert_eq!(
            PropertyValue::Text("test".to_string()).to_string(),
            "test"
        );
        assert_eq!(
            PropertyValue::Number(42.0, None).to_string(),
            "42"
        );
        assert_eq!(
            PropertyValue::Switch(true).to_string(),
            "On"
        );
        assert_eq!(
            PropertyValue::Light(PropertyState::Ok).to_string(),
            "Ok"
        );
        assert_eq!(
            PropertyValue::Blob(vec![1, 2, 3]).to_string(),
            "3 bytes"
        );
    }

    #[test]
    fn test_property_creation() {
        let prop = Property::new(
            "CCD".to_string(),
            "EXPOSURE".to_string(),
            PropertyValue::Number(1.0, Some("%.2f".to_string())),
            PropertyState::Idle,
            PropertyPerm::RW,
        )
        .with_label("Exposure Time".to_string())
        .with_group("Main Control".to_string())
        .with_timeout(60);

        assert_eq!(prop.device, "CCD");
        assert_eq!(prop.name, "EXPOSURE");
        assert_eq!(prop.label, Some("Exposure Time".to_string()));
        assert_eq!(prop.group, Some("Main Control".to_string()));
        assert_eq!(prop.state, PropertyState::Idle);
        assert_eq!(prop.perm, PropertyPerm::RW);
        assert_eq!(prop.timeout, 60);
        assert!(prop.is_readable());
        assert!(prop.is_writable());

        if let PropertyValue::Number(val, fmt) = prop.value {
            assert_eq!(val, 1.0);
            assert_eq!(fmt, Some("%.2f".to_string()));
        } else {
            panic!("Expected Number property value");
        }
    }
}
