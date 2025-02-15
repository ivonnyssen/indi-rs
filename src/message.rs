use quick_xml::se::Serializer;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::error::{Error, Result};
use crate::property::{Property, PropertyState, SwitchRule};

/// INDI message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Message {
    /// Get properties message
    #[serde(rename = "getProperties")]
    GetProperties {
        /// Version of the INDI protocol
        #[serde(rename = "@version")]
        version: String,
        /// Device to get properties for
        #[serde(rename = "@device", skip_serializing_if = "Option::is_none")]
        device: Option<String>,
        /// Name of the property to get
        #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// Define property message
    #[serde(rename = "defProperty")]
    DefProperty(Property),
    /// Delete property message
    #[serde(rename = "delProperty")]
    DelProperty {
        /// Device name
        #[serde(rename = "@device")]
        device: String,
    },
    /// Set property message
    #[serde(rename = "setProperty")]
    SetProperty {
        /// Raw XML content of the message
        #[serde(rename = "$value")]
        content: String,
    },
    /// New property message
    #[serde(rename = "newProperty")]
    NewProperty(Property),
    /// Message message
    #[serde(rename = "message")]
    Message {
        /// Raw XML content of the message
        #[serde(rename = "$value")]
        content: String,
    },
    /// Define switch vector message
    #[serde(rename = "defSwitchVector")]
    DefSwitchVector(DefSwitchVector),
    /// Define text vector
    #[serde(rename = "defTextVector")]
    DefTextVector {
        /// Device name
        #[serde(rename = "@device")]
        device: String,
        /// Property name
        #[serde(rename = "@name")]
        name: String,
        /// Property label
        #[serde(rename = "@label")]
        label: String,
        /// Property group
        #[serde(rename = "@group")]
        group: String,
        /// Property state
        #[serde(rename = "@state")]
        state: PropertyState,
        /// Property permission
        #[serde(rename = "@perm")]
        perm: String,
        /// Property timeout
        #[serde(rename = "@timeout")]
        timeout: i32,
        /// Property timestamp
        #[serde(rename = "@timestamp")]
        timestamp: String,
        /// Text elements
        #[serde(rename = "defText")]
        texts: Vec<DefText>,
    },
    /// Define number vector
    #[serde(rename = "defNumberVector")]
    DefNumberVector {
        /// Device name
        #[serde(rename = "@device")]
        device: String,
        /// Property name
        #[serde(rename = "@name")]
        name: String,
        /// Property label
        #[serde(rename = "@label")]
        label: String,
        /// Property group
        #[serde(rename = "@group")]
        group: String,
        /// Property state
        #[serde(rename = "@state")]
        state: PropertyState,
        /// Property permission
        #[serde(rename = "@perm")]
        perm: String,
        /// Property timeout
        #[serde(rename = "@timeout")]
        timeout: i32,
        /// Property timestamp
        #[serde(rename = "@timestamp")]
        timestamp: String,
        /// Number elements
        #[serde(rename = "defNumber")]
        numbers: Vec<DefNumber>,
    },
    /// New switch vector message
    #[serde(rename = "newSwitchVector")]
    NewSwitchVector {
        /// Device name
        #[serde(rename = "@device")]
        device: String,
        /// Property name
        #[serde(rename = "@name")]
        name: String,
        /// Property state
        #[serde(rename = "@state")]
        state: PropertyState,
        /// Switch elements
        #[serde(rename = "oneSwitch")]
        switches: Vec<OneSwitch>,
    },
}

/// Switch element in a switch vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneSwitch {
    /// Name of the switch
    #[serde(rename = "@name")]
    pub name: String,
    /// Label for the switch
    #[serde(rename = "@label")]
    pub label: String,
    /// Value of the switch (On/Off)
    #[serde(rename = "$text")]
    pub value: String,
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
    #[serde(rename = "$value")]
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
    #[serde(rename = "$value")]
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
    #[serde(default)]
    pub label: String,
    /// Property group
    #[serde(rename = "@group")]
    #[serde(default)]
    pub group: String,
    /// Property state
    #[serde(rename = "@state")]
    pub state: PropertyState,
    /// Property permission
    #[serde(rename = "@perm")]
    pub perm: String,
    /// Switch rule
    #[serde(rename = "@rule")]
    pub rule: SwitchRule,
    /// Property timeout
    #[serde(rename = "@timeout")]
    #[serde(default)]
    pub timeout: i32,
    /// Property timestamp
    #[serde(rename = "@timestamp")]
    #[serde(default)]
    pub timestamp: String,
    /// Switch elements
    #[serde(rename = "oneSwitch")]
    pub switches: Vec<OneSwitch>,
}

