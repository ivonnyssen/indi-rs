//! INDI Protocol Client Implementation
//!
//! This module provides the client implementation for the INDI protocol.
//! It handles connecting to INDI servers, sending commands, and receiving responses.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream as AsyncTcpStream;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};
use crate::message::Message;
use crate::property::{Property, PropertyPerm, PropertyState, PropertyValue};

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
    #[allow(dead_code)] // Used indirectly through Arc<Mutex>
    stream: Arc<Mutex<AsyncTcpStream>>,
    state: Arc<Mutex<ClientState>>,
    sender: mpsc::Sender<Message>,
}

impl Client {
    /// Create new client
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let stream = AsyncTcpStream::connect(&config.server_addr).await?;
        let stream = Arc::new(Mutex::new(stream));
        let state = Arc::new(Mutex::new(ClientState::new()));
        let (sender, receiver) = mpsc::channel(32);

        // Spawn connection handler task
        let state_clone = state.clone();
        let stream_clone = stream.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::connection_task(receiver, stream_clone, state_clone).await {
                error!("Connection task error: {}", e);
            }
        });

        Ok(Self {
            stream,
            state,
            sender,
        })
    }

    /// Connect to INDI server
    pub async fn connect(&self) -> Result<()> {
        info!("Sending initial GetProperties message");
        let message = Message::GetProperties("<getProperties version=\"1.7\"/>".to_string());
        self.sender
            .send(message)
            .await
            .map_err(|_| Error::Message("Failed to send message".to_string()))?;
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
                _ => {
                    return Err(Error::Property(
                        "Only switch properties are supported for array values".to_string(),
                    ))
                }
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
        self.sender
            .send(message.clone())
            .await
            .map_err(|_| Error::Message("Failed to send message".to_string()))?;
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
                            if let Ok(message) = Message::from_str(&buffer) {
                                match message {
                                    Message::DefProperty(prop) => {
                                        debug!("Got property definition: {:?}", prop);
                                        let mut state = state.lock().await;
                                        let device_props = state.devices.entry(prop.device.clone()).or_insert_with(HashMap::new);
                                        device_props.insert(prop.name.clone(), prop);
                                    }
                                    _ => debug!("Ignoring message: {:?}", message),
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
                            writer.write_all(b"\n").await?;
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
