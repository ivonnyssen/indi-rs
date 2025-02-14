//! INDI Protocol Client Implementation
//!
//! This module provides the client implementation for the INDI protocol.
//! It handles connecting to INDI servers, sending commands, and receiving responses.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream as AsyncTcpStream;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::error::{Error, Result};
use crate::message::{Message, OneSwitch};
use crate::property::{Property, PropertyPerm, PropertyState, PropertyValue, SwitchState};

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

    /// Update state with a message
    pub fn update(&mut self, message: &Message) {
        match message {
            Message::DefProperty(prop) => {
                debug!("Got property definition for device '{}', property '{}'", prop.device, prop.name);
                let device_props = self.devices.entry(prop.device.clone()).or_default();
                device_props.insert(prop.name.clone(), prop.clone());
            }
            Message::DefSwitchVector { device, name, state: prop_state, perm, switches, .. } => {
                debug!("Got switch vector for device '{}', property '{}'", device, name);
                debug!("Switches: {:?}", switches);
                let device_props = self.devices.entry(device.clone()).or_default();

                // Create parent property with switches
                let prop = Property::new(
                    device.clone(),
                    name.clone(),
                    PropertyValue::SwitchVector(
                        switches.iter().map(|s| {
                            (
                                s.name.clone(),
                                if s.value.trim() == "On" { SwitchState::On } else { SwitchState::Off }
                            )
                        }).collect()
                    ),
                    *prop_state,
                    PropertyPerm::from_str(perm).unwrap_or(PropertyPerm::ReadWrite),
                );
                device_props.insert(name.clone(), prop);
            }
            Message::DelProperty { device } => {
                debug!("Got delete property for device '{}'", device);
                self.devices.remove(device);
            }
            _ => {
                debug!("Ignoring message: {:?}", message);
            }
        }
    }
}

/// INDI client
#[derive(Debug)]
pub struct Client {
    config: ClientConfig,
    state: Arc<Mutex<ClientState>>,
    sender: broadcast::Sender<Message>,
    stream: Arc<Mutex<Option<AsyncTcpStream>>>,
}

impl Client {
    /// Create new client
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let (sender, _receiver) = broadcast::channel(32);
        let state = Arc::new(Mutex::new(ClientState::new()));

