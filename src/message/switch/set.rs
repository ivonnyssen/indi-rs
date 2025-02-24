use serde::{Deserialize, Serialize};
use crate::property::PropertyState;
use crate::timestamp::INDITimestamp;
use super::one::OneSwitch;

/// Set switch vector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "setSwitchVector")]
pub struct SetSwitchVector {
    /// Device name
    #[serde(rename = "@device")]
    pub device: String,
    /// Property name
    #[serde(rename = "@name")]
    pub name: String,
    /// Property state
    #[serde(rename = "@state")]
    pub state: Option<PropertyState>,
    /// Property timeout
    #[serde(rename = "@timeout")]
    pub timeout: Option<f64>,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    pub timestamp: Option<INDITimestamp>,
    /// Optional message/commentary
    #[serde(rename = "@message")]
    pub message: Option<String>,
    /// Switch elements
    #[serde(rename = "oneSwitch")]
    pub switches: Vec<OneSwitch>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::property::SwitchState;

    #[test]
    fn test_set_switch_vector() {
        let vector = SetSwitchVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            state: None,
            timeout: None,
            timestamp: None,
            message: None,
            switches: vec![OneSwitch {
                name: "switch1".to_string(),
                value: SwitchState::On,
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert!(vector.state.is_none());
        assert!(vector.timeout.is_none());
        assert!(vector.timestamp.is_none());
        assert!(vector.message.is_none());
        assert_eq!(vector.switches.len(), 1);
        assert_eq!(vector.switches[0].name, "switch1");
        assert_eq!(vector.switches[0].value, SwitchState::On);
    }

    #[test]
    fn test_set_switch_vector_with_all_fields() {
        let timestamp = INDITimestamp::from_str("2024-01-01T12:34:56.7").unwrap();
        let vector = SetSwitchVector {
            device: "test_device".to_string(),
            name: "test_name".to_string(),
            state: Some(PropertyState::Ok),
            timeout: Some(1000.0),
            timestamp: Some(timestamp.clone()),
            message: Some("Test message".to_string()),
            switches: vec![OneSwitch {
                name: "switch1".to_string(),
                value: SwitchState::On,
            }],
        };

        assert_eq!(vector.device, "test_device");
        assert_eq!(vector.name, "test_name");
        assert_eq!(vector.state.unwrap(), PropertyState::Ok);
        assert_eq!(vector.timeout.unwrap(), 1000.0);
        assert_eq!(vector.timestamp.unwrap().to_string(), "2024-01-01T12:34:56.7");
        assert_eq!(vector.message.unwrap(), "Test message");
        assert_eq!(vector.switches.len(), 1);
        assert_eq!(vector.switches[0].name, "switch1");
        assert_eq!(vector.switches[0].value, SwitchState::On);
    }
}
