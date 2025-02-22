use crate::error::Result;
use crate::message::Message;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};
use tokio::sync::Mutex;
use tracing::{debug, error};

/// Configuration module for INDI client
mod config;
/// Connection handling for INDI protocol
pub mod connection;
/// Message handling module for INDI client
pub mod message;
/// State management module for INDI client
mod state;

use self::connection::Connection;
pub use self::message::MessageHandler;
pub use config::ClientConfig;
pub use state::ClientState;

/// INDI client implementation
///
/// The Client struct provides functionality for:
/// - Connecting to an INDI server
/// - Sending and receiving messages
/// - Managing device properties
/// - Handling connection state
#[derive(Debug, Clone)]
pub struct Client {
    config: ClientConfig,
    state: Arc<Mutex<ClientState>>,
    reader: Arc<Mutex<BufReader<OwnedReadHalf>>>,
    writer: Arc<Mutex<BufWriter<OwnedWriteHalf>>>,
}

impl Client {
    /// Create a new client
    pub async fn new(config: ClientConfig) -> Result<Self> {
        debug!("Connecting to {}:{}", config.host, config.port);
        let stream = TcpStream::connect((config.host.as_str(), config.port)).await?;
        let (read_half, write_half) = stream.into_split();

        Ok(Self {
            config,
            state: Arc::new(Mutex::new(ClientState::default())),
            reader: Arc::new(Mutex::new(BufReader::new(read_half))),
            writer: Arc::new(Mutex::new(BufWriter::new(write_half))),
        })
    }

    /// Get reader
    pub fn reader(&self) -> Arc<Mutex<BufReader<OwnedReadHalf>>> {
        self.reader.clone()
    }

    /// Get writer
    pub fn writer(&self) -> Arc<Mutex<BufWriter<OwnedWriteHalf>>> {
        self.writer.clone()
    }

    /// Get state
    pub fn state(&self) -> Arc<Mutex<ClientState>> {
        self.state.clone()
    }

    /// Read messages from the server
    pub async fn read_messages(&self) -> Result<()> {
        debug!(
            "Starting message reader for {}:{}",
            self.config.host, self.config.port
        );
        let mut buf = Vec::new();
        loop {
            let mut reader = self.reader.lock().await;
            match reader.read_until(b'>', &mut buf).await {
                Ok(0) => {
                    debug!("Server closed connection");
                    break;
                }
                Ok(_) => {
                    let message = Message::new(String::from_utf8_lossy(&buf).to_string());
                    debug!("Received message: {}", message.content);
                    buf.clear();
                }
                Err(e) => {
                    error!(
                        "Error reading from server {}:{}: {}",
                        self.config.host, self.config.port, e
                    );
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }
}

impl Connection for Client {
    async fn disconnect(&mut self) -> Result<()> {
        debug!(
            "Disconnecting from server {}:{}",
            self.config.host, self.config.port
        );
        Ok(())
    }
}

impl MessageHandler for Client {
    async fn send_message(&mut self, message: &str) -> Result<()> {
        debug!(
            "Sending message to {}:{}: {}",
            self.config.host,
            self.config.port,
            message.trim()
        );
        let writer = self.writer();
        let mut writer = writer.lock().await;
        writer.write_all(message.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }
}
