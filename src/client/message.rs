use crate::client::Client;
use crate::error::Result;
use crate::message::definition::{DefNumberVector, DefSwitchVector, DefTextVector};
use crate::message::new::{OneNumber, OneSwitch};
use crate::message::set::{SetNumberVector, SetSwitchVector};
use crate::message::{EnableBLOB, GetProperties, Message, MessageType};
use crate::property::Property;
use quick_xml::de::from_str;
use std::collections::HashMap;
use std::str;
use tokio::io::AsyncWriteExt;

impl Client {
    /// Send a message to the server
    pub async fn send_message(&mut self, message: MessageType) -> Result<()> {
        let xml = message.to_xml()?;
        if let Some(stream) = &mut self.stream {
            stream.write_all(xml.as_bytes()).await?;
            stream.write_all(b"\n").await?;
        }
        Ok(())
    }

    /// Handle a message from the server
    pub async fn handle_message(&mut self, message: Message) -> Result<()> {
        let mut state = self.state.lock().await;
        let content = str::from_utf8(message.content.as_bytes())?;

        // Try to parse the message content as different property types
        if let Ok(prop) = from_str::<DefTextVector>(content) {
            state.update_text_vector(prop)?;
        } else if let Ok(prop) = from_str::<DefNumberVector>(content) {
            state.update_number_vector(prop)?;
        } else if let Ok(prop) = from_str::<DefSwitchVector>(content) {
            state.update_switch_vector(prop)?;
        }

        Ok(())
    }

    /// Get properties from the server
    pub async fn get_properties(&mut self, device: Option<&str>, name: Option<&str>) -> Result<()> {
        let message = MessageType::GetProperties(GetProperties {
            version: "1.7".to_string(),
            device: device.map(|s| s.to_string()),
            name: name.map(|s| s.to_string()),
        });

        self.send_message(message).await
    }

    /// Get devices from the client state
    pub async fn get_devices(&mut self) -> Result<Vec<String>> {
        let state = self.state.lock().await;
        Ok(state.properties.keys().cloned().collect())
    }

    /// Get device properties from the client state
    pub async fn get_device_properties(
        &mut self,
        device: &str,
    ) -> Option<HashMap<String, Property>> {
        let state = self.state.lock().await;
        state.properties.get(device).cloned()
    }

    /// Enable BLOB for a device
    pub async fn enable_blob(&mut self, device: &str, mode: &str) -> Result<()> {
        let message = MessageType::EnableBLOB(EnableBLOB {
            device: device.to_string(),
            name: None,
            value: mode.to_string(),
        });
        self.send_message(message).await
    }

    /// Set switch vector
    pub async fn set_switch_vector(
        &mut self,
        device: &str,
        name: &str,
        switches: Vec<OneSwitch>,
    ) -> Result<()> {
        let message = MessageType::SetSwitchVector(SetSwitchVector {
            device: device.to_string(),
            name: name.to_string(),
            switches,
        });
        self.send_message(message).await
    }

    /// Set number vector
    pub async fn set_number_vector(
        &mut self,
        device: &str,
        name: &str,
        numbers: Vec<OneNumber>,
    ) -> Result<()> {
        let message = MessageType::SetNumberVector(SetNumberVector {
            device: device.to_string(),
            name: name.to_string(),
            numbers,
        });
        self.send_message(message).await
    }
}
