use serde::{Deserialize, Serialize};

/// Get properties request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProperties {
    /// Device name
    #[serde(rename = "@device")]
    pub device: Option<String>,
    /// Property name
    #[serde(rename = "@name")]
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
