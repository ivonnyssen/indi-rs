//! INDI Protocol Client Implementation
//! 
//! This module provides the client implementation for the INDI protocol.
//! It handles connecting to INDI servers, sending commands, and receiving responses.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error};

use crate::error::Error;
use crate::message::Message;
use crate::property::{Property, PropertyValue, PropertyState, PropertyPerm};
use crate::Result;

/// Default INDI server port
pub const DEFAULT_PORT: u16 = 7624;

/// INDI client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Server address
    pub server_addr: SocketAddr,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:7624".parse().unwrap(),
        }
    }
}

/// INDI client state
#[derive(Debug)]
struct ClientState {
    /// Known devices and their properties
    devices: HashMap<String, HashMap<String, Property>>,
    /// Connection status
    connected: bool,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            devices: HashMap::new(),
            connected: false,
        }
    }
}

/// INDI client
#[derive(Debug, Clone)]
pub struct Client {
    /// Client configuration
    config: ClientConfig,
    /// Client state
    state: Arc<Mutex<ClientState>>,
    /// Message sender
    sender: mpsc::Sender<Message>,
}

impl Client {
    /// Create a new INDI client
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(32);
        let state = Arc::new(Mutex::new(ClientState::default()));

        let client = Self {
            config,
            state: Arc::clone(&state),
            sender,
        };

        // Spawn connection handler
        tokio::spawn(Self::connection_task(
            receiver,
            client.config.clone(),
            Arc::clone(&state),
        ));

        Ok(client)
    }

    /// Send a message to the server
    pub async fn send_message(&self, message: Message) -> Result<()> {
        self.sender
            .send(message)
            .await
            .map_err(|_| Error::Message("Failed to send message".to_string()))
    }

    /// Get property from server
    pub async fn get_property(&self, device: &str, name: &str) -> Result<()> {
        let message = Message::GetProperty(format!(
            "<getProperty device=\"{}\" name=\"{}\"/>",
            device, name
        ));
        self.send_message(message).await
    }

    /// Set property value
    pub async fn set_property(&self, device: &str, name: &str, value: PropertyValue) -> Result<()> {
        let value_xml = match value {
            PropertyValue::Text(s) => format!("<oneText>{s}</oneText>"),
            PropertyValue::Number(n, _) => format!("{n}"), // Simple number format for now
            PropertyValue::Switch(b) => format!("<oneSwitch>{}</oneSwitch>", if b { "On" } else { "Off" }),
            PropertyValue::Light(s) => format!("<oneLight>{s}</oneLight>"),
            PropertyValue::Blob(b) => format!("<oneBLOB>{}</oneBLOB>", String::from_utf8_lossy(&b)),
        };

        let message = Message::SetProperty(format!(
            "<setProperty device=\"{}\" name=\"{}\">{}</setProperty>",
            device, name, value_xml
        ));
        self.send_message(message).await
    }

    /// Get a device's properties
    pub async fn get_device_properties(&self, device: &str) -> Option<HashMap<String, Property>> {
        self.state.lock().await.devices.get(device).cloned()
    }

    /// Returns true if the client is connected
    pub async fn is_connected(&self) -> bool {
        self.state.lock().await.connected
    }

    /// Connection handler task
    async fn connection_task(
        mut receiver: mpsc::Receiver<Message>,
        config: ClientConfig,
        state: Arc<Mutex<ClientState>>,
    ) -> Result<()> {
        let socket = TcpStream::connect(config.server_addr).await?;
        state.lock().await.connected = true;
        
        let (reader, mut writer) = tokio::io::split(socket);
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => break, // Connection closed
                        Ok(_) => {
                            // Process incoming message
                            match Message::from_xml(&line) {
                                Ok(msg) => {
                                    debug!("Received message: {:?}", msg);
                                    // Update device state based on message
                                    if let Message::DefProperty(_) = &msg {
                                        if let (Ok(device), Ok(name), Ok(property_value)) = (
                                            msg.get_device(),
                                            msg.get_property_name(),
                                            msg.get_property_value(),
                                        ) {
                                            let property = Property::new(
                                                device.clone(),
                                                name.clone(),
                                                property_value,
                                                PropertyState::Idle,
                                                PropertyPerm::RO,
                                            );
                                            let mut state = state.lock().await;
                                            state.devices
                                                .entry(device)
                                                .or_insert_with(HashMap::new)
                                                .insert(name, property);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to parse message: {}", e);
                                }
                            }
                            line.clear();
                        }
                        Err(e) => {
                            error!("Failed to read from socket: {}", e);
                            break;
                        }
                    }
                }
                msg = receiver.recv() => {
                    match msg {
                        Some(msg) => {
                            // Send message to server
                            match msg.to_xml() {
                                Ok(xml) => {
                                    if let Err(e) = writer.write_all(xml.as_bytes()).await {
                                        error!("Failed to write to socket: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to serialize message: {}", e);
                                }
                            }
                        }
                        None => break, // Channel closed
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
    use tokio::net::TcpListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn setup_test_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0; 1024];
            
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => break, // Connection closed
                    Ok(n) => {
                        socket.write_all(&buf[..n]).await.unwrap();
                    }
                    Err(_) => break,
                }
            }
        });

        addr
    }

    #[tokio::test]
    async fn test_client_connection() {
        let addr = setup_test_server().await;
        
        let config = ClientConfig {
            server_addr: addr,
        };

        let client = Client::new(config).await.unwrap();
        
        // Give the client time to connect
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        assert!(client.is_connected().await);
    }

    #[tokio::test]
    async fn test_send_message() {
        let addr = setup_test_server().await;
        
        let config = ClientConfig {
            server_addr: addr,
        };

        let client = Client::new(config).await.unwrap();
        
        // Give the client time to connect
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let message = Message::Message("Test message".to_string());
        client.send_message(message).await.unwrap();
    }
}
