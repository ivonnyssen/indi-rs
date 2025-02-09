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
                debug!("Received message: {:?}", message);
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
                println!("Received message: {}", msg);
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
                    if let Ok(_) = message.to_xml() {
                        // Send message
                    }
                }
            }
        } else {
            // Send properties for specific device
            if let Some(props) = state.properties.get(&device) {
                for prop in props.values() {
                    let message = Message::DefProperty(prop.clone());
                    if let Ok(_) = message.to_xml() {
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
                if let Ok(_) = message.to_xml() {
                    // Send message
                }
            }
        }
        Ok(())
    }

    /// Handle DefProperty message
    async fn handle_def_property(&mut self, property: Property) -> Result<()> {
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
    let attr_str = format!("{}=\"", attr);
    if let Some(attr_pos) = xml.find(&attr_str) {
        let start = attr_pos + attr_str.len();
        if let Some(end) = xml[start..].find('"') {
            return Some(xml[start..start + end].to_string());
        }
    }
    None
}

fn parse_element_content(xml: &str, element: &str) -> Option<String> {
    // Simple XML element content parsing
    let element_str = format!("<{}>", element);
    if let Some(element_pos) = xml.find(&element_str) {
        let start = element_pos + element_str.len();
        let end = xml[start..]
            .find(&format!("</{}>", element))
            .unwrap_or(xml.len() - start);
        Some(xml[start..start + end].trim().to_string())
    } else {
        None
    }
}
