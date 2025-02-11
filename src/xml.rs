use crate::property::{Property, PropertyState, PropertyValue, PropertyPerm};
use crate::error::Result;
use std::str::FromStr;

/// Parse attribute from XML string
pub fn parse_attribute(xml: &str, attr: &str) -> Option<String> {
    let attr_str = format!("{}=\"", attr);
    if let Some(attr_pos) = xml.find(&attr_str) {
        let start = attr_pos + attr_str.len();
        if let Some(end) = xml[start..].find('"') {
            return Some(xml[start..start + end].to_string());
        }
    }
    None
}

/// Parse element content from XML string
pub fn parse_element_content(xml: &str, element: &str) -> Option<String> {
    let element_str = format!("<{}>", element);
    if let Some(element_pos) = xml.find(&element_str) {
        let start = element_pos + element_str.len();
        let end = xml[start..]
            .find(&format!("</{}>", element))
            .unwrap_or(xml.len() - start);
        Some(xml[start..start + end].trim().to_string())
    } else {
        None
    }
}

/// Parse element content with attribute from XML string
pub fn parse_element_content_with_attr(xml: &str, element: &str, attr: &str, attr_value: &str) -> Option<String> {
    let element_str = format!("<{} {}=\"{}\">", element, attr, attr_value);
    if let Some(element_pos) = xml.find(&element_str) {
        let start = element_pos + element_str.len();
        let end = xml[start..]
            .find(&format!("</{}>", element))
            .unwrap_or(xml.len() - start);
        Some(xml[start..start + end].trim().to_string())
    } else {
        None
    }
}

/// Parse property from XML string
pub fn parse_property(xml: &str) -> Option<Property> {
    let device = parse_attribute(xml, "device")?;
    let name = parse_attribute(xml, "name")?;
    let state = parse_attribute(xml, "state")
        .map(|s| PropertyState::from_str(&s).unwrap_or(PropertyState::Idle))
        .unwrap_or(PropertyState::Idle);
    let perm = parse_attribute(xml, "perm")
        .map(|s| PropertyPerm::from_str(&s).unwrap_or(PropertyPerm::ReadWrite))
        .unwrap_or(PropertyPerm::ReadWrite);

    // Parse property value
    let value = if xml.contains("<oneText") {
        parse_element_content(xml, "oneText")
            .map(PropertyValue::Text)
    } else if xml.contains("<oneNumber") {
        parse_element_content(xml, "oneNumber")
            .and_then(|s| s.parse::<f64>().ok())
            .map(|n| PropertyValue::Number(n, None))
    } else if xml.contains("<oneSwitch") {
        // For CONNECTION property, look for CONNECT switch
        if name == "CONNECTION" {
            parse_element_content_with_attr(xml, "oneSwitch", "name", "CONNECT")
                .map(|s| PropertyValue::Switch(s == "On"))
        } else {
            parse_element_content(xml, "oneSwitch")
                .map(|s| PropertyValue::Switch(s == "On"))
        }
    } else if xml.contains("<oneLight") {
        parse_element_content(xml, "oneLight")
            .and_then(|s| PropertyState::from_str(&s).ok())
            .map(PropertyValue::Light)
    } else {
        Some(PropertyValue::Text("".to_string()))
    }?;

    Some(Property::new(device, name, value, state, perm))
}
