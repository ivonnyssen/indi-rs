use serde::{Deserialize, Serialize};
use crate::timestamp::INDITimestamp;

/// Get properties request
/// 
/// Request to snoop properties from devices. If no device is specified,
/// properties from all devices will be returned. If no property name is specified,
/// all properties from the specified device(s) will be returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "getProperties")]
pub struct GetProperties {
    /// Device to snoop, or all devices if absent
    #[serde(rename = "@device", skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    /// Property of device to snoop, or all properties if absent
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Delete property command
/// 
/// Delete the given property, or entire device if no property is specified.
/// This command tells a Client that a given Property is no longer available.
/// If only a Device is specified without a Property name, the Client must assume 
/// all Properties for that Device (and the Device itself) are no longer available.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "delProperty")]
pub struct DelProperty {
    /// Required name of Device
    #[serde(rename = "@device")]
    pub device: String,
    /// Name of property to delete. If absent, entire device is deleted
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Moment when this delete was generated
    #[serde(rename = "@timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<INDITimestamp>,
    /// Optional commentary
    #[serde(rename = "@message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::timestamp::INDITimestamp;

    #[test]
    fn test_get_properties_optional_fields() {
        let props = GetProperties {
            device: None,
            name: None,
        };

        assert!(props.device.is_none());
        assert!(props.name.is_none());

        let props_with_device = GetProperties {
            device: Some("test_device".to_string()),
            name: None,
        };

        assert_eq!(props_with_device.device, Some("test_device".to_string()));
        assert!(props_with_device.name.is_none());
    }

    #[test]
    fn test_del_property() {
        let del = DelProperty {
            device: "test_device".to_string(),
            name: None,
            timestamp: None,
            message: None,
        };

        assert_eq!(del.device, "test_device");
        assert!(del.name.is_none());
        assert!(del.timestamp.is_none());
        assert!(del.message.is_none());

        let timestamp = INDITimestamp::from_str("2024-01-01T12:34:56.7").unwrap();
        let del_with_name = DelProperty {
            device: "test_device".to_string(),
            name: Some("test_property".to_string()),
            timestamp: Some(timestamp.clone()),
            message: Some("Property removed".to_string()),
        };

        assert_eq!(del_with_name.device, "test_device");
        assert_eq!(del_with_name.name.unwrap(), "test_property");
        assert_eq!(del_with_name.timestamp.unwrap(), timestamp);
        assert_eq!(del_with_name.message.unwrap(), "Property removed");
    }
}
