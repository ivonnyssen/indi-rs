use serde::{Deserialize, Serialize};
use crate::property::SwitchState;

/// One switch element used in new and set operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "oneSwitch")]
pub struct OneSwitch {
    /// Name of this switch element
    #[serde(rename = "@name")]
    pub name: String,
    /// Switch state
    #[serde(rename = "$text")]
    pub value: SwitchState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_switch() {
        let switch = OneSwitch {
            name: "test_switch".to_string(),
            value: SwitchState::On,
        };

        assert_eq!(switch.name, "test_switch");
        assert_eq!(switch.value, SwitchState::On);
    }
}
