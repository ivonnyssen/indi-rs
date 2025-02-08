use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0},
    multi::many0,
    sequence::delimited,
    IResult,
};

use crate::error::{Error, Result};
use crate::property::PropertyValue;

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

#[derive(Debug)]
struct XmlAttribute {
    name: String,
    value: String,
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

fn parse_attribute(input: &str) -> IResult<&str, XmlAttribute> {
    let (input, _) = multispace0(input)?;
    let (input, name) = take_while1(is_name_char)(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, value) = delimited(char('"'), take_while1(|c| c != '"'), char('"'))(input)?;

    Ok((
        input,
        XmlAttribute {
            name: name.to_string(),
            value: value.to_string(),
        },
    ))
}

fn parse_attributes(input: &str) -> IResult<&str, Vec<XmlAttribute>> {
    many0(parse_attribute)(input)
}

fn parse_xml_tag(input: &str) -> IResult<&str, (String, Vec<XmlAttribute>, Option<String>)> {
    let (input, _) = char('<')(input)?;
    let (input, tag_name) = take_while1(is_name_char)(input)?;
    let (input, attrs) = parse_attributes(input)?;
    let (input, _) = multispace0(input)?;

    // Handle self-closing tags
    let (input, content) = if input.starts_with("/>") {
        let (input, _) = tag("/>")(input)?;
        (input, None)
    } else {
        let (input, _) = char('>')(input)?;
        let mut content = String::new();
        let mut depth = 1;
        let mut pos = 0;

        // Convert input to chars for easier indexing
        let chars: Vec<char> = input.chars().collect();

        while pos < chars.len() {
            let slice = &input[pos..];
            let end_tag = format!("</{}", tag_name);
            let start_tag = format!("<{}", tag_name);

            if slice.starts_with(&end_tag) {
                depth -= 1;
                if depth == 0 {
                    // Skip to end of tag
                    while pos < chars.len() && chars[pos] != '>' {
                        pos += 1;
                    }
                    pos += 1; // Skip '>'
                    let remaining = &input[pos..];
                    return Ok((remaining, (tag_name.to_string(), attrs, Some(content))));
                }
            } else if slice.starts_with(&start_tag) {
                depth += 1;
            }

            if pos < chars.len() {
                content.push(chars[pos]);
            }
            pos += 1;
        }

        (input, Some(content))
    };

    Ok((input, (tag_name.to_string(), attrs, content)))
}

impl Message {
    /// Parse an XML string into a Message
    pub fn from_xml(xml: &str) -> Result<Self> {
        println!("Parsing XML: {}", xml);
        let (_, (tag_name, attrs, content)) = parse_xml_tag(xml.trim())
            .map_err(|e| Error::Message(format!("Failed to parse XML: {}", e)))?;

        // Reconstruct the XML for storage
        let mut xml_content = format!("<{}", tag_name);
        for attr in &attrs {
            xml_content.push_str(&format!(" {}=\"{}\"", attr.name, attr.value));
        }

        if let Some(content) = content {
            println!("Found content: {}", content);
            xml_content.push('>');
            xml_content.push_str(&content);
            xml_content.push_str("</");
            xml_content.push_str(&tag_name);
            xml_content.push('>');
        } else {
            println!("No content found");
            xml_content.push_str("/>");
        }

        println!("Reconstructed XML: {}", xml_content);
        match tag_name.as_str() {
            "defDevice" => Ok(Message::DefDevice(xml_content)),
            "defProperty" => Ok(Message::DefProperty(xml_content)),
            "setProperty" => Ok(Message::SetProperty(xml_content)),
            "getProperty" => Ok(Message::GetProperty(xml_content)),
            "newProperty" => Ok(Message::NewProperty(xml_content)),
            "delProperty" => Ok(Message::DelProperty(xml_content)),
            "message" => Ok(Message::Message(xml_content)),
            _ => Err(Error::Message(format!(
                "Unknown message type: {}",
                tag_name
            ))),
        }
    }

    /// Convert a Message to an XML string
    pub fn to_xml(&self) -> Result<String> {
        match self {
            Message::DefDevice(xml)
            | Message::DefProperty(xml)
            | Message::SetProperty(xml)
            | Message::GetProperty(xml)
            | Message::NewProperty(xml)
            | Message::DelProperty(xml)
            | Message::Message(xml) => Ok(xml.clone()),
        }
    }

    /// Extract device name from XML message
    pub fn get_device(&self) -> Result<String> {
        let xml = self.to_xml()?;
        let (_, (_, attrs, _)) = parse_xml_tag(&xml)
            .map_err(|e| Error::Message(format!("Failed to parse XML: {}", e)))?;

        attrs
            .iter()
            .find(|attr| attr.name == "device")
            .map(|attr| attr.value.clone())
            .ok_or_else(|| Error::Message("Device attribute not found".to_string()))
    }

