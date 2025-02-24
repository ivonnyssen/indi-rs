use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Property state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PropertyState {
    /// Device/property is in idle state
    Idle,
    /// Device/property is in normal state
    Ok,
    /// Device/property is busy
    Busy,
    /// Device/property has an error
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_states() {
        assert_eq!("Idle".parse::<PropertyState>().unwrap(), PropertyState::Idle);
        assert_eq!("Ok".parse::<PropertyState>().unwrap(), PropertyState::Ok);
        assert_eq!("Busy".parse::<PropertyState>().unwrap(), PropertyState::Busy);
        assert_eq!("Alert".parse::<PropertyState>().unwrap(), PropertyState::Alert);
        assert!("Invalid".parse::<PropertyState>().is_err());

        assert_eq!(PropertyState::Idle.to_string(), "Idle");
        assert_eq!(PropertyState::Ok.to_string(), "Ok");
        assert_eq!(PropertyState::Busy.to_string(), "Busy");
        assert_eq!(PropertyState::Alert.to_string(), "Alert");
    }
}