impl DefSwitchVector {
    /// Validates the switch vector according to its rule
    pub fn validate(&self) -> Result<()> {
        let on_count = self.switches.iter().filter(|s| s.value == "On").count();

        match self.rule {
            SwitchRule::OneOfMany if on_count != 1 => Err(Error::Property(format!(
                "OneOfMany rule requires exactly one switch to be On, found {}",
                on_count
            ))),
            SwitchRule::AtMostOne if on_count > 1 => Err(Error::Property(format!(
                "AtMostOne rule allows at most one switch to be On, found {}",
                on_count
            ))),
            _ => Ok(()),
        }
    }
}

impl Message {
    /// Create a new GetProperties message
    pub fn get_properties(
        version: impl Into<String>,
        device: Option<String>,
        name: Option<String>,
    ) -> Self {
        Self::GetProperties {
            version: version.into(),
            device,
            name,
        }
    }

    /// Convert message to XML
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = String::new();
        let mut ser = Serializer::new(&mut writer);
        ser.indent(' ', 4);
        self.serialize(ser)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        Ok(writer)
    }
}

impl FromStr for Message {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Create a cursor over the string data
        let cursor = std::io::Cursor::new(s);
        let reader = std::io::BufReader::new(cursor);
        let mut deserializer = quick_xml::de::Deserializer::from_reader(reader);
        serde_path_to_error::deserialize(&mut deserializer)
            .map_err(|e| Error::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message() {
        // Test GetProperties with device and name
        let msg = Message::get_properties(
            "1.7",
            Some("CCD Simulator".to_string()),
            Some("CONNECTION".to_string()),
        );
        let xml = msg.to_xml().unwrap();
        assert!(xml.contains("version=\"1.7\""));
        assert!(xml.contains("device=\"CCD Simulator\""));
        assert!(xml.contains("name=\"CONNECTION\""));

        // Test parsing the XML back
        let parsed = Message::from_str(&xml).unwrap();
        match parsed {
            Message::GetProperties {
                version,
                device,
                name,
            } => {
                assert_eq!(version, "1.7");
                assert_eq!(device.unwrap(), "CCD Simulator");
                assert_eq!(name.unwrap(), "CONNECTION");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_parse_def_switch_vector() {
        let xml = r#"<defSwitchVector device="Telescope Simulator" name="CONNECTION" label="Connection" group="Main Control" state="Idle" perm="rw" rule="OneOfMany" timeout="60" timestamp="2025-02-14T00:42:55">
<oneSwitch name="CONNECT" label="Connect">Off</oneSwitch>
<oneSwitch name="DISCONNECT" label="Disconnect">On</oneSwitch>
</defSwitchVector>"#;
        let parsed = Message::from_str(xml).unwrap();
        match parsed {
            Message::DefSwitchVector(def_switch) => {
                assert_eq!(def_switch.device, "Telescope Simulator");
                assert_eq!(def_switch.name, "CONNECTION");
                assert_eq!(def_switch.label, "Connection");
                assert_eq!(def_switch.group, "Main Control");
                assert_eq!(def_switch.state, PropertyState::Idle);
                assert_eq!(def_switch.perm, "rw");
                assert_eq!(def_switch.rule, SwitchRule::OneOfMany);
                assert_eq!(def_switch.timeout, 60);
                assert_eq!(def_switch.timestamp, "2025-02-14T00:42:55");
                assert_eq!(def_switch.switches.len(), 2);
                assert_eq!(def_switch.switches[0].name, "CONNECT");
                assert_eq!(def_switch.switches[0].label, "Connect");
                assert_eq!(def_switch.switches[0].value.trim(), "Off");
                assert_eq!(def_switch.switches[1].name, "DISCONNECT");
                assert_eq!(def_switch.switches[1].label, "Disconnect");
                assert_eq!(def_switch.switches[1].value.trim(), "On");
            }
            _ => panic!("Expected DefSwitchVector"),
        }
    }
}
