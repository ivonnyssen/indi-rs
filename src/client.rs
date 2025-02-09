//! INDI Protocol Client Implementation
//!
//! This module provides the client implementation for the INDI protocol.
//! It handles connecting to INDI servers, sending commands, and receiving responses.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

use crate::error::{Error, Result};
use crate::message::Message;
use crate::property::{Property, PropertyValue};

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Server address
    pub server_addr: SocketAddr,
}

/// Client state
#[derive(Debug, Default)]
pub struct ClientState {
    /// Device properties
    pub devices: HashMap<String, HashMap<String, Property>>,
}

/// INDI client
#[derive(Debug, Clone)]
pub struct Client {
    /// Client state
    state: Arc<Mutex<ClientState>>,
    /// Message sender
    sender: mpsc::Sender<Message>,
}

impl Client {
    /// Create a new client
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(32);
        let state = Arc::new(Mutex::new(ClientState::default()));

        // Spawn connection handler task
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::connection_task(receiver, config, state_clone).await {
                eprintln!("Connection task error: {}", e);
            }
        });

        Ok(Self { state, sender })
    }

    /// Connect to an INDI server
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        use std::net::ToSocketAddrs;
        let addr = (host, port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| Error::Message("Invalid address".to_string()))?;
        
        let config = ClientConfig {
            server_addr: addr,
        };
        
        Self::new(config).await
    }

    /// Set property value
    pub async fn set_property(
        &self,
        device: &str,
        name: &str,
        value: &PropertyValue,
    ) -> Result<()> {
        // Format property value based on type
        let value_xml = match value {
            PropertyValue::Text(s) => format!("<oneText>{}</oneText>", s),
            PropertyValue::Number(n, _) => format!("<oneNumber>{}</oneNumber>", n),
            PropertyValue::Switch(s) => format!("<oneSwitch>{}</oneSwitch>", s),
            PropertyValue::Light(l) => format!("<oneLight>{}</oneLight>", l),
            PropertyValue::Blob { data, format, size } => format!(
                "<oneBLOB format=\"{}\" size=\"{}\">{}</oneBLOB>",
                format,
                size,
                STANDARD.encode(data)
            ),
        };

        let message = Message::SetProperty(format!(
            "<setProperty device=\"{}\" name=\"{}\">{}</setProperty>",
            device, name, value_xml
        ));

        self.sender
            .send(message)
            .await
            .map_err(|_| Error::Message("Failed to send message: channel closed".to_string()))?;

        Ok(())
    }

    /// Get a device's properties
    pub async fn get_device_properties(&self, device: &str) -> Option<HashMap<String, Property>> {
        let state = self.state.lock().await;
        println!(
            "Client: Getting properties for device {}, current state: {:?}",
            device, state.devices
        );
        state.devices.get(device).cloned()
    }

    /// Get all devices
    pub async fn get_devices(&self) -> Result<Vec<String>> {
        // Send getProperties message to get all devices
        let message = Message::GetProperty("<getProperties version=\"1.7\"/>".to_string());
        self.sender
            .send(message)
            .await
            .map_err(|_| Error::Message("Failed to send message: channel closed".to_string()))?;

        // Wait a bit for the server to respond
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Get current devices
        let state = self.state.lock().await;
        Ok(state.devices.keys().cloned().collect())
    }

    /// Connection handler task
    async fn connection_task(
        mut receiver: mpsc::Receiver<Message>,
        config: ClientConfig,
        state: Arc<Mutex<ClientState>>,
    ) -> Result<()> {
        let stream = TcpStream::connect(config.server_addr).await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            if let Ok(message) = Message::from_xml(&line) {
                                if let Err(e) = Self::handle_message(&state, message).await {
                                    eprintln!("Error handling message: {}", e);
                                }
                            }
                            line.clear();
                        }
                        Err(e) => {
                            eprintln!("Error reading from socket: {}", e);
                            break;
                        }
                    }
                }
                message = receiver.recv() => {
                    match message {
                        Some(message) => {
                            let xml = message.to_xml()?;
                            writer.write_all(xml.as_bytes()).await?;
                            writer.write_all(b"\n").await?;
                            writer.flush().await?;
                        }
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(state: &Arc<Mutex<ClientState>>, message: Message) -> Result<()> {
        let mut state = state.lock().await;

        if let Message::DefProperty(_) = message {
            let device = message.get_device()?;
            let name = message.get_property_name()?;
            let value = message.get_property_value()?;

            let device_props = state.devices.entry(device.clone()).or_default();
            device_props.insert(
                name.clone(),
                Property::new(device, name, value, Default::default(), Default::default()),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_get_device_properties() {
        // Create a TCP listener for the test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();

        // Spawn test server
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }

                if line.contains("getProperties") {
                    println!("Test server: Detected getProperty request");
                    let def_prop = "<defProperty device=\"test_device\" name=\"test_prop\" state=\"Idle\" perm=\"ro\"><oneText>test value</oneText></defProperty>";
                    println!("Test server: Sending defProperty response: {}", def_prop);
                    writer
                        .write_all(def_prop.as_bytes())
                        .await
                        .expect("Failed to write to socket");
                    writer
                        .write_all(b"\n")
                        .await
                        .expect("Failed to write newline to socket"); // Add newline to ensure proper message separation
                    writer.flush().await.expect("Failed to flush socket");
                    println!("Test server: Sent and flushed defProperty response");
                } else if line.contains("setProperty") {
                    println!("Test server: Detected setProperty request");
                    let def_prop = "<defProperty device=\"test_device\" name=\"test_prop\" state=\"Ok\" perm=\"ro\"><oneText>test value</oneText></defProperty>";
                    println!("Test server: Sending defProperty response: {}", def_prop);
                    writer
                        .write_all(def_prop.as_bytes())
                        .await
                        .expect("Failed to write to socket");
                    writer
                        .write_all(b"\n")
                        .await
                        .expect("Failed to write newline to socket"); // Add newline to ensure proper message separation
                    writer.flush().await.expect("Failed to flush socket");
                    println!("Test server: Sent and flushed defProperty response");
                }

                line.clear();
            }
        });

        // Create client
        let client = Client::new(ClientConfig { server_addr })
            .await
            .expect("Failed to create client");

        // Send getProperties message
        let message = Message::GetProperty("<getProperties version=\"1.7\"/>".to_string());
        client
            .sender
            .send(message)
            .await
            .expect("Failed to send message");

        // Wait for response
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check if properties were received
        let props = client
            .get_device_properties("test_device")
            .await
            .expect("No properties found");
        assert_eq!(props.len(), 1);
        assert!(props.contains_key("test_prop"));
    }

    #[tokio::test]
    async fn test_message_parsing() {
        let xml = "<defProperty device=\"test_device\" name=\"test_prop\" state=\"Idle\" perm=\"ro\"><oneText>test value</oneText></defProperty>";
        let message = Message::from_xml(xml).unwrap();
        assert!(matches!(message, Message::DefProperty(_)));
        assert_eq!(message.get_device().unwrap(), "test_device");
        assert_eq!(message.get_property_name().unwrap(), "test_prop");
    }

    #[tokio::test]
    async fn test_invalid_message() {
        let xml = "<invalidTag>test</invalidTag>";
        assert!(Message::from_xml(xml).is_err());
    }
}
