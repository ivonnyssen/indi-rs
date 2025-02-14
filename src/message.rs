use quick_xml::se::Serializer;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::error::{Error, Result};
use crate::property::{Property, PropertyState};

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
    DefSwitchVector {
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
        /// Property rule
        #[serde(rename = "@rule")]
        rule: String,
        /// Property timeout
        #[serde(rename = "@timeout")]
        timeout: i32,
        /// Property timestamp
        #[serde(rename = "@timestamp")]
        timestamp: String,
        /// Switch elements
        #[serde(rename = "defSwitch")]
        switches: Vec<DefSwitch>,
    },
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
}

/// Switch element in a switch vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefSwitch {
    /// Switch name
    #[serde(rename = "@name")]
    pub name: String,
    /// Switch label
    #[serde(rename = "@label")]
    pub label: String,
    /// Switch value
    #[serde(rename = "$value")]
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

impl Message {
    /// Create a new GetProperties message
    pub fn get_properties(version: impl Into<String>, device: Option<String>, name: Option<String>) -> Self {
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
        self.serialize(ser).map_err(|e| Error::SerializationError(e.to_string()))?;
        Ok(writer)
    }
}

impl FromStr for Message {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Create a cursor over the string data
        let cursor = std::io::Cursor::new(s);
        let reader = std::io::BufReader::new(cursor);
        quick_xml::de::from_reader(reader).map_err(|e| Error::SerializationError(e.to_string()))
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
            Message::GetProperties { version, device, name } => {
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
 <defSwitch name="CONNECT" label="Connect">
Off
 </defSwitch>
 <defSwitch name="DISCONNECT" label="Disconnect">
On
 </defSwitch>
</defSwitchVector>"#;
        let parsed = Message::from_str(xml).unwrap();
        match parsed {
            Message::DefSwitchVector { device, name, switches, .. } => {
                assert_eq!(device, "Telescope Simulator");
                assert_eq!(name, "CONNECTION");
                assert_eq!(switches.len(), 2);
                assert_eq!(switches[0].name, "CONNECT");
                assert_eq!(switches[0].value, "Off");
                assert_eq!(switches[1].name, "DISCONNECT");
                assert_eq!(switches[1].value, "On");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
