use serde::{Deserialize, Serialize};
use crate::timestamp::INDITimestamp;
use super::one::OneSwitch;

/// New switch value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSwitch {
    /// Switch name
    #[serde(rename = "@name")]
    pub name: String,
    /// Switch state
    #[serde(rename = "$text")]
    pub state: bool,
}

/// New switch vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "newSwitchVector")]
pub struct NewSwitchVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<INDITimestamp>,
    /// Switch elements
    #[serde(rename = "oneSwitch")]
    pub switches: Vec<OneSwitch>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::message::switch::SwitchState;

    #[test]
    fn test_new_switch_vector() {
        let timestamp = INDITimestamp::from_str("2024-01-01T12:34:56.7").unwrap();
        let vector = NewSwitchVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            timestamp: Some(timestamp.clone()),
            switches: vec![OneSwitch {
                name: "switch1".to_string(),
                value: SwitchState::On,
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.timestamp.unwrap().to_string(), "2024-01-01T12:34:56.7");
        assert_eq!(vector.switches.len(), 1);
        assert_eq!(vector.switches[0].name, "switch1");
        assert_eq!(vector.switches[0].value, SwitchState::On);
    }
}