        Ok(Client {
            config,
            state,
            sender,
            stream: Arc::new(Mutex::new(None)),
        })
    }

    /// Connect to the INDI server
    pub async fn connect(&self) -> Result<()> {
        let stream = AsyncTcpStream::connect(&self.config.server_addr).await?;
        *self.stream.lock().await = Some(stream);

        // Spawn connection handler task
        let state = self.state.clone();
        let stream = self.stream.clone();
        let receiver = self.sender.subscribe();

        tokio::spawn(async move {
            if let Err(e) = Self::connection_task(receiver, stream, state).await {
                error!("Connection task error: {}", e);
            }
        });

        Ok(())
    }

    /// Get properties from the INDI server
    pub async fn get_properties(&mut self, device: Option<&str>, name: Option<&str>) -> Result<()> {
        let message = Message::get_properties(
            "1.7",
            device.map(|s| s.to_string()),
            name.map(|s| s.to_string()),
        );
        self.sender
            .send(message)
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
        name: &str,
        values: &[(String, PropertyValue)],
    ) -> Result<()> {
        debug!("Setting property array {}@{} to {:?}", device, name, values);

        // For CONNECTION property, we need to send a special message
        if name == "CONNECTION" {
            let mut switches = HashMap::new();
            for (element_name, value) in values {
                if let PropertyValue::Switch(state) = value {
                    switches.insert(element_name.clone(), *state);
                }
            }
            let prop = Property::new(
                device.to_string(),
                name.to_string(),
                PropertyValue::SwitchVector(switches),
                PropertyState::Ok,
                PropertyPerm::ReadWrite,
            );
            let message = Message::NewProperty(prop);
            self.write_message(&message).await?;
            return Ok(());
        }

        let mut props = Vec::new();
        for (element_name, value) in values {
            match value {
                PropertyValue::Switch(state) => {
                    let prop = Property::new_with_value(
                        device.to_string(),
                        name.to_string(),
                        element_name.to_string(),
                        PropertyValue::Switch(*state),
                        PropertyState::Ok, // Set state to Ok to indicate we're actively changing it
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
        let array_prop = Property::new_with_elements(
            device.to_string(),
            name.to_string(),
            props,
            PropertyState::Ok, // Set state to Ok to indicate we're actively changing it
            PropertyPerm::ReadWrite,
        );

        let message = Message::NewProperty(array_prop);
        self.write_message(&message).await?;
        Ok(())
    }

    /// Set a switch vector property for a device
    pub async fn set_switch_vector(&self, device: &str, name: &str, switches: &[(String, SwitchState)]) -> Result<()> {
        debug!(
            "Sending switch vector for device '{}', property '{}', switches: {:?}",
            device, name, switches
        );
        let message = Message::NewSwitchVector {
            device: device.to_string(),
            name: name.to_string(),
            state: PropertyState::Ok,
            switches: switches
                .iter()
                .map(|(name, state)| OneSwitch {
                    name: name.clone(),
                    value: state.to_string(),
                })
                .collect(),
        };
        self.send_message(message)?;
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

    /// Send a message to the INDI server
    pub fn send_message(&self, message: Message) -> Result<()> {
        self.sender
            .send(message)
            .map_err(|_| Error::Message("Failed to send message".to_string()))?;
        Ok(())
    }

    /// Write message to stream
    async fn write_message(&self, message: &Message) -> Result<()> {
        self.send_message(message.clone())?;
        Ok(())
    }

    /// Parse XML stream and return complete messages
    fn try_parse_xml(xml_buffer: &str) -> (Option<Message>, bool) {
        // Trim whitespace
        let xml_buffer = xml_buffer.trim();
        if xml_buffer.is_empty() {
            return (None, false);
        }

        // Check if it's a complete message
        let mut depth = 0;
        let mut in_tag = false;
        let mut chars = xml_buffer.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '<' => {
                    if !in_tag {
                        in_tag = true;
                        if chars.peek() == Some(&'/') {
                            chars.next(); // consume '/'
                            depth -= 1;
                        } else {
                            depth += 1;
                        }
                    }
                }
                '/' => {
                    if in_tag && chars.peek() == Some(&'>') {
                        chars.next(); // consume '>'
                        depth -= 1;
                        in_tag = false;
                    }
                }
                '>' => {
                    if in_tag {
                        in_tag = false;
                    }
                }
                _ => continue,
            }
        }

        let is_complete = depth == 0 && !in_tag;

        // Try to parse if it's complete
        if is_complete {
            if let Ok(message) = Message::from_str(xml_buffer) {
                return (Some(message), true);
            }
        }

        (None, is_complete)
    }

    /// Connection handler task
    async fn connection_task(
        mut receiver: broadcast::Receiver<Message>,
        stream: Arc<Mutex<Option<AsyncTcpStream>>>,
        state: Arc<Mutex<ClientState>>,
    ) -> Result<()> {
        info!("Starting connection task...");

        // Get stream from mutex
        if stream.lock().await.is_none() {
            return Err(Error::NotConnected);
        }

        let mut stream_guard = stream.lock().await;
        let stream = stream_guard.as_mut().unwrap();
        let (reader, mut writer) = split(stream);
        let mut reader = BufReader::new(reader);
        let mut buffer = String::new();
        let mut xml_buffer = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            debug!("Connection closed by peer");
                            break;
                        }
                        Ok(_) => {
                            xml_buffer.push_str(&buffer);
                            buffer.clear();

                            let (message, is_complete) = Self::try_parse_xml(&xml_buffer);
                            if let Some(message) = message {
                                debug!("Received message: {:?}", message);
                                let mut state = state.lock().await;
                                state.update(&message);
                            }
                            if is_complete {
                                xml_buffer.clear();
                            }
                        }
                        Err(e) => {
                            error!("Error reading from socket: {}", e);
                            break;
                        }
                    }
                }
                msg = receiver.recv() => {
                    match msg {
                        Ok(message) => {
                            debug!("Sending message: {:?}", message);
                            let xml = message.to_xml()?;
                            debug!("Sending XML: {}", xml);
                            writer.write_all(xml.as_bytes()).await?;
                            writer.flush().await?;
                            debug!("Message sent");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            debug!("Sender dropped, closing connection");
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            warn!("Message queue lagged, some messages were dropped");
                            continue;
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
    use tokio::net::TcpListener;

    #[test]
    fn test_try_parse_xml_complete_message() {
        let xml = r#"<defSwitchVector device="CCD Simulator" name="CONNECTION" state="Ok" perm="rw" rule="OneOfMany"><oneSwitch name="CONNECT">On</oneSwitch><oneSwitch name="DISCONNECT">Off</oneSwitch></defSwitchVector>"#;

        let (message, is_complete) = Client::try_parse_xml(xml);
        assert!(is_complete);
        assert!(message.is_some());
        
        if let Some(Message::DefSwitchVector { device, name, switches, .. }) = message {
            assert_eq!(device, "CCD Simulator");
            assert_eq!(name, "CONNECTION");
            assert_eq!(switches.len(), 2);
            assert_eq!(switches[0].name, "CONNECT");
            assert_eq!(switches[0].value, "On");
            assert_eq!(switches[1].name, "DISCONNECT");
            assert_eq!(switches[1].value, "Off");
        } else {
            panic!("Expected DefSwitchVector message");
        }
    }

    #[test]
    fn test_try_parse_xml_incomplete_message() {
        let xml = r#"<defSwitchVector device="CCD Simulator" name="CONNECTION" state="Ok" perm="rw" rule="OneOfMany"><oneSwitch name="CONNECT">On</oneSwitch>"#;

        let (message, is_complete) = Client::try_parse_xml(xml);
        assert!(!is_complete);
        assert!(message.is_none());
    }

    #[test]
    fn test_try_parse_xml_self_closing_tag() {
        let xml = r#"<delProperty device="CCD Simulator"/>"#;

        let (message, is_complete) = Client::try_parse_xml(xml);
        assert!(is_complete);
        assert!(message.is_some());
        
        if let Some(Message::DelProperty { device }) = message {
            assert_eq!(device, "CCD Simulator");
        } else {
            panic!("Expected DelProperty message");
        }
    }

    #[test]
    fn test_try_parse_xml_multiple_messages() {
        let xml = r#"<delProperty device="CCD Simulator"/><defSwitchVector device="CCD Simulator" name="CONNECTION" state="Ok" perm="rw" rule="OneOfMany"><oneSwitch name="CONNECT">On</oneSwitch><oneSwitch name="DISCONNECT">Off</oneSwitch></defSwitchVector>"#;

        // First message
        let (message, is_complete) = Client::try_parse_xml(xml);
        assert!(is_complete);
        assert!(message.is_some());
        
        if let Some(Message::DelProperty { device }) = message {
            assert_eq!(device, "CCD Simulator");
        } else {
            panic!("Expected DelProperty message");
        }

        // Find end of first message and parse second
        let pos = xml.find("/>").unwrap() + 2;
        let remaining = &xml[pos..];
        let (message, is_complete) = Client::try_parse_xml(remaining);
        assert!(is_complete);
        assert!(message.is_some());
        
        if let Some(Message::DefSwitchVector { device, name, switches, .. }) = message {
            assert_eq!(device, "CCD Simulator");
            assert_eq!(name, "CONNECTION");
            assert_eq!(switches.len(), 2);
        } else {
            panic!("Expected DefSwitchVector message");
        }
    }

    #[test]
    fn test_try_parse_xml_malformed() {
        let xml = r#"<defSwitchVector device="CCD Simulator" name="CONNECTION"><badTag>Invalid</badTag>"#;

        let (message, is_complete) = Client::try_parse_xml(xml);
        assert!(!is_complete);
        assert!(message.is_none());
    }

    #[tokio::test]
    async fn test_client_connect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = Client::new(ClientConfig {
            server_addr: format!("127.0.0.1:{}", addr.port()),
        })
        .await
        .unwrap();
        
        client.connect().await.unwrap();
    }

    #[tokio::test]
    async fn test_client_connect_failure() {
        // Create client should succeed since it doesn't connect yet
        let client = Client::new(ClientConfig {
            server_addr: "127.0.0.1:0".to_string(),
        })
        .await
        .unwrap();
        
        // But connecting to a non-existent server should fail
        let result = client.connect().await;
        assert!(result.is_err());
        if let Err(Error::Io(e)) = result {
            assert_eq!(e.kind(), std::io::ErrorKind::ConnectionRefused);
        } else {
            panic!("Expected IO error");
        }
    }

    #[tokio::test]
    async fn test_set_switch_vector() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Create and connect client
        let client = Client::new(ClientConfig {
            server_addr: addr.to_string(),
        })
        .await
        .unwrap();

        // Spawn server task
        let server_handle = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            
            // Read client message
            socket.readable().await.unwrap();
            let n = socket.try_read(&mut buf).unwrap();
            assert!(n > 0);
            
            // Send response immediately
            socket.try_write(b"<setSwitchVector device='CCD Simulator' name='CONNECTION' state='Ok'/>").unwrap();
        });

        // Connect and send message
        client.connect().await.unwrap();

        // Small delay to ensure connection task is running
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let switches = [
            ("CONNECT".to_string(), SwitchState::On),
            ("DISCONNECT".to_string(), SwitchState::Off),
        ];

        client.set_switch_vector("CCD Simulator", "CONNECTION", &switches[..]).await.unwrap();

        // Wait for server to finish
        server_handle.await.unwrap();
    }
}
