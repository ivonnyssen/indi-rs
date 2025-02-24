use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_permissions() {
        assert_eq!(PropertyPerm::from_str("ro").unwrap(), PropertyPerm::Ro);
        assert_eq!(PropertyPerm::from_str("wo").unwrap(), PropertyPerm::Wo);
        assert_eq!(PropertyPerm::from_str("rw").unwrap(), PropertyPerm::Rw);
        assert!(PropertyPerm::from_str("invalid").is_err());

        assert_eq!(PropertyPerm::Ro.to_string(), "ro");
        assert_eq!(PropertyPerm::Wo.to_string(), "wo");
        assert_eq!(PropertyPerm::Rw.to_string(), "rw");
    }
}
