use serde::{Deserialize, Serialize};
use std::fmt;

/// Switch rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SwitchRule {
    /// Only one switch can be on
    OneOfMany,
    /// At most one switch can be on
    AtMostOne,
    /// Any number of switches can be on
    AnyOfMany,
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
