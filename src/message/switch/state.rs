use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use crate::error::{Error, Result};

/// Switch state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SwitchState {
    /// Switch is on
    On,
    /// Switch is off
    Off,
}

impl fmt::Display for SwitchState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwitchState::On => write!(f, "On"),
            SwitchState::Off => write!(f, "Off"),
        }
    }
}

impl FromStr for SwitchState {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "On" => Ok(SwitchState::On),
            "Off" => Ok(SwitchState::Off),
            _ => Err(Error::ParseError(format!(
                "Invalid switch state: {}",
                s
            ))),
        }
    }
}