    /// Extract property name from XML message
    pub fn get_property_name(&self) -> Result<String> {
        let xml = self.to_xml()?;
        let (_, (_, attrs, _)) = parse_xml_tag(&xml)
            .map_err(|e| Error::Message(format!("Failed to parse XML: {}", e)))?;

        attrs
            .iter()
            .find(|attr| attr.name == "name")
            .map(|attr| attr.value.clone())
            .ok_or_else(|| Error::Message("Name attribute not found".to_string()))
    }

    /// Extract property value from XML message
    pub fn get_property_value(&self) -> Result<PropertyValue> {
        let xml = self.to_xml()?;
        println!("Getting property value from XML: {}", xml);
        let (_, (tag_name, _attrs, content)) = parse_xml_tag(xml.trim())
            .map_err(|e| Error::Message(format!("Failed to parse XML: {}", e)))?;

        // Only certain message types can have property values
        match tag_name.as_str() {
            "defProperty" | "setProperty" | "newProperty" => (),
            _ => {
                return Err(Error::Message(
                    "Message type does not contain property value".to_string(),
                ))
            }
        }

        let content = content.ok_or_else(|| Error::Message("No content found".to_string()))?;
        println!("Found content: {}", content);

        // Parse the value tag inside the content
        let (_, (value_type, attrs, value)) = parse_xml_tag(content.trim())
            .map_err(|e| Error::Message(format!("Failed to parse value XML: {}", e)))?;

        // Get the value, either from content or from empty tag
        let value = match value {
            Some(v) => v.trim().to_string(),
            None => "".to_string(),
        };
        println!("Found value: {}", value);

        match value_type.as_str() {
            "oneText" => Ok(PropertyValue::Text(value)),
            "oneNumber" => Ok(PropertyValue::Number(
                value
                    .parse()
                    .map_err(|_| Error::Message("Invalid number".to_string()))?,
                Some(value_type.to_string()),
            )),
            "oneSwitch" => Ok(PropertyValue::Switch(value == "On")),
            "oneLight" => {
                Ok(PropertyValue::Light(value.parse().map_err(|_| {
                    Error::Message("Invalid light state".to_string())
                })?))
            }
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
            _ => Err(Error::Message(format!(
                "Unknown property value type: {}",
                value_type
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_parsing() {
        let xml = "<getProperty device=\"test_device\" name=\"test_prop\"/>";
        let msg = Message::from_xml(xml).unwrap();
        assert!(matches!(msg, Message::GetProperty(_)));
        assert_eq!(msg.get_device().unwrap(), "test_device");
        assert_eq!(msg.get_property_name().unwrap(), "test_prop");
    }

    #[test]
    fn test_property_value_parsing() {
        let xml = "<newProperty device=\"CCD\" name=\"EXPOSURE\"><oneNumber>1.5</oneNumber></newProperty>";
        let msg = Message::from_xml(xml).unwrap();

        if let PropertyValue::Number(val, _) = msg.get_property_value().unwrap() {
            assert_eq!(val, 1.5);
        } else {
            panic!("Wrong property value type");
        }

        let xml =
            "<newProperty device=\"CCD\" name=\"POWER\"><oneSwitch>On</oneSwitch></newProperty>";
        let msg = Message::from_xml(xml).unwrap();

        if let PropertyValue::Switch(val) = msg.get_property_value().unwrap() {
            assert!(val);
        } else {
            panic!("Wrong property value type");
        }

        // Test BLOB property
        let data = vec![1, 2, 3, 4];
        let xml = format!(
            "<newProperty device=\"CCD\" name=\"IMAGE\"><oneBLOB format=\".fits\" size=\"4\">{}</oneBLOB></newProperty>",
            STANDARD.encode(&data)
        );
        let msg = Message::from_xml(&xml).unwrap();

        if let PropertyValue::Blob {
            data: parsed_data,
            format,
            size,
        } = msg.get_property_value().unwrap()
        {
            assert_eq!(parsed_data, data);
            assert_eq!(format, ".fits");
            assert_eq!(size, 4);
        } else {
            panic!("Wrong property value type");
        }
    }

    #[test]
    fn test_message_serialization() {
        let xml = "<getProperty device=\"test_device\" name=\"test_prop\"/>";
        let msg = Message::from_xml(xml).unwrap();
        assert_eq!(msg.to_xml().unwrap(), xml);
    }
}
