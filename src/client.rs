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
        let message = Message::get_properties("1.7", None, None);
        self.sender
            .send(message)
            .await
            .map_err(|_| Error::Message("Failed to send message".to_string()))?;
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
    pub async fn set_switch_vector(
        &mut self,
        device: &str,
        name: &str,
        values: &[(String, SwitchState)],
    ) -> Result<()> {
        debug!("Setting switch vector {} for device {}", name, device);

        let mut switches = HashMap::new();
        for (element_name, state) in values {
            switches.insert(element_name.clone(), *state);
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
        let mut xml_buffer = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            debug!("Connection closed by server");
                            break;
                        }
                        Ok(n) => {
                            debug!("Received {} bytes: {}", n, buffer);
                            xml_buffer.push_str(&buffer);
                            buffer.clear(); // Clear the line buffer for next read

                            // Try to parse complete XML messages
                            if xml_buffer.contains("/>") || xml_buffer.contains("</") {
                                debug!("Attempting to parse XML: {}", xml_buffer);
                                match Message::from_str(&xml_buffer) {
                                    Ok(message) => {
                                        debug!("Parsed message: {:?}", message);
                                        match message {
                                            Message::DefProperty(prop) => {
                                                debug!("Got property definition for device '{}', property '{}'", prop.device, prop.name);
                                                let mut state = state.lock().await;
                                                let device_props = state.devices.entry(prop.device.clone()).or_insert_with(HashMap::new);
                                                device_props.insert(prop.name.clone(), prop);
                                                debug!("Updated state after DefProperty: {:?}", *state);
                                            }
                                            Message::DefSwitchVector { device, name, state: prop_state, perm, switches, .. } => {
                                                debug!("Got switch vector for device '{}', property '{}'", device, name);
                                                debug!("Switches: {:?}", switches);
                                                let mut state = state.lock().await;
                                                let device_props = state.devices.entry(device.clone()).or_insert_with(HashMap::new);

                                                // Create parent property with switches
                                                let prop = Property::new_with_value(
                                                    device.clone(),
                                                    name.clone(),
                                                    name.clone(), // Use the property name as the element name
                                                    PropertyValue::SwitchVector(
                                                        switches.iter().map(|s| {
                                                            (
                                                                s.name.clone(),
                                                                if s.value.trim() == "On" { SwitchState::On } else { SwitchState::Off }
                                                            )
                                                        }).collect()
                                                    ),
                                                    prop_state,
                                                    PropertyPerm::from_str(&perm).unwrap_or(PropertyPerm::ReadWrite),
                                                );
                                                device_props.insert(name.clone(), prop);
                                                debug!("Updated state after DefSwitchVector: {:?}", *state);
                                            }
                                            Message::DelProperty { device } => {
                                                debug!("Got delete property for device '{}'", device);
                                                let mut state = state.lock().await;
                                                state.devices.remove(&device);
                                                debug!("Updated state after DelProperty: {:?}", *state);
                                            }
                                            _ => {
                                                debug!("Ignoring message: {:?}", message);
                                            }
                                        }
                                        xml_buffer.clear(); // Clear the XML buffer after successful parse
                                    }
                                    Err(e) => {
                                        debug!("Failed to parse XML: {}", e);
                                        debug!("XML buffer contents: {}", xml_buffer);
                                        // Don't clear buffer on error, might be incomplete message
                                    }
                                }
                            }
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
                            debug!("Sending message: {:?}", message);
                            let xml = message.to_xml()?;
                            debug!("Sending XML: {}", xml);
                            writer.write_all(xml.as_bytes()).await?;
                            writer.write_all(b"\n").await?;
                            writer.flush().await?;
                            debug!("Message sent");
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
            debug!("Server received: {}", line);
            assert!(line.contains("getProperties"));

            // Send mock response with real INDI server response
            let response = r#"<defSwitchVector device="Telescope Simulator" name="CONNECTION" label="Connection" group="Main Control" state="Idle" perm="rw" rule="OneOfMany" timeout="60" timestamp="2025-02-14T00:42:55">
    <defSwitch name="CONNECT" label="Connect">
Off
    </defSwitch>
    <defSwitch name="DISCONNECT" label="Disconnect">
On
    </defSwitch>
</defSwitchVector>
"#;
            debug!("Server sending: {}", response);
            let mut writer = buf_reader.into_inner();
            writer.write_all(response.as_bytes()).await.unwrap();
            writer.write_all(b"\n").await.unwrap();
            writer.flush().await.unwrap();
            debug!("Server sent response");
        });

        // Create client with mock server address
        let config = ClientConfig {
            server_addr: addr.to_string(),
        };

        let client = Client::new(config).await.expect("Failed to create client");
        client.connect().await.expect("Failed to connect");

        // Wait a bit for the message to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Get devices and check
        let devices = client.get_devices().await.expect("Failed to get devices");
        debug!("Found devices: {:?}", devices);
        assert_eq!(
            devices.len(),
            1,
            "Expected 1 device, found {}",
            devices.len()
        );
        assert_eq!(
            devices[0], "Telescope Simulator",
            "Expected Telescope Simulator, found {}",
            devices[0]
        );

        // Get properties and check
        let state = client.state.lock().await;
        debug!("Client state: {:?}", *state);
        drop(state);

        if let Some(props) = client.get_device_properties("Telescope Simulator").await {
            debug!("Found properties for Telescope Simulator: {:?}", props);
            assert_eq!(props.len(), 1, "Expected 1 property, found {}", props.len());
            assert!(
                props.contains_key("CONNECTION"),
                "Expected CONNECTION property"
            );
        } else {
            panic!("No properties found for Telescope Simulator");
        }
    }

    #[tokio::test]
    async fn test_get_properties() {
        // Start test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Create client
        let config = ClientConfig {
            server_addr: addr.to_string(),
        };
        let mut client = Client::new(config).await.unwrap();

        // Test get_properties
        client.get_properties(None, None).await.unwrap();
        client
            .get_properties(Some("CCD Simulator"), None)
            .await
            .unwrap();
        client
            .get_properties(Some("CCD Simulator"), Some("CONNECTION"))
            .await
            .unwrap();

        // Accept connection and verify messages
        let (mut socket, _) = listener.accept().await.unwrap();
        let (reader, _writer) = split(&mut socket);
        let mut reader = BufReader::new(reader);
        let mut buffer = String::new();

        // Read and verify first message (get all properties)
        reader.read_line(&mut buffer).await.unwrap();
        assert!(buffer.contains("getProperties"));
        assert!(buffer.contains("version=\"1.7\""));
        assert!(!buffer.contains("device="));
        assert!(!buffer.contains("name="));
        buffer.clear();

        // Read and verify second message (get device properties)
        reader.read_line(&mut buffer).await.unwrap();
        assert!(buffer.contains("getProperties"));
        assert!(buffer.contains("version=\"1.7\""));
        assert!(buffer.contains("device=\"CCD Simulator\""));
        assert!(!buffer.contains("name="));
        buffer.clear();

        // Read and verify third message (get specific property)
        reader.read_line(&mut buffer).await.unwrap();
        assert!(buffer.contains("getProperties"));
        assert!(buffer.contains("version=\"1.7\""));
        assert!(buffer.contains("device=\"CCD Simulator\""));
        assert!(buffer.contains("name=\"CONNECTION\""));
    }
}
