use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::error::{Error, Result};
use crate::message::Message;
use crate::property::{Property, PropertyState, PropertyValue};

use quick_xml::events::Event;
use quick_xml::Reader;

/// Default INDI server port
pub const DEFAULT_PORT: u16 = 7624;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server address
    pub bind_address: SocketAddr,
    /// Maximum number of clients
    pub max_clients: usize,
    /// Maximum message size
    pub max_message_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: format!("0.0.0.0:{}", DEFAULT_PORT).parse().unwrap(),
            max_clients: 10,
            max_message_size: 1024 * 1024, // 1MB
        }
    }
}

/// Server state
#[derive(Debug, Default)]
pub struct ServerState {
    /// Device properties
    pub properties: HashMap<String, HashMap<String, Property>>,
}

/// INDI server
pub struct Server {
    /// Server configuration
    config: ServerConfig,
    /// Server state
    state: Arc<RwLock<ServerState>>,
}

impl Server {
    /// Create new server
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ServerState::default())),
        }
    }

    /// Starts the server
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting INDI server on {}", self.config.bind_address);

        let listener = TcpListener::bind(&self.config.bind_address)
            .await
            .map_err(Error::Io)?;

        loop {
            let (socket, addr) = listener.accept().await?;
            if let Err(e) = self.handle_client(socket, addr).await {
                error!("Client error: {}", e);
            }
        }
    }

    /// Handle client connection
    async fn handle_client(&mut self, socket: TcpStream, addr: SocketAddr) -> Result<()> {
        info!("New client connected: {}", addr);

        let mut reader = BufReader::new(socket);
        let mut line = String::new();

        while let Ok(n) = reader.read_line(&mut line).await {
            if n == 0 {
                break;
            }

            if let Ok(message) = Message::from_str(&line) {
                debug!(message = ?message, "Received message");
                if let Err(e) = self.handle_message(message).await {
                    error!("Failed to handle message: {}", e);
                }
            }

            line.clear();
        }

        info!("Client disconnected: {}", addr);
        Ok(())
    }

    /// Handle incoming message
    async fn handle_message(&mut self, message: Message) -> Result<()> {
        match message {
            Message::GetProperties(xml) => {
                self.handle_get_properties(&xml).await?;
            }
            Message::SetProperty(xml) => {
                self.handle_set_property(&xml).await?;
            }
            Message::DefProperty(property) => {
                self.handle_def_property(property).await?;
            }
            Message::NewProperty(property) => {
                self.handle_new_property(property).await?;
            }
            Message::Message(msg) => {
                info!("Received message: {}", msg);
            }
        }
        Ok(())
    }

    /// Handle GetProperties message
    async fn handle_get_properties(&mut self, xml: &str) -> Result<()> {
        let device = parse_attribute(xml, "device").unwrap_or_else(|| "*".to_string());

        let state = self.state.read().await;
        if device == "*" {
            // Send all properties
            for (_device, props) in state.properties.iter() {
                for prop in props.values() {
                    let message = Message::DefProperty(prop.clone());
                    if message.to_xml().is_ok() {
                        // Send message
                    }
                }
            }
        } else {
            // Send properties for specific device
            if let Some(props) = state.properties.get(&device) {
                for prop in props.values() {
                    let message = Message::DefProperty(prop.clone());
                    if message.to_xml().is_ok() {
                        // Send message
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle SetProperty message
    async fn handle_set_property(&mut self, xml: &str) -> Result<()> {
        let device = parse_attribute(xml, "device")
            .ok_or_else(|| Error::ParseError("Missing device attribute".into()))?;
        let name = parse_attribute(xml, "name")
            .ok_or_else(|| Error::ParseError("Missing name attribute".into()))?;

        let mut state = self.state.write().await;
        if let Some(props) = state.properties.get_mut(&device) {
            if let Some(prop) = props.get_mut(&name) {
                // Update property value
                let value = parse_element_content(xml, "oneText")
                    .map(PropertyValue::Text)
                    .or_else(|| {
                        parse_element_content(xml, "oneNumber")
                            .and_then(|s| s.parse::<f64>().ok())
                            .map(|n| PropertyValue::Number(n, None))
                    })
                    .or_else(|| {
                        parse_element_content(xml, "oneSwitch")
                            .map(|s| PropertyValue::Switch(s == "On"))
                    })
                    .or_else(|| {
                        parse_element_content(xml, "oneLight")
                            .and_then(|s| PropertyState::from_str(&s).ok())
                            .map(PropertyValue::Light)
                    })
                    .ok_or_else(|| Error::ParseError("Missing property value".into()))?;

                prop.value = value;

                // Send NewProperty message
                let message = Message::NewProperty(prop.clone());
                if message.to_xml().is_ok() {
                    // Send message
                }
            }
        }
        Ok(())
    }

    /// Handle DefProperty message
    async fn handle_def_property(&mut self, property: Property) -> Result<()> {
        let xml = property.to_xml().unwrap();
        let mut reader = Reader::from_str(&xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    debug!(event = ?e, "XML Start event");
                }
                Ok(Event::End(ref e)) => {
                    debug!(event = ?e, "XML End event");
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    error!("Error at position {}: {:?}", reader.buffer_position(), e);
                    break;
                }
                _ => (),
            }
            buf.clear();
        }

        Ok(())
    }

    /// Handle NewProperty message
    async fn handle_new_property(&mut self, property: Property) -> Result<()> {
        let mut state = self.state.write().await;
        let device = property.device.clone();
        let name = property.name.clone();
        state
            .properties
            .entry(device)
            .or_default()
            .insert(name, property);
        Ok(())
    }
}

fn parse_attribute(xml: &str, attr: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                for attribute in e.attributes().flatten() {
                    if attribute.key.as_ref() == attr.as_bytes() {
                        return Some(attribute.unescape_value().unwrap().to_string());
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

fn parse_element_content(xml: &str, element: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut content = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == element.as_bytes() {
                    if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                        content = e.unescape().ok().map(|s| s.to_string());
                        break;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                error!("Error at position {}: {:?}", reader.buffer_position(), e);
                break;
            }
            _ => (),
        }
        buf.clear();
    }

    content
}
