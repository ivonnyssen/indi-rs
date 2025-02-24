use crate::prelude::PropertyPerm;
use crate::property::{PropertyState, SwitchRule, SwitchState};
use crate::timestamp::INDITimestamp;
use crate::message::common::vector::INDIVector;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Define a collection of switches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defSwitchVector")]
pub struct DefSwitchVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// GUI label, use name by default
    #[serde(rename = "@label")]
    pub label: Option<String>,
    /// Property group membership, blank by default
    #[serde(rename = "@group")]
    pub group: Option<String>,
    /// Current state of Property
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Ostensible Client controlability
    #[serde(rename = "@perm")]
    pub perm: PropertyPerm,
    /// Hint for GUI presentation
    #[serde(rename = "@rule")]
    pub rule: SwitchRule,
    /// Worse-case time, 0 default, N/A for ro
    #[serde(rename = "@timeout")]
    pub timeout: Option<f64>,
    /// Moment when these data were valid
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<INDITimestamp>,
    /// Commentary
    #[serde(rename = "@message")]
    pub message: Option<String>,
    /// Switch elements
    #[serde(rename = "defSwitch")]
    pub switches: Vec<DefSwitch>,
}

/// Define one member of a switch vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "defSwitch")]
pub struct DefSwitch {
    /// Name of this switch element
    #[serde(rename = "@name")]
    pub name: String,
    /// GUI label, or use name by default
    #[serde(rename = "@label")]
    pub label: Option<String>,
    /// Switch state
    #[serde(rename = "$text")]
    pub value: SwitchState,
}

impl DefSwitchVector {
    /// Validates the switch vector according to its rule
    pub fn validate(&self) -> Result<()> {
        match self.rule {
            SwitchRule::OneOfMany => {
                let on_count = self
                    .switches
                    .iter()
                    .filter(|s| s.value == SwitchState::On)
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
                    .filter(|s| s.value == SwitchState::On)
                    .count();
                if on_count > 1 {
                    return Err(crate::error::Error::InvalidSwitchState(
                        "AtMostOne rule violated: more than one switch is ON".to_string(),
                    ));
                }
            }
            SwitchRule::AnyOfMany => {
                // Any combination is valid
            }
        }
        Ok(())
    }
}

impl INDIVector for DefSwitchVector {
    type Element = DefSwitch;

