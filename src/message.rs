//! INDI Protocol Message Implementation
//! 
//! This module provides the message types and parsing functionality for the INDI protocol.
//! Messages are XML-based and follow the INDI protocol specification.

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::str;

use crate::error::Error;
use crate::property::PropertyValue;
use crate::Result;

/// INDI protocol message types
#[derive(Debug, Clone)]
pub enum Message {
    /// Define a device
    DefDevice(String),
    /// Define a property
    DefProperty(String),
    /// Set a property value
    SetProperty(String),
    /// Get a property value
    GetProperty(String),
    /// New property value
    NewProperty(String),
    /// Delete a property
    DelProperty(String),
    /// Message from device
    Message(String),
}

impl Message {
    /// Parse an XML string into a Message
    pub fn from_xml(xml: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut xml_content = String::new();
        let mut in_root = false;
        let mut root_tag = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let tag = str::from_utf8(name.as_ref())?.to_string();
                    
                    // Build XML content with attributes
                    xml_content.push('<');
                    xml_content.push_str(&tag);

                    // Extract attributes
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = str::from_utf8(attr.key.as_ref())?.to_string();
                            let value = str::from_utf8(&attr.value)?.to_string();
                            xml_content.push_str(&format!(" {}=\"{}\"", key, value));
                        }
                    }
                    xml_content.push('>');

                    if !in_root {
                        in_root = true;
                        root_tag = tag;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = str::from_utf8(e.as_ref())?.to_string();
                    xml_content.push_str(&text);
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let tag = str::from_utf8(name.as_ref())?.to_string();
                    xml_content.push_str(&format!("</{}>", tag));

                    if in_root && tag == root_tag {
                        return match root_tag.as_str() {
                            "defDevice" => Ok(Message::DefDevice(xml_content)),
                            "defProperty" => Ok(Message::DefProperty(xml_content)),
                            "setProperty" => Ok(Message::SetProperty(xml_content)),
                            "getProperty" => Ok(Message::GetProperty(xml_content)),
                            "newProperty" => Ok(Message::NewProperty(xml_content)),
                            "delProperty" => Ok(Message::DelProperty(xml_content)),
                            "message" => Ok(Message::Message(xml_content)),
                            _ => Err(Error::Message(format!("Unknown message type: {}", tag))),
                        };
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let tag = str::from_utf8(name.as_ref())?.to_string();
                    
                    // Build XML content with attributes
                    xml_content.push('<');
                    xml_content.push_str(&tag);

                    // Extract attributes
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = str::from_utf8(attr.key.as_ref())?.to_string();
                            let value = str::from_utf8(&attr.value)?.to_string();
                            xml_content.push_str(&format!(" {}=\"{}\"", key, value));
                        }
                    }
                    xml_content.push_str("/>");

                    if !in_root {
                        return match tag.as_str() {
                            "defDevice" => Ok(Message::DefDevice(xml_content)),
                            "defProperty" => Ok(Message::DefProperty(xml_content)),
                            "setProperty" => Ok(Message::SetProperty(xml_content)),
                            "getProperty" => Ok(Message::GetProperty(xml_content)),
                            "newProperty" => Ok(Message::NewProperty(xml_content)),
                            "delProperty" => Ok(Message::DelProperty(xml_content)),
                            "message" => Ok(Message::Message(xml_content)),
                            _ => Err(Error::Message(format!("Unknown message type: {}", tag))),
                        };
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Message(format!("Error parsing XML: {}", e))),
                _ => (),
            }
        }

        Err(Error::Message("Invalid INDI message".to_string()))
    }

    /// Convert a Message to an XML string
    pub fn to_xml(&self) -> Result<String> {
        match self {
            Message::DefDevice(xml) |
            Message::DefProperty(xml) |
            Message::SetProperty(xml) |
            Message::GetProperty(xml) |
            Message::NewProperty(xml) |
            Message::DelProperty(xml) |
            Message::Message(xml) => Ok(xml.to_string()),
        }
    }

    /// Extract device name from XML message
    pub fn get_device(&self) -> Result<String> {
        let xml = match self {
            Message::DefDevice(xml) |
            Message::DefProperty(xml) |
            Message::SetProperty(xml) |
            Message::GetProperty(xml) |
            Message::NewProperty(xml) |
            Message::DelProperty(xml) |
            Message::Message(xml) => xml,
        };

        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        while let Ok(event) = reader.read_event_into(&mut buf) {
            if let Event::Start(ref e) | Event::Empty(ref e) = event {
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        if attr.key.as_ref() == b"device" {
                            let value = str::from_utf8(&attr.value)?.to_string();
                            return Ok(value);
                        }
                    }
                }
            }
        }

        Err(Error::Message("Device name not found in message".to_string()))
    }

    /// Extract property name from XML message
    pub fn get_property_name(&self) -> Result<String> {
        let xml = match self {
            Message::DefDevice(xml) |
            Message::DefProperty(xml) |
            Message::SetProperty(xml) |
            Message::GetProperty(xml) |
            Message::NewProperty(xml) |
            Message::DelProperty(xml) |
            Message::Message(xml) => xml,
        };

        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        while let Ok(event) = reader.read_event_into(&mut buf) {
            if let Event::Start(ref e) | Event::Empty(ref e) = event {
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        if attr.key.as_ref() == b"name" {
                            let value = str::from_utf8(&attr.value)?.to_string();
                            return Ok(value);
                        }
                    }
                }
            }
        }

        Err(Error::Message("Property name not found in message".to_string()))
    }

    /// Extract property value from XML message
    pub fn get_property_value(&self) -> Result<PropertyValue> {
        let xml = match self {
            Message::DefProperty(xml) |
            Message::SetProperty(xml) |
            Message::NewProperty(xml) => xml,
            _ => return Err(Error::Message("Message type does not contain property value".to_string())),
        };

        println!("Parsing XML: {}", xml);
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut in_value = false;
        let mut value_type = None;
        let mut value_text = String::new();

        while let Ok(event) = reader.read_event_into(&mut buf) {
            match event {
                Event::Start(ref e) => {
                    let name = e.name();
                    let tag = str::from_utf8(name.as_ref())?.to_string();
                    println!("Start tag: {}", tag);
                    match tag.as_str() {
                        "oneText" | "oneNumber" | "oneSwitch" | "oneLight" | "oneBLOB" => {
                            in_value = true;
                            value_type = Some(tag.clone());
                            value_text.clear();
                            println!("Found value type: {}", tag);
                        }
                        _ => (),
                    }
                }
                Event::Text(ref e) if in_value => {
                    let text = str::from_utf8(e.as_ref())?;
                    println!("Found text: {}", text);
                    value_text.push_str(text);
                }
                Event::End(ref e) => {
                    let name = e.name();
                    let tag = str::from_utf8(name.as_ref())?.to_string();
                    println!("End tag: {}", tag);
                    if ["oneText", "oneNumber", "oneSwitch", "oneLight", "oneBLOB"].contains(&tag.as_str()) {
                        in_value = false;
                        if let Some(vtype) = &value_type {
                            let text = value_text.trim();
                            println!("Final value: {} (type: {})", text, vtype);
                            if !text.is_empty() {
                                return match vtype.as_str() {
                                    "oneText" => Ok(PropertyValue::Text(text.to_string())),
                                    "oneNumber" => Ok(PropertyValue::Number(
                                        text.parse().map_err(|_| Error::Message("Invalid number".to_string()))?,
                                        None,
                                    )),
                                    "oneSwitch" => Ok(PropertyValue::Switch(text == "On")),
                                    "oneLight" => Ok(PropertyValue::Light(
                                        text.parse().map_err(|_| Error::Message("Invalid light state".to_string()))?,
                                    )),
                                    "oneBLOB" => Ok(PropertyValue::Blob(text.as_bytes().to_vec())),
                                    _ => Err(Error::Message("Unknown property value type".to_string())),
                                };
                            }
                        }
                    }
                }
                Event::Eof => break,
                _ => (),
            }
        }

        Err(Error::Message("Property value not found in message".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_parsing() {
        let xml = r#"<getProperty device="CCD" name="EXPOSURE"/>"#;
        let msg = Message::from_xml(xml).unwrap();
        
        let device = msg.get_device().unwrap();
        assert_eq!(device, "CCD");

        let prop_name = msg.get_property_name().unwrap();
        assert_eq!(prop_name, "EXPOSURE");

        match msg {
            Message::GetProperty(content) => {
                assert!(content.contains("device=\"CCD\""));
                assert!(content.contains("name=\"EXPOSURE\""));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_property_value_parsing() {
        let xml = r#"<newProperty device="CCD" name="EXPOSURE"><oneNumber>1.5</oneNumber></newProperty>"#;
        let msg = Message::from_xml(xml).unwrap();
        
        if let PropertyValue::Number(val, _) = msg.get_property_value().unwrap() {
            assert_eq!(val, 1.5);
        } else {
            panic!("Wrong property value type");
        }

        let xml = r#"<newProperty device="CCD" name="POWER"><oneSwitch>On</oneSwitch></newProperty>"#;
        let msg = Message::from_xml(xml).unwrap();
        
        if let PropertyValue::Switch(val) = msg.get_property_value().unwrap() {
            assert!(val);
        } else {
            panic!("Wrong property value type");
        }
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::GetProperty("<getProperty device=\"CCD\" name=\"EXPOSURE\"/>".to_string());
        let xml = msg.to_xml().unwrap();
        assert!(xml.contains("device=\"CCD\""));
        assert!(xml.contains("name=\"EXPOSURE\""));
    }
}
