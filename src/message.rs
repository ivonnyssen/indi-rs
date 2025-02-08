use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, multispace0};
use nom::multi::many0;
use nom::sequence::delimited;
use nom::branch::alt;
use nom::Parser;
use nom::IResult;

use crate::error::{Error, Result};
use crate::property::{PropertyPerm, PropertyState, PropertyValue};

#[derive(Debug, Clone)]
struct XmlAttribute {
    name: String,
    value: String,
}

fn parse_attribute(input: &str) -> IResult<&str, XmlAttribute> {
    // Skip leading whitespace
    let (input, _) = multispace0.parse(input)?;

    // Parse attribute name
    let (input, name) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;

    // Skip whitespace and equals sign
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char('=').parse(input)?;
    let (input, _) = multispace0.parse(input)?;

    // Parse attribute value in quotes
    let (input, value) = delimited(char('"'), take_while1(|c| c != '"'), char('"')).parse(input)?;

    Ok((
        input,
        XmlAttribute {
            name: name.to_string(),
            value: value.to_string(),
        },
    ))
}

fn parse_attributes(input: &str) -> IResult<&str, Vec<XmlAttribute>> {
    many0(parse_attribute).parse(input)
}

fn parse_element_start(input: &str) -> IResult<&str, (String, Vec<XmlAttribute>)> {
    // Skip leading whitespace and opening angle bracket
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = char('<').parse(input)?;

    // Parse element name
    let (input, name) = take_while1(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;

    // Parse attributes
    let (input, attrs) = parse_attributes(input)?;

    // Skip closing angle bracket or self-closing tag
    let (input, _) = multispace0.parse(input)?;
    let (input, _) = alt((tag("/>"), tag(">"))).parse(input)?;

    Ok((input, (name.to_string(), attrs)))
}

fn parse_element_content(input: &str) -> IResult<&str, String> {
    // Parse content until closing angle bracket
    let (input, content) = take_while1(|c| c != '<').parse(input)?;

    Ok((input, content.to_string()))
}

/// Message types supported by the INDI protocol
#[derive(Debug, Clone)]
pub enum Message {
    /// Get property definition
    GetProperty(String),
    /// Define property
    DefProperty(String),
    /// Set property
    SetProperty(String),
    /// New property
    NewProperty(String),
    /// Define a device
    DefDevice(String),
    /// Message from device
    Message(String),
}

impl Message {
    /// Parse XML message into Message enum
    pub fn from_xml(xml: &str) -> Result<Self> {
        let (_, (name, _)) = parse_element_start(xml).map_err(|e| Error::Message(e.to_string()))?;

        match name.as_str() {
            "getProperties" => Ok(Message::GetProperty(xml.to_string())),
            "defProperty" => Ok(Message::DefProperty(xml.to_string())),
            "setProperty" => Ok(Message::SetProperty(xml.to_string())),
            "newProperty" => Ok(Message::NewProperty(xml.to_string())),
            "defDevice" => Ok(Message::DefDevice(xml.to_string())),
            "message" => Ok(Message::Message(xml.to_string())),
            _ => Err(Error::Message(format!("Invalid message type: {}", name))),
        }
    }

    /// Convert Message enum to XML string
    pub fn to_xml(&self) -> Result<String> {
        match self {
            Message::GetProperty(xml) => Ok(xml.clone()),
            Message::DefProperty(xml) => Ok(xml.clone()),
            Message::SetProperty(xml) => Ok(xml.clone()),
            Message::NewProperty(xml) => Ok(xml.clone()),
            Message::DefDevice(xml) => Ok(xml.clone()),
            Message::Message(xml) => Ok(xml.clone()),
        }
    }

    /// Get device name from message
    pub fn get_device(&self) -> Result<String> {
        let xml = self.to_xml()?;
        let (_, (_, attrs)) = parse_element_start(&xml).map_err(|e| Error::Message(e.to_string()))?;

        attrs
            .iter()
            .find(|attr| attr.name == "device")
            .map(|attr| attr.value.clone())
            .ok_or_else(|| Error::Message("Missing device attribute".to_string()))
    }

    /// Get property name from message
    pub fn get_property_name(&self) -> Result<String> {
        let xml = self.to_xml()?;
        let (_, (_, attrs)) = parse_element_start(&xml).map_err(|e| Error::Message(e.to_string()))?;

        attrs
            .iter()
            .find(|attr| attr.name == "name")
            .map(|attr| attr.value.clone())
            .ok_or_else(|| Error::Message("Missing name attribute".to_string()))
    }

    /// Get property value from message
    pub fn get_property_value(&self) -> Result<PropertyValue> {
        let xml = self.to_xml()?;
        let (input, (_, _)) = parse_element_start(&xml).map_err(|e| Error::Message(e.to_string()))?;

        // Parse value element
        let (input, (value_type, attrs)) =
            parse_element_start(input).map_err(|e| Error::Message(e.to_string()))?;
        let (_, value) = parse_element_content(input).map_err(|e| Error::Message(e.to_string()))?;

        match value_type.as_str() {
            "oneText" => Ok(PropertyValue::Text(value)),
            "oneNumber" => Ok(PropertyValue::Number(
                value.parse().map_err(|_| {
                    Error::Message(format!("Invalid number value: {}", value))
                })?,
                None,
            )),
            "oneSwitch" => Ok(PropertyValue::Switch(value == "On")),
            "oneLight" => Ok(PropertyValue::Light(value.parse()?)),
            "oneBLOB" => {
                let format = attrs
                    .iter()
                    .find(|attr| attr.name == "format")
                    .map(|attr| attr.value.clone())
                    .unwrap_or_default();
                let data = STANDARD
                    .decode(value.trim())
                    .map_err(|_| Error::Message("Invalid base64 data".to_string()))?;
                let size = data.len();
                Ok(PropertyValue::Blob {
                    data,
                    format,
                    size,
                })
            }
            _ => Err(Error::Message(format!("Invalid value type: {}", value_type))),
        }
    }

    /// Get property state from message
    pub fn get_property_state(&self) -> Result<PropertyState> {
        let xml = self.to_xml()?;
        let (_, (_, attrs)) = parse_element_start(&xml).map_err(|e| Error::Message(e.to_string()))?;

        attrs
            .iter()
            .find(|attr| attr.name == "state")
            .map(|attr| attr.value.parse())
            .unwrap_or_else(|| Ok(PropertyState::default()))
    }

    /// Get property permission from message
    pub fn get_property_perm(&self) -> Result<PropertyPerm> {
        let xml = self.to_xml()?;
        let (_, (_, attrs)) = parse_element_start(&xml).map_err(|e| Error::Message(e.to_string()))?;

        attrs
            .iter()
            .find(|attr| attr.name == "perm")
            .map(|attr| attr.value.parse())
            .unwrap_or_else(|| Ok(PropertyPerm::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_parsing() {
        let xml = "<getProperties version=\"1.7\"/>";
        let message = Message::from_xml(xml).unwrap();
        assert!(matches!(message, Message::GetProperty(_)));
    }

    #[test]
    fn test_message_serialization() {
        let xml = "<getProperties version=\"1.7\"/>";
        let message = Message::GetProperty(xml.to_string());
        assert_eq!(message.to_xml().unwrap(), xml);
    }

    #[test]
    fn test_property_value_parsing() {
        let xml = "<defProperty device=\"test\" name=\"test\"><oneText>test value</oneText></defProperty>";
        let message = Message::DefProperty(xml.to_string());
        let value = message.get_property_value().unwrap();
        assert!(matches!(value, PropertyValue::Text(_)));
        if let PropertyValue::Text(text) = value {
            assert_eq!(text, "test value");
        }
    }
}
