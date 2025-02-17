use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::error::Result;
use crate::message::MessageType;
use tracing::debug;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server address
    pub bind_addr: String,
}

/// Server state
#[derive(Debug, Default)]
pub struct ServerState {
    /// Devices and their properties
    pub devices: HashMap<String, HashMap<String, MessageType>>,
    /// Last message received
    pub last_message: Option<MessageType>,
}

impl ServerState {
    /// Create a new server state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update state with a message
    pub fn update(&mut self, message: &MessageType) {
        match message {
            MessageType::GetProperties(get_props) => {
                debug!("Got get properties for device '{:?}'", get_props.device);
                // Handle get properties request
            }
            _ => {
                debug!("Got message: {:?}", message);
            }
        }
        self.last_message = Some(message.clone());
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
        debug!("Server listening on {}", self.config.bind_addr);

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    debug!("New client connection from {}", addr);
                    let state = self.state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(socket, state).await {
                            debug!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    debug!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Handle client connection
    async fn handle_client(socket: TcpStream, state: Arc<Mutex<ServerState>>) -> Result<()> {
        let (reader, _writer) = socket.into_split();
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
                    match MessageType::from_str(&buffer) {
                        Ok(message) => {
                            let mut state = state.lock().await;
                            state.update(&message);
                        }
                        Err(e) => {
                            debug!("Failed to parse message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    debug!("Error reading from client: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}
