use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::error::Result;
use crate::message::Message;
use crate::property::Property;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server address
    pub bind_addr: String,
}

/// Server state
#[derive(Debug, Default)]
pub struct ServerState {
    /// Connected clients
    pub clients: HashMap<String, TcpStream>,
    /// Device properties
    pub properties: HashMap<String, HashMap<String, Property>>,
}

impl ServerState {
    /// Create new server state
    pub fn new() -> Self {
        Self::default()
    }
}

/// INDI server
#[derive(Debug)]
pub struct Server {
    /// Server configuration
    config: ServerConfig,
    /// Server state
    state: Arc<Mutex<ServerState>>,
}

impl Server {
    /// Create new server
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(ServerState::new())),
        }
    }

    /// Start server
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.bind_addr).await?;
        info!("Server listening on {}", self.config.bind_addr);

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("New client connection from {}", addr);
                    let state = self.state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(socket, state).await {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Handle client connection
    async fn handle_client(socket: TcpStream, state: Arc<Mutex<ServerState>>) -> Result<()> {
        let (reader, mut writer) = socket.into_split();
        let mut reader = BufReader::new(reader);
        let mut buffer = String::new();

        loop {
            buffer.clear();
            match reader.read_line(&mut buffer).await {
                Ok(0) => {
                    debug!("Client disconnected");
                    break;
                }
                Ok(_) => {
                    debug!("Received message: {}", buffer);
                    match Message::from_str(&buffer) {
                        Ok(message) => {
                            match message {
                                Message::GetProperties { version, device, name } => {
                                    debug!("GetProperties request: version={}, device={:?}, name={:?}", version, device, name);
                                    let state = state.lock().await;
                                    for (dev, props) in &state.properties {
                                        if device.as_ref().map_or(true, |d| d == dev) {
                                            for (prop_name, prop) in props {
                                                if name.as_ref().map_or(true, |n| n == prop_name) {
                                                    let msg = Message::DefProperty(prop.clone());
                                                    let xml = msg.to_xml()?;
                                                    writer.write_all(xml.as_bytes()).await?;
                                                    writer.write_all(b"\n").await?;
                                                }
                                            }
                                        }
                                    }
                                }
                                Message::SetProperty { content } => {
                                    debug!("SetProperty request: {}", content);
                                    // Handle property setting
                                }
                                _ => {
                                    debug!("Ignoring message: {:?}", message);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from client: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}
