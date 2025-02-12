use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use quick_xml::se::Serializer;
use serde::Serialize;
use std::str::FromStr;

use crate::error::{Error, Result};
use crate::property::{Property, PropertyPerm, PropertyState, PropertyValue};

/// GetProperties message attributes
#[derive(Debug, Clone, Serialize)]
pub struct GetPropertiesMessage {
    /// Protocol version
    #[serde(rename = "@version")]
    /// The version of the protocol being used.
    pub version: String,
    /// Device name (optional)
    #[serde(rename = "@device", skip_serializing_if = "Option::is_none")]
    /// The name of the device being queried.
    pub device: Option<String>,
    /// Property name (optional)
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    /// The name of the property being queried.
    pub name: Option<String>,
}

impl GetPropertiesMessage {
    /// Create a new GetProperties message
    pub fn new(version: impl Into<String>, device: Option<String>, name: Option<String>) -> Self {
        Self {
            version: version.into(),
            device,
            name,
        }
    }

    /// Convert to XML string
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = String::new();
        let ser = Serializer::new(&mut writer);
        self.serialize(ser)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        Ok(writer)
    }
}

/// INDI message
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Message {
    /// GetProperties message
    #[serde(rename = "getProperties")]
    GetProperties {
        /// Raw XML content of the message
        #[serde(rename = "$value")]
        /// The raw XML content of the GetProperties message.
        content: String,
    },
    /// DefProperty message
    #[serde(rename = "defProperty")]
    DefProperty(Property),
    /// SetProperty message
    #[serde(rename = "setProperty")]
    SetProperty {
        /// Raw XML content of the message
        #[serde(rename = "$value")]
        /// The raw XML content of the SetProperty message.
        content: String,
    },
    /// NewProperty message
    #[serde(rename = "newProperty")]
    NewProperty(Property),
    /// Message message
    #[serde(rename = "message")]
    Message {
        /// Raw XML content of the message
        #[serde(rename = "$value")]
        /// The raw XML content of the Message message.
        content: String,
    },
}

impl Message {
    /// Create a new GetProperties message
    pub fn get_properties(
        version: impl Into<String>,
        device: Option<String>,
        name: Option<String>,
    ) -> Self {
        let msg = GetPropertiesMessage::new(version, device, name);
        Self::GetProperties {
            content: msg.to_xml().unwrap_or_default(),
        }
    }

    /// Convert message to XML string
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = String::new();
        let ser = Serializer::new(&mut writer);
        self.serialize(ser)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        Ok(writer)
    }

    /// Parse common property attributes from XML
    fn parse_property_attrs(xml: &str) -> (String, String, PropertyState, PropertyPerm) {
        let device = parse_attribute(xml, "device")
            .or_else(|| parse_attribute(xml, "name"))
            .unwrap_or_default();
        let name = parse_attribute(xml, "name")
            .or_else(|| parse_attribute(xml, "device"))
            .unwrap_or_default();
        let state = parse_attribute(xml, "state")
            .map(|s| PropertyState::from_str(&s).unwrap_or(PropertyState::Idle))
            .unwrap_or(PropertyState::Idle);
        let perm = parse_attribute(xml, "perm")
            .map(|s| PropertyPerm::from_str(&s).unwrap_or(PropertyPerm::ReadWrite))
            .unwrap_or(PropertyPerm::ReadWrite);
        (device, name, state, perm)
    }

    /// Parse a text property value
    fn parse_text_value(xml: &str) -> Option<PropertyValue> {
        parse_element_content(xml, "oneText")
            .or_else(|| parse_element_content(xml, "defText"))
            .map(PropertyValue::Text)
    }

    /// Parse a number property value
    fn parse_number_value(xml: &str) -> Option<PropertyValue> {
        parse_element_content(xml, "oneNumber")
            .or_else(|| parse_element_content(xml, "defNumber"))
            .and_then(|s| s.parse().ok())
            .map(|n| PropertyValue::Number(n, None))
    }

    /// Parse a switch property value
    fn parse_switch_value(xml: &str) -> Option<PropertyValue> {
        parse_element_content(xml, "oneSwitch")
            .or_else(|| parse_element_content(xml, "defSwitch"))
            .map(|s| PropertyValue::Switch(s == "On"))
    }

    /// Parse a light property value
    fn parse_light_value(xml: &str) -> Option<PropertyValue> {
        parse_element_content(xml, "oneLight")
            .or_else(|| parse_element_content(xml, "defLight"))
            .and_then(|s| PropertyState::from_str(&s).ok())
            .map(PropertyValue::Light)
    }

    /// Parse a BLOB property value
    fn parse_blob_value(xml: &str) -> Option<PropertyValue> {
        let format = parse_attribute(xml, "format").unwrap_or_default();
        let size: usize = parse_attribute(xml, "size")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        parse_element_content(xml, "oneBLOB")
            .or_else(|| parse_element_content(xml, "defBLOB"))
            .and_then(|data_str| STANDARD.decode(data_str).ok())
            .map(|data| PropertyValue::Blob { format, data, size })
    }

    /// Parse a property value based on the XML content
    fn parse_property_value(xml: &str) -> PropertyValue {
        if xml.contains("<defSwitchVector") || xml.contains("<newSwitchVector") {
            Self::parse_switch_value(xml).unwrap_or(PropertyValue::Switch(false))
        } else if xml.contains("<defNumberVector") || xml.contains("<newNumberVector") {
            Self::parse_number_value(xml).unwrap_or(PropertyValue::Number(0.0, None))
        } else if xml.contains("<defLightVector") || xml.contains("<newLightVector") {
            Self::parse_light_value(xml).unwrap_or(PropertyValue::Light(PropertyState::Idle))
        } else if xml.contains("<defBLOBVector") || xml.contains("<newBLOBVector") {
            Self::parse_blob_value(xml).unwrap_or(PropertyValue::Blob {
                format: String::new(),
                data: Vec::new(),
                size: 0,
            })
        } else {
            Self::parse_text_value(xml).unwrap_or(PropertyValue::Text(String::new()))
        }
    }
}

