use crate::error::Result;
use crate::prelude::PropertyPerm;
use crate::property::{PropertyState, SwitchRule, SwitchState};
use serde::{Deserialize, Serialize};

/// Text vector definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defTextVector")]
pub struct DefTextVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property label
    #[serde(rename = "@label")]
    pub label: String,
    /// Property group
    #[serde(rename = "@group")]
    pub group: String,
    /// Property state
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Property permission
    #[serde(rename = "@perm")]
    pub perm: PropertyPerm,
    /// Property timeout
    #[serde(rename = "@timeout")]
    pub timeout: i32,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    /// Text elements
    #[serde(rename = "defText")]
    pub texts: Vec<DefText>,
}

/// Number vector definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defNumberVector")]
pub struct DefNumberVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property label
    #[serde(rename = "@label")]
    pub label: String,
    /// Property group
    #[serde(rename = "@group")]
    pub group: String,
    /// Property state
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Property permission
    #[serde(rename = "@perm")]
    pub perm: PropertyPerm,
    /// Property timeout
    #[serde(rename = "@timeout")]
    pub timeout: i32,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    /// Number elements
    #[serde(rename = "defNumber")]
    pub numbers: Vec<DefNumber>,
}

/// Switch element in a switch vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defSwitch")]
pub struct DefSwitch {
    /// Switch name
    #[serde(rename = "@name")]
    pub name: String,
    /// Switch label
    #[serde(rename = "@label")]
    pub label: String,
    /// Switch state
    #[serde(rename = "$text")]
    pub state: SwitchState,
}

/// Text element in a text vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefText {
    /// Text name
    #[serde(rename = "@name")]
    pub name: String,
    /// Text label
    #[serde(rename = "@label")]
    pub label: String,
    /// Text value
    #[serde(rename = "$text")]
    pub value: String,
}

/// Number element in a number vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefNumber {
    /// Number name
    #[serde(rename = "@name")]
    pub name: String,
    /// Number label
    #[serde(rename = "@label")]
    pub label: String,
    /// Number format
    #[serde(rename = "@format")]
    pub format: String,
    /// Number minimum value
    #[serde(rename = "@min")]
    pub min: String,
    /// Number maximum value
    #[serde(rename = "@max")]
    pub max: String,
    /// Number step value
    #[serde(rename = "@step")]
    pub step: String,
    /// Number value
    #[serde(rename = "$text")]
    pub value: String,
}

/// Represents a switch vector property definition in the INDI protocol.
/// Contains information about a set of switches including their device, name,
/// state, and individual switch elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defSwitchVector")]
pub struct DefSwitchVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property label
    #[serde(rename = "@label")]
    pub label: String,
    /// Property group
    #[serde(rename = "@group")]
    pub group: String,
    /// Property state
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Property permission
    #[serde(rename = "@perm")]
    pub perm: PropertyPerm,
    /// Switch rule
    #[serde(rename = "@rule")]
    pub rule: SwitchRule,
    /// Property timeout
    #[serde(rename = "@timeout")]
    pub timeout: i32,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    /// Message
    #[serde(rename = "@message")]
    pub message: String,
    /// Switch elements
    #[serde(rename = "defSwitch")]
    pub switches: Vec<DefSwitch>,
}

impl DefSwitchVector {
    /// Validates the switch vector according to its rule
    pub fn validate(&self) -> Result<()> {
        match self.rule {
            SwitchRule::OneOfMany => {
                let on_count = self
                    .switches
                    .iter()
                    .filter(|s| s.state == SwitchState::On)
                    .count();
                if on_count > 1 {
                    return Err(crate::error::Error::InvalidSwitchState(
                        "OneOfMany rule violated: more than one switch is ON".to_string(),
                    ));
                }
            }
            SwitchRule::AtMostOne => {
                let on_count = self
                    .switches
                    .iter()
                    .filter(|s| s.state == SwitchState::On)
                    .count();
                if on_count > 1 {
                    return Err(crate::error::Error::InvalidSwitchState(
                        "AtMostOne rule violated: more than one switch is ON".to_string(),
                    ));
                }
            }
            SwitchRule::AnyOfMany => (),
        }
        Ok(())
    }
}
