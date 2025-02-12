//! INDI Protocol Client Implementation
//!
//! This module provides the client implementation for the INDI protocol.
//! It handles connecting to INDI servers, sending commands, and receiving responses.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::TcpStream as AsyncTcpStream;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};
use crate::message::Message;
use crate::property::{Property, PropertyValue, PropertyState, PropertyPerm};

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
    stream: Arc<Mutex<AsyncTcpStream>>,
}

impl Client {
    /// Create new client
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(32);
        let state = Arc::new(Mutex::new(ClientState::new()));
        let stream = Arc::new(Mutex::new(AsyncTcpStream::connect(&config.server_addr).await?));

        // Spawn connection handler task
        let state_clone = state.clone();
        let stream_clone = stream.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::connection_task(receiver, stream_clone, state_clone).await {
                error!("Connection task error: {}", e);
            }
        });

        Ok(Self {
            state,
            sender,
            stream,
        })
    }

    /// Connect to INDI server
    pub async fn connect(&self) -> Result<()> {
        info!("Sending initial GetProperties message");
        let message = Message::GetProperties("<getProperties version=\"1.7\"/>".to_string());
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
        let prop = Property::new(
            device.to_string(),
            name.to_string(),
            value.clone(),
            PropertyState::Idle,
            PropertyPerm::ReadWrite,
        );
        
        let message = Message::NewProperty(prop);
        self.write_message(&message).await?;
        Ok(())
    }

    /// Set a property array value for a device
    pub async fn set_property_array(
        &self,
        device: &str,
        property: &str,
        values: &[(String, PropertyValue)],
    ) -> Result<()> {
        // Create a vector of properties
        let mut props = Vec::new();
        for (name, value) in values {
            match value {
                PropertyValue::Switch(_) => {
                    let prop = Property::new(
                        device.to_string(),
                        name.to_string(),
                        value.clone(),
                        PropertyState::Idle,
                        PropertyPerm::ReadWrite,
                    );
                    props.push(prop);
                }
                _ => return Err(Error::Property(
                    "Only switch properties are supported for array values".to_string()
                )),
            }
        }
        
        // Create a new property to hold the array
        let array_prop = Property::new(
            device.to_string(),
            property.to_string(),
            PropertyValue::Switch(false), // Placeholder value
            PropertyState::Idle,
            PropertyPerm::ReadWrite,
        );
        
        let message = Message::NewProperty(array_prop);
        self.write_message(&message).await?;
        Ok(())
    }

    /// Get all devices
    pub async fn get_devices(&self) -> Result<Vec<String>> {
        // Send getProperties message to get all devices
        let message = Message::GetProperties(String::new());
        self.write_message(&message).await?;

        // TODO: Wait for response and parse devices
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

    /// Send message to stream
    async fn send_message(&self, message: &str) -> Result<()> {
        let mut stream = self.stream.lock().await;
        stream.write_all(message.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;
        Ok(())
    }

    /// Connection handler task
    async fn connection_task(
        mut receiver: mpsc::Receiver<Message>,
        stream: Arc<Mutex<AsyncTcpStream>>,
        state: Arc<Mutex<ClientState>>,
    ) -> Result<()> {
        info!("Starting connection task...");

        let mut stream_guard = stream.lock().await;
        let (reader, mut writer) = split(&mut *stream_guard);
        let mut reader = BufReader::new(reader);
        let mut buffer = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            debug!("Connection closed by server");
                            break;
                        }
                        Ok(_) => {
                            debug!("Received message {}", buffer);
                            // Parse XML message
                            if buffer.contains("<defSwitchVector") {
                                if let Some(device) = Self::extract_attribute(&buffer, "device") {
                                    if let Some(name) = Self::extract_attribute(&buffer, "name") {
                                        if let Some(state_str) = Self::extract_attribute(&buffer, "state") {
                                            debug!("Device {} property {} state {}", device, name, state_str);
                                            // Update property state
                                            let mut state = state.lock().await;
                                            if let Some(device_props) = state.devices.get_mut(&device) {
                                                if let Some(prop) = device_props.get_mut(&name) {
                                                    prop.state = match state_str.as_str() {
                                                        "Idle" => PropertyState::Idle,
                                                        "Ok" => PropertyState::Ok,
                                                        "Busy" => PropertyState::Busy,
                                                        "Alert" => PropertyState::Alert,
                                                        _ => PropertyState::Idle,
                                                    };
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            buffer.clear();
                        }
                        Err(e) => {
                            warn!("Error reading from socket: {}", e);
                            break;
                        }
                    }
                }
                msg = receiver.recv() => {
                    match msg {
                        Some(message) => {
                            debug!("Got message: {:?}", message);
                            let xml = message.to_xml()?;
                            writer.write_all(xml.as_bytes()).await?;
                            writer.flush().await?;
                        }
                        None => {
                            debug!("Channel closed");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract attribute value from XML string
    fn extract_attribute(xml: &str, attr: &str) -> Option<String> {
        if let Some(start) = xml.find(&format!("{}=\"", attr)) {
            let start = start + attr.len() + 2;
            if let Some(end) = xml[start..].find('\"') {
                return Some(xml[start..start + end].to_string());
            }
        }
        None
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
