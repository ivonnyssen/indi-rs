use crate::error::Result;
use crate::message::definition::{DefNumberVector, DefSwitchVector, DefTextVector};
use crate::property::{Property, PropertyValue, SwitchState};
use std::collections::HashMap;

/// Client state
#[derive(Debug, Default)]
pub struct ClientState {
    /// Properties by device and name
    pub properties: HashMap<String, HashMap<String, Property>>,
    /// Last message received
    pub last_message: Option<crate::message::MessageType>,
}

impl ClientState {
    /// Create a new client state
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            last_message: None,
        }
    }

    /// Get a property by device and name
    pub fn get_property(&self, device: &str, name: &str) -> Option<&Property> {
        self.properties
            .get(device)
            .and_then(|props| props.get(name))
    }

    /// Update state with a text vector definition
    pub fn update_text_vector(&mut self, prop: DefTextVector) -> Result<()> {
        let values = prop.texts.into_iter().map(|t| t.value).collect::<Vec<_>>();
        let value = values.join(","); // Join multiple values with commas

        let property = Property::new(
            prop.device.clone(),
            prop.name.clone(),
            PropertyValue::Text(value),
            prop.state,
            prop.perm,
            prop.timestamp,
        );
        self.update_property(property);
        Ok(())
    }

    /// Update state with a number vector definition
    pub fn update_number_vector(&mut self, prop: DefNumberVector) -> Result<()> {
        let values = prop
            .numbers
            .into_iter()
            .map(|n| n.value.parse::<f64>().unwrap())
            .collect::<Vec<_>>();

        let value = values.first().copied().unwrap_or_default();
        let property = Property::new(
            prop.device.clone(),
            prop.name.clone(),
            PropertyValue::Number(value, None),
            prop.state,
            prop.perm,
            prop.timestamp,
        );
        self.update_property(property);
        Ok(())
    }

    /// Update state with a switch vector definition
    pub fn update_switch_vector(&mut self, prop: DefSwitchVector) -> Result<()> {
        let values = prop
            .switches
            .into_iter()
            .map(|s| s.state)
            .collect::<Vec<_>>();

        let value = values.first().copied().unwrap_or(SwitchState::Off);
        let property = Property::new(
            prop.device.clone(),
            prop.name.clone(),
            PropertyValue::Switch(value),
            prop.state,
            prop.perm,
            prop.timestamp,
        );
        self.update_property(property);
        Ok(())
    }

    /// Update a property in the state
    fn update_property(&mut self, property: Property) {
        let device = property.device.clone();
        let name = property.name.clone();
        self.properties
            .entry(device)
            .or_default()
            .insert(name, property);
    }

    /// Remove a property
    pub fn remove_property(&mut self, device: &str, name: Option<&str>) {
        if let Some(device_props) = self.properties.get_mut(device) {
            if let Some(name) = name {
                device_props.remove(name);
                if device_props.is_empty() {
                    self.properties.remove(device);
                }
            } else {
                self.properties.remove(device);
            }
        }
    }
}
