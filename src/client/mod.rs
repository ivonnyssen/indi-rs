//! INDI Protocol Client Implementation
//!
//! This module provides the client implementation for the INDI protocol.
//! It handles connecting to INDI servers, sending commands, and receiving responses.

mod config;
mod connection;
mod message;
mod state;

#[cfg(test)]
mod tests;

pub use config::ClientConfig;
pub use state::ClientState;

use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};

use crate::error::Result;
use crate::message::MessageType;

/// INDI client
#[derive(Debug)]
pub struct Client {
    /// Client configuration
    config: ClientConfig,
    /// TCP stream
    stream: Option<TcpStream>,
    /// Client state
    state: Arc<Mutex<ClientState>>,
    /// Message sender for broadcasting messages to all connected receivers
    /// This is used internally by the client and should not be removed even if unused
    #[allow(dead_code)]
    sender: broadcast::Sender<MessageType>,
    /// Message receiver for receiving messages from the broadcast channel
    /// This is used internally by the client and should not be removed even if unused
    #[allow(dead_code)]
    receiver: broadcast::Receiver<MessageType>,
}

impl Client {
    /// Create a new client
    pub fn new(config: ClientConfig) -> Self {
        let (sender, receiver) = broadcast::channel(32);
        Self {
            config,
            state: Arc::new(Mutex::new(ClientState::default())),
            sender,
            receiver,
            stream: None,
        }
    }

    /// Connect to the INDI server
    pub async fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let stream = TcpStream::connect(&addr).await?;
        self.stream = Some(stream);
        Ok(())
    }
}
