//! INDI Protocol Client Implementation
//!
//! This module provides the client implementation for the INDI protocol.
//! It handles connecting to INDI servers, sending commands, and receiving responses.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

use tracing::{debug, error, info};

use crate::error::{Error, Result};
use crate::message::Message;
use crate::property::{Property, PropertyValue};

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Server address
    pub server_addr: String,
}

/// Client state
#[derive(Debug, Default)]
pub struct ClientState {
    /// Connected devices
    pub devices: HashMap<String, HashMap<String, Property>>,
}

impl ClientState {
    /// Create new client state
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }
}

/// INDI client
#[derive(Debug)]
pub struct Client {
    /// Client state
    state: Arc<Mutex<ClientState>>,
    /// Message sender
    sender: mpsc::Sender<Message>,
    /// Stream
    stream: Arc<Mutex<TcpStream>>,
}

impl Client {
    /// Create new client
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(32);
        let state = Arc::new(Mutex::new(ClientState::new()));
        let stream = Arc::new(Mutex::new(TcpStream::connect(&config.server_addr).await?));

        // Spawn connection handler task
        let state_clone = state.clone();
        let stream_clone = stream.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::connection_task(receiver, config, state_clone, stream_clone).await
            {
                error!("Connection task error: {}", e);
            }
        });

        Ok(Self {
            state,
            sender,
            stream,
        })
    }

    /// Connect to an INDI server
    pub async fn connect(&self) -> Result<()> {
        // Send initial GetProperties message
        let message = Message::GetProperties("<getProperties version=\"1.7\"/>".to_string());
        info!("Sending initial GetProperties message: {}", message.to_xml()?);
        self.write_message(&message).await?;
        Ok(())
    }

    /// Set property value
    pub async fn set_property(
        &self,
        device: &str,
        name: &str,
        value: &PropertyValue,
    ) -> Result<()> {
        let value_xml = self.format_property_value(value);

        let message = Message::SetProperty(format!(
            "<setProperty device=\"{}\" name=\"{}\">{}</setProperty>",
            device, name, value_xml
        ));

        self.write_message(&message).await?;
        Ok(())
    }

    /// Format property value as XML
    fn format_property_value(&self, value: &PropertyValue) -> String {
        match value {
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
        }
    }

    /// Get all devices
    pub async fn get_devices(&self) -> Result<Vec<String>> {
        // Send getProperties message to get all devices
        let message = Message::GetProperties("<getProperties version=\"1.7\"/>".to_string());
        self.sender
            .send(message)
            .await
            .map_err(|_| Error::Message("Failed to send message".to_string()))?;

        // Wait for response
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Get devices from state
        let state = self.state.lock().await;
        Ok(state.devices.keys().cloned().collect())
    }

    /// Get properties for a specific device
    pub async fn get_device_properties(&self, device: &str) -> Option<HashMap<String, Property>> {
        let state = self.state.lock().await;
        state.devices.get(device).cloned()
    }

    /// Write message to stream
    async fn write_message(&self, message: &Message) -> Result<()> {
        let xml = message.to_xml()?;
        let mut stream = self.stream.lock().await;
        stream.write_all(xml.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;
        Ok(())
    }

    /// Connection handler task
    async fn connection_task(
        mut receiver: mpsc::Receiver<Message>,
        _config: ClientConfig,
        state: Arc<Mutex<ClientState>>,
        stream: Arc<Mutex<TcpStream>>,
    ) -> Result<()> {
        info!("Starting connection task...");

        // Create reader
        let mut stream_guard = stream.lock().await;
        let (reader, mut writer) = stream_guard.split();
        // We can't drop stream_guard here as the reader/writer still need it

        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line).await {
            if n == 0 {
                break;
            }
            debug!(message = %line, "Received message");
            if let Ok(message) = Message::from_str(&line) {
                let mut state = state.lock().await;
                match &message {
                    Message::DefProperty(property) => {
                        debug!(device = %property.device, "Got DefProperty");
                        let device = property.device.clone();
                        let name = property.name.clone();
                        state
                            .devices
                            .entry(device)
                            .or_default()
                            .insert(name, property.clone());
                    }
                    Message::NewProperty(property) => {
                        debug!(device = %property.device, "Got NewProperty");
                        let device = property.device.clone();
                        let name = property.name.clone();
                        state
                            .devices
                            .entry(device)
                            .or_default()
                            .insert(name, property.clone());
                    }
                    _ => {
                        debug!(message_type = ?message, "Got other message type");
                    }
                }
            }
            line.clear();
        }

        loop {
            tokio::select! {
                // Handle outgoing messages
                Some(message) = receiver.recv() => {
                    let xml = message.to_xml()?;
                    writer.write_all(xml.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                    writer.flush().await?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client() {
        let config = ClientConfig {
            server_addr: "localhost:7624".to_string(),
        };

        let client = Client::new(config).await.expect("Failed to create client");

        // Send getProperties message
        let message = Message::GetProperties("<getProperties version=\"1.7\"/>".to_string());
        client
            .sender
            .send(message)
            .await
            .expect("Failed to send message");
    }
}
