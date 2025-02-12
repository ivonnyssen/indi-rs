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

use quick_xml::events::{BytesStart, BytesText};
use quick_xml::Writer;

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
        info!(
            "Sending initial GetProperties message: {}",
            message.to_xml()?
        );
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
        let mut writer = Writer::new(Vec::new());
        let elem_name = match value {
            PropertyValue::Switch(_) => "oneSwitch",
            PropertyValue::Text(_) => "oneText",
            PropertyValue::Number(_, _) => "oneNumber",
            PropertyValue::Light(_) => "oneLight",
            PropertyValue::Blob { .. } => "oneBLOB",
        };

        let mut elem = BytesStart::new(elem_name);

        match value {
            PropertyValue::Number(_, format) => {
                elem.push_attribute(("format", format.as_deref().unwrap_or("%f")));
            }
            PropertyValue::Blob { format, size, .. } => {
                elem.push_attribute(("format", format.to_string().as_str()));
                elem.push_attribute(("size", size.to_string().as_str()));
            }
            _ => {}
        }

        writer
            .write_event(quick_xml::events::Event::Start(elem.clone()))
            .unwrap();

        match value {
            PropertyValue::Switch(value) => {
                let content = if *value { "On" } else { "Off" };
                writer
                    .write_event(quick_xml::events::Event::Text(BytesText::new(content)))
                    .unwrap();
            }
            PropertyValue::Text(value) => {
                writer
                    .write_event(quick_xml::events::Event::Text(BytesText::new(value)))
                    .unwrap();
            }
            PropertyValue::Number(value, _) => {
                writer
                    .write_event(quick_xml::events::Event::Text(BytesText::new(
                        &value.to_string(),
                    )))
                    .unwrap();
            }
            PropertyValue::Light(state) => {
                writer
                    .write_event(quick_xml::events::Event::Text(BytesText::new(
                        state.to_string().as_str(),
                    )))
                    .unwrap();
            }
            PropertyValue::Blob { data, .. } => {
                let encoded = STANDARD.encode(data);
                writer
                    .write_event(quick_xml::events::Event::Text(BytesText::new(&encoded)))
                    .unwrap();
            }
        }

        writer
            .write_event(quick_xml::events::Event::End(elem.to_end()))
            .unwrap();
        String::from_utf8(writer.into_inner()).unwrap()
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
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_client() {
        // Create a mock server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn mock server
        tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut buf_reader = tokio::io::BufReader::new(socket);
            let mut line = String::new();

            // Read client message
            buf_reader.read_line(&mut line).await.unwrap();
            assert!(line.contains("getProperties"));

            // Send mock response
            let response = "<defTextVector device=\"MockDevice\" name=\"MockProp\" state=\"Ok\" perm=\"ro\"><defText>test</defText></defTextVector>\n";
            buf_reader
                .into_inner()
                .write_all(response.as_bytes())
                .await
                .unwrap();
        });

        // Create client with mock server address
        let config = ClientConfig {
            server_addr: addr.to_string(),
        };

        let client = Client::new(config).await.expect("Failed to create client");
        client.connect().await.expect("Failed to connect");

        // Wait a bit for the server to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check if we got the mock device
        let devices = client.get_devices().await.expect("Failed to get devices");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0], "MockDevice");

        if let Some(props) = client.get_device_properties("MockDevice").await {
            assert_eq!(props.len(), 1);
            assert!(props.contains_key("MockProp"));
        } else {
            panic!("No properties found for MockDevice");
        }
    }
}