    fn device(&self) -> &str {
        &self.device
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    fn state(&self) -> PropertyState {
        self.state
    }

    fn perm(&self) -> PropertyPerm {
        self.perm
    }

    fn timeout(&self) -> Option<f64> {
        self.timeout
    }

    fn timestamp(&self) -> Option<&INDITimestamp> {
        self.timestamp.as_ref()
    }

    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn elements(&self) -> &[Self::Element] {
        &self.switches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_def_switch() {
        let switch = DefSwitch {
            name: "test_switch".to_string(),
            label: Some("Test Switch".to_string()),
            value: SwitchState::Off,
        };

        assert_eq!(switch.name, "test_switch");
        assert_eq!(switch.label.unwrap(), "Test Switch");
        assert_eq!(switch.value, SwitchState::Off);
    }

    #[test]
    fn test_def_switch_vector() {
        let vector = DefSwitchVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: Some("Test Label".to_string()),
            group: Some("Test Group".to_string()),
            state: PropertyState::Ok,
            perm: PropertyPerm::Rw,
            rule: SwitchRule::OneOfMany,
            timeout: None,
            timestamp: None,
            message: None,
            switches: vec![DefSwitch {
                name: "switch1".to_string(),
                label: Some("Switch 1".to_string()),
                value: SwitchState::On,
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.label.unwrap(), "Test Label");
        assert_eq!(vector.group.unwrap(), "Test Group");
        assert_eq!(vector.state, PropertyState::Ok);
        assert_eq!(vector.perm, PropertyPerm::Rw);
        assert_eq!(vector.rule, SwitchRule::OneOfMany);
        assert!(vector.timeout.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
        assert_eq!(vector.switches.len(), 1);
    }

    #[test]
    fn test_def_switch_vector_with_all_fields() {
        let timestamp = INDITimestamp::from_str("2024-01-01T12:34:56.7").unwrap();
        let vector = DefSwitchVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: Some("Test Label".to_string()),
            group: Some("Test Group".to_string()),
            state: PropertyState::Ok,
            perm: PropertyPerm::Rw,
            rule: SwitchRule::OneOfMany,
            timeout: Some(1000.0),
            timestamp: Some(timestamp.clone()),
            message: Some("Test message".to_string()),
            switches: vec![DefSwitch {
                name: "switch1".to_string(),
                label: Some("Switch 1".to_string()),
                value: SwitchState::On,
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.label.unwrap(), "Test Label");
        assert_eq!(vector.group.unwrap(), "Test Group");
        assert_eq!(vector.state, PropertyState::Ok);
        assert_eq!(vector.perm, PropertyPerm::Rw);
        assert_eq!(vector.rule, SwitchRule::OneOfMany);
        assert_eq!(vector.timeout.unwrap(), 1000.0);
        assert_eq!(vector.timestamp.unwrap().to_string(), "2024-01-01T12:34:56.7");
        assert_eq!(vector.message.unwrap(), "Test message");
        assert_eq!(vector.switches.len(), 1);
    }

    #[test]
    fn test_switch_vector_trait_implementation() {
        let test_timestamp = "2024-01-01T00:00:00".parse().unwrap();
        let switch_vector = DefSwitchVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: Some("test_label".to_string()),
            group: Some("test_group".to_string()),
            state: PropertyState::Idle,
            perm: PropertyPerm::Rw,
            rule: SwitchRule::OneOfMany,
            timeout: Some(10.0),
            timestamp: Some(test_timestamp),
            message: Some("test_message".to_string()),
            switches: vec![],
        };

        // Test trait methods
        assert_eq!(switch_vector.device(), "test_device");
        assert_eq!(switch_vector.name(), "test_name");
        assert_eq!(switch_vector.label(), Some("test_label"));
        assert_eq!(switch_vector.group(), Some("test_group"));
        assert_eq!(switch_vector.state(), PropertyState::Idle);
        assert_eq!(switch_vector.perm(), PropertyPerm::Rw);
        assert_eq!(switch_vector.timeout(), Some(10.0));
        assert_eq!(switch_vector.message(), Some("test_message"));
        assert!(switch_vector.elements().is_empty());
        assert_eq!(switch_vector.timestamp().unwrap().to_string(), "2024-01-01T00:00:00");
    }

    #[test]
    fn test_switch_vector_validation() {
        let mut vector = DefSwitchVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            label: None,
            group: None,
            state: PropertyState::Idle,
            perm: PropertyPerm::Rw,
            rule: SwitchRule::OneOfMany,
            timeout: None,
            timestamp: None,
            message: None,
            switches: vec![
                DefSwitch {
                    name: "switch1".to_string(),
                    label: Some("Switch 1".to_string()),
                    value: SwitchState::On,
                },
                DefSwitch {
                    name: "switch2".to_string(),
                    label: Some("Switch 2".to_string()),
                    value: SwitchState::Off,
                },
            ],
        };

        // Test valid OneOfMany configuration
        assert!(vector.validate().is_ok());

        // Test invalid OneOfMany configuration
        vector.switches[1].value = SwitchState::On;
        assert!(vector.validate().is_err());

        // Test AtMostOne rule
        vector.rule = SwitchRule::AtMostOne;
        assert!(vector.validate().is_err());

        // Fix AtMostOne configuration
        vector.switches[0].value = SwitchState::Off;
        assert!(vector.validate().is_ok());

        // Test AnyOfMany rule
        vector.rule = SwitchRule::AnyOfMany;
        vector.switches[0].value = SwitchState::On;
        vector.switches[1].value = SwitchState::On;
        assert!(vector.validate().is_ok());
    }
}