impl FromStr for Message {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let xml = s.trim();

        if xml.starts_with("<getProperties") {
            return Ok(Message::GetProperties {
                content: xml.to_string(),
            });
        }

        if xml.starts_with("<setProperty") {
            return Ok(Message::SetProperty {
                content: xml.to_string(),
            });
        }

        if xml.starts_with("<message") {
            return Ok(Message::Message {
                content: xml.to_string(),
            });
        }

        if xml.starts_with("<defProperty")
            || xml.starts_with("<defSwitchVector")
            || xml.starts_with("<defTextVector")
            || xml.starts_with("<defNumberVector")
            || xml.starts_with("<defLightVector")
            || xml.starts_with("<defBLOBVector")
        {
            let (device, name, state, perm) = Message::parse_property_attrs(xml);
            let value = Message::parse_property_value(xml);
            return Ok(Message::DefProperty(Property::new(
                device, name, value, state, perm,
            )));
        }

        if xml.starts_with("<newProperty") {
            let (device, name, state, perm) = Message::parse_property_attrs(xml);
            let value = Message::parse_property_value(xml);
            return Ok(Message::NewProperty(Property::new(
                device, name, value, state, perm,
            )));
        }

        Err(Error::ParseError("Unknown message type".into()))
    }
}

fn parse_attribute(xml: &str, attr: &str) -> Option<String> {
    let attr_str = format!("{}=\"", attr);
    if let Some(attr_pos) = xml.find(&attr_str) {
        let value_start = attr_pos + attr_str.len();
        if let Some(end) = xml[value_start..].find('"') {
            return Some(xml[value_start..value_start + end].to_string());
        }
    }
    None
}

fn parse_element_content(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    if let Some(start) = xml.find(&start_tag) {
        let content_start = start + start_tag.len();
        if let Some(end) = xml[content_start..].find(&end_tag) {
            return Some(xml[content_start..content_start + end].trim().to_string());
        }
    }
    None
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

        // Test GetProperties without device and name
        let msg = Message::get_properties("1.7", None, None);
        let xml = msg.to_xml().unwrap();
        assert!(xml.contains("version=\"1.7\""));
        assert!(!xml.contains("device="));
        assert!(!xml.contains("name="));

        // Test parsing existing XML
        let xml = "<setProperty version=\"1.7\" device=\"CCD Simulator\" name=\"CONNECTION\" />";
        let msg = Message::from_str(xml).unwrap();
        assert!(matches!(msg, Message::SetProperty { content: _ }));

        let xml = "<message>Test message</message>";
        let msg = Message::from_str(xml).unwrap();
        assert!(matches!(msg, Message::Message { content: _ }));
    }
}
