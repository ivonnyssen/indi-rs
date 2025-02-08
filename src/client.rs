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

use crate::error::Error;
use crate::message::Message;
use crate::property::{Property, PropertyPerm, PropertyState, PropertyValue};

type Result<T> = std::result::Result<T, Error>;

/// Default INDI server port
pub const DEFAULT_PORT: u16 = 7624;

/// Client configuration
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

/// Client state
#[derive(Debug, Default)]
struct ClientState {
    /// Whether the client is connected
    connected: bool,
    /// Device properties
    devices: HashMap<String, HashMap<String, Property>>,
}

/// INDI client
#[derive(Debug)]
pub struct Client {
    /// Client configuration
    config: ClientConfig,
    /// Message sender
    sender: mpsc::Sender<Message>,
    /// Client state
    state: Arc<Mutex<ClientState>>,
}

impl Client {
    /// Create a new INDI client
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(32);
        let state = Arc::new(Mutex::new(ClientState::default()));

        let client = Self {
            config,
            sender,
            state: Arc::clone(&state),
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
            .map_err(|e| Error::Message(format!("Failed to send message: {}", e)))?;
        Ok(())
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
    pub async fn set_property(
        &self,
        device: &str,
        name: &str,
        value: &PropertyValue,
    ) -> Result<()> {
        let value_xml = match value {
            PropertyValue::Text(s) => format!("<oneText>{}</oneText>", s),
            PropertyValue::Number(n, _) => format!("<oneNumber>{}</oneNumber>", n),
            PropertyValue::Switch(s) => format!("<oneSwitch>{}</oneSwitch>", s),
            PropertyValue::Light(l) => format!("<oneLight>{}</oneLight>", l),
            PropertyValue::Blob(b) => format!(
                "<oneBLOB>{}</oneBLOB>",
                String::from_utf8_lossy(b).into_owned()
            ),
        };

        let message = Message::SetProperty(format!(
            "<setProperty device=\"{}\" name=\"{}\">{}</setProperty>",
            device, name, value_xml
        ));
        self.send_message(message).await
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
        println!("Client: Connected to server");

        let (reader, mut writer) = tokio::io::split(socket);
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        let mut xml_buffer = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            println!("Client: Connection closed by server");
                            break;
                        }
                        Ok(n) => {
                            println!("Client: Read {} bytes: {:?}", n, line);

                            // Append to XML buffer
                            xml_buffer.push_str(&line);
                            println!("Client: Current XML buffer: {}", xml_buffer);

                            // Check if we have a complete XML message
                            if xml_buffer.contains("<") && xml_buffer.contains(">") {
                                println!("Client: Detected complete XML message: {}", xml_buffer);

                                // Process complete XML message
                                match Message::from_xml(&xml_buffer) {
                                    Ok(msg) => {
                                        println!("Client: Successfully parsed message: {:?}", msg);
                                        // Update device state based on message
                                        if let Message::DefProperty(xml) = &msg {
                                            println!("Client: Processing DefProperty message with XML: {}", xml);
                                            match (msg.get_device(), msg.get_property_name(), msg.get_property_value()) {
                                                (Ok(device), Ok(name), Ok(value)) => {
                                                    println!("Client: Successfully extracted property details: device={device}, name={name}, value={value:?}");
                                                    let property = Property::new(
                                                        device.clone(),
                                                        name.clone(),
                                                        value,
                                                        PropertyState::Idle,
                                                        PropertyPerm::RO,
                                                    );
                                                    let mut state = state.lock().await;
                                                    println!("Client: Updating state with new property");
                                                    state.devices
                                                        .entry(device)
                                                        .or_insert_with(HashMap::new)
                                                        .insert(name, property);
                                                    println!("Client: Updated state: {:?}", state.devices);
                                                }
                                                (Err(e), _, _) => println!("Client: Failed to get device: {}", e),
                                                (_, Err(e), _) => println!("Client: Failed to get property name: {}", e),
                                                (_, _, Err(e)) => println!("Client: Failed to get property value: {}", e),
                                            }
                                        }
                                    }
                                    Err(e) => println!("Client: Failed to parse message: {}", e),
                                }

                                // Clear buffer for next message
                                xml_buffer.clear();
                            }

                            // Clear line buffer
                            line.clear();
                        }
                        Err(e) => {
                            println!("Client: Error reading from socket: {}", e);
                            break;
                        }
                    }
                }
                Some(message) = receiver.recv() => {
                    println!("Client: Sending message: {:?}", message);
                    let xml = message.to_xml()?;
                    println!("Client: Sending XML: {}", xml);
                    writer.write_all(xml.as_bytes()).await?;
                    writer.write_all(b"\n").await?;  // Add newline to ensure proper message separation
                    writer.flush().await?;
                    println!("Client: Successfully sent message");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;

    async fn setup_test_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        println!("Test server: Listening on {}", addr);

        tokio::spawn(async move {
            println!("Test server: Waiting for connection");
            let (mut socket, client_addr) = listener.accept().await.unwrap();
            println!("Test server: Accepted connection from {}", client_addr);

            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        println!("Test server: Connection closed");
                        break;
                    }
                    Ok(n) => {
                        println!("Test server: Read {} bytes: {:?}", n, line);

                        // If this is a getProperty message, send back a defProperty
                        if line.contains("getProperty") {
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

                            // Add a longer delay to ensure the client has time to process
                            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
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

                            // Add a longer delay to ensure the client has time to process
                            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        }

                        line.clear();
                    }
                    Err(e) => {
                        println!("Test server error: {}", e);
                        break;
                    }
                }
            }
            println!("Test server: Exiting");
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        println!("Test server setup complete");
        addr
    }

    #[tokio::test]
    async fn test_get_device_properties() {
        let addr = setup_test_server().await;
        let config = ClientConfig { server_addr: addr };
        let client = Client::new(config).await.unwrap();

        // Wait for connection to be established
        let mut retries = 5;
        while !client.is_connected().await && retries > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            retries -= 1;
        }
        assert!(
            client.is_connected().await,
            "Failed to establish connection"
        );

        // Initially no devices
        let props = client.get_device_properties("test_device").await;
        assert!(props.is_none(), "Expected no properties initially");

        // Send a get_property request which should trigger the server to send a DefProperty response
        println!("Test: Sending get_property request");
        client
            .get_property("test_device", "test_prop")
            .await
            .unwrap();

        // Give time for the server response to be processed
        println!("Test: Waiting for response processing");
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Check the client's state directly
        {
            let state = client.state.lock().await;
            println!("Test: Client state after get_property: {:?}", state.devices);
        }

        // Now we should have the property
        let props = client.get_device_properties("test_device").await;
        assert!(
            props.is_some(),
            "Expected properties to be present after receiving DefProperty"
        );
        let props = props.unwrap();
        assert!(
            props.contains_key("test_prop"),
            "Expected test_prop to be present in properties"
        );

        // Print the actual property value
        if let Some(prop) = props.get("test_prop") {
            println!("Test: Found property: {:?}", prop);
            match &prop.value {
                PropertyValue::Text(text) => assert_eq!(text, "test value"),
                _ => panic!("Expected text property"),
            }
        }
    }

    #[tokio::test]
    async fn test_message_parsing() {
        let xml = "<getProperty device=\"test_device\" name=\"test_prop\"/>";
        let msg = Message::from_xml(xml).unwrap();
        assert!(matches!(msg, Message::GetProperty(_)));
        assert_eq!(msg.get_device().unwrap(), "test_device");
        assert_eq!(msg.get_property_name().unwrap(), "test_prop");
    }

    #[tokio::test]
    async fn test_invalid_message() {
        let xml = "Invalid XML<>";
        assert!(Message::from_xml(xml).is_err());
    }
}
