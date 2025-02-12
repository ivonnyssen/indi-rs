use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::str::FromStr;

use crate::error::{Error, Result};
use crate::property::{Property, PropertyPerm, PropertyState, PropertyValue};

/// INDI message
#[derive(Debug, Clone)]
pub enum Message {
    /// GetProperties message
    GetProperties(String),
    /// DefProperty message
    DefProperty(Property),
    /// SetProperty message
    SetProperty(String),
    /// NewProperty message
    NewProperty(Property),
    /// Message message
    Message(String),
}

impl Message {
    /// Convert message to XML string
    pub fn to_xml(&self) -> Result<String> {
        match self {
            Message::GetProperties(xml) => Ok(xml.clone()),
            Message::DefProperty(property) => {
                let value_xml = match &property.value {
                    PropertyValue::Switch(value) => format!(
                        "<oneSwitch>{}</oneSwitch>",
                        if *value { "On" } else { "Off" }
                    ),
                    PropertyValue::Text(value) => format!("<oneText>{}</oneText>", value),
                    PropertyValue::Number(value, format) => format!(
                        "<oneNumber format=\"{}\">{}</oneNumber>",
                        format.as_deref().unwrap_or("%f"),
                        value
                    ),
                    PropertyValue::Light(state) => format!("<oneLight>{}</oneLight>", state),
                    PropertyValue::Blob { format, data, size } => format!(
                        "<oneBLOB format=\"{}\" size=\"{}\">{}</oneBLOB>",
                        format,
                        size,
                        STANDARD.encode(data)
                    ),
                };

                Ok(format!(
                    "<defProperty device=\"{}\" name=\"{}\" state=\"{}\" perm=\"{}\">{}</defProperty>",
                    property.device,
                    property.name,
                    property.state,
                    property.perm,
                    value_xml
                ))
            }
            Message::SetProperty(xml) => Ok(xml.clone()),
            Message::NewProperty(property) => {
                let value_xml = match &property.value {
                    PropertyValue::Switch(value) => format!(
                        "<oneSwitch>{}</oneSwitch>",
                        if *value { "On" } else { "Off" }
                    ),
                    PropertyValue::Text(value) => format!("<oneText>{}</oneText>", value),
                    PropertyValue::Number(value, format) => format!(
                        "<oneNumber format=\"{}\">{}</oneNumber>",
                        format.as_deref().unwrap_or("%f"),
                        value
                    ),
                    PropertyValue::Light(state) => format!("<oneLight>{}</oneLight>", state),
                    PropertyValue::Blob { format, data, size } => format!(
                        "<oneBLOB format=\"{}\" size=\"{}\">{}</oneBLOB>",
                        format,
                        size,
                        STANDARD.encode(data)
                    ),
                };

                Ok(format!(
                    "<newProperty device=\"{}\" name=\"{}\" state=\"{}\" perm=\"{}\">{}</newProperty>",
                    property.device,
                    property.name,
                    property.state,
                    property.perm,
                    value_xml
                ))
            }
            Message::Message(msg) => Ok(msg.clone()),
        }
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
            Self::parse_switch_value(xml).unwrap_or_else(|| PropertyValue::Switch(false))
        } else if xml.contains("<defNumberVector") || xml.contains("<newNumberVector") {
            Self::parse_number_value(xml).unwrap_or_else(|| PropertyValue::Number(0.0, None))
        } else if xml.contains("<defLightVector") || xml.contains("<newLightVector") {
            Self::parse_light_value(xml)
                .unwrap_or_else(|| PropertyValue::Light(PropertyState::Idle))
        } else if xml.contains("<defBLOBVector") || xml.contains("<newBLOBVector") {
            Self::parse_blob_value(xml).unwrap_or_else(|| PropertyValue::Blob {
                format: String::new(),
                data: Vec::new(),
                size: 0,
            })
        } else {
            Self::parse_text_value(xml).unwrap_or_else(|| PropertyValue::Text(String::new()))
        }
    }
}

impl FromStr for Message {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let xml = s.trim();

        if xml.starts_with("<getProperties") {
            return Ok(Message::GetProperties(xml.to_string()));
        }

        if xml.starts_with("<setProperty") {
            return Ok(Message::SetProperty(xml.to_string()));
        }

        if xml.starts_with("<message") {
            return Ok(Message::Message(xml.to_string()));
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
        let xml = "<getProperties version='1.7' />";
        let msg = Message::from_str(xml).unwrap();
        assert!(matches!(msg, Message::GetProperties(_)));
    }
}
