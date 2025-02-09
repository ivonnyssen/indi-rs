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
}

impl FromStr for Message {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let xml = s.trim();
        if xml.starts_with("<getProperties") {
            Ok(Message::GetProperties(xml.to_string()))
        } else if xml.starts_with("<defProperty")
            || xml.starts_with("<defSwitchVector")
            || xml.starts_with("<defTextVector")
            || xml.starts_with("<defNumberVector")
            || xml.starts_with("<defLightVector")
            || xml.starts_with("<defBLOBVector")
        {
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

            // Parse property value based on type
            let value = if xml.contains("<oneText") || xml.contains("<defText") {
                let text = parse_element_content(xml, "oneText")
                    .or_else(|| parse_element_content(xml, "defText"))
                    .unwrap_or_default();
                PropertyValue::Text(text)
            } else if xml.contains("<oneNumber") || xml.contains("<defNumber") {
                let num_str = parse_element_content(xml, "oneNumber")
                    .or_else(|| parse_element_content(xml, "defNumber"))
                    .unwrap_or_default();
                let num = num_str.parse().unwrap_or(0.0);
                PropertyValue::Number(num, None)
            } else if xml.contains("<oneSwitch") || xml.contains("<defSwitch") {
                let switch = parse_element_content(xml, "oneSwitch")
                    .or_else(|| parse_element_content(xml, "defSwitch"))
                    .unwrap_or_default();
                PropertyValue::Switch(switch == "On")
            } else if xml.contains("<oneLight") || xml.contains("<defLight") {
                let state_str = parse_element_content(xml, "oneLight")
                    .or_else(|| parse_element_content(xml, "defLight"))
                    .unwrap_or_default();
                let state = PropertyState::from_str(&state_str).unwrap_or(PropertyState::Idle);
                PropertyValue::Light(state)
            } else if xml.contains("<oneBLOB") || xml.contains("<defBLOB") {
                let format = parse_attribute(xml, "format").unwrap_or_default();
                let size_str = parse_attribute(xml, "size").unwrap_or_default();
                let size: usize = size_str.parse().unwrap_or(0);
                let data_str = parse_element_content(xml, "oneBLOB")
                    .or_else(|| parse_element_content(xml, "defBLOB"))
                    .unwrap_or_default();
                let data = STANDARD.decode(data_str).unwrap_or_default();
                PropertyValue::Blob { format, data, size }
            } else {
                PropertyValue::Text("".to_string())
            };

            Ok(Message::DefProperty(Property::new(
                device, name, value, state, perm,
            )))
        } else if xml.starts_with("<setProperty") {
            Ok(Message::SetProperty(xml.to_string()))
        } else if xml.starts_with("<newProperty") {
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
            let value = parse_element_content(xml, "oneText")
                .map(PropertyValue::Text)
                .or_else(|| {
                    parse_element_content(xml, "oneNumber")
                        .and_then(|s| s.parse().ok())
                        .map(|n| PropertyValue::Number(n, None))
                })
                .or_else(|| {
                    parse_element_content(xml, "oneSwitch")
                        .map(|s| PropertyValue::Switch(s == "On"))
                })
                .or_else(|| {
                    parse_element_content(xml, "oneLight")
                        .and_then(|s| PropertyState::from_str(&s).ok())
                        .map(PropertyValue::Light)
                })
                .unwrap_or_default();

            Ok(Message::NewProperty(Property::new(
                device, name, value, state, perm,
            )))
        } else if xml.starts_with("<message") {
            Ok(Message::Message(xml.to_string()))
        } else {
            Err(Error::ParseError("Unknown message type".into()))
        }
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
