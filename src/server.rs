//! INDI Protocol Server Implementation
//!
//! This module provides the server implementation for the INDI protocol.
//! It handles client connections, manages devices and their properties,
//! and routes messages between clients and devices.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

use crate::error::Error;
use crate::message::Message;
use crate::property::{Property, PropertyState, PropertyValue};
use crate::Result;

/// Default INDI server port
pub const DEFAULT_PORT: u16 = 7624;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind to
    pub bind_address: SocketAddr,
    /// Maximum number of clients
    pub max_clients: usize,
    /// Maximum message size in bytes
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

/// Device driver trait
#[async_trait::async_trait]
pub trait DeviceDriver: Send + Sync {
    /// Returns the device name
    fn name(&self) -> &str;

    /// Returns all properties for this device
    fn properties(&self) -> Vec<Property>;

    /// Handles a property update
    async fn handle_property(&mut self, property: Property) -> Result<()>;
}

/// Client connection state
struct ClientConnection {
    /// Message sender for this client
    sender: mpsc::Sender<Message>,
}

/// Server state
struct ServerState {
    /// Connected clients
    clients: HashMap<SocketAddr, ClientConnection>,
    /// Registered devices
    devices: HashMap<String, Box<dyn DeviceDriver>>,
    /// Device properties
    properties: HashMap<String, HashMap<String, Property>>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
            devices: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    /// Gets a mutable reference to a device driver
    async fn get_device_mut(&mut self, device: &str) -> Option<&mut Box<dyn DeviceDriver>> {
        self.devices.get_mut(device)
    }
}

/// INDI server
pub struct Server {
    config: ServerConfig,
    state: Arc<RwLock<ServerState>>,
    shutdown: broadcast::Sender<()>,
}

impl Server {
    /// Creates a new INDI server with the given configuration
    pub fn new(config: ServerConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            config,
            state: Arc::new(RwLock::new(ServerState::new())),
            shutdown: shutdown_tx,
        }
    }

    /// Creates a new INDI server with default configuration
    pub fn new_default() -> Self {
        Self::new(ServerConfig::default())
    }

    /// Registers a device driver with the server
    pub async fn register_device<D: DeviceDriver + 'static>(&self, driver: D) -> Result<()> {
        let mut state = self.state.write().await;
        let name = driver.name().to_string();

        // Store device properties
        let properties = driver.properties();
        let mut device_props = HashMap::new();
        for prop in properties {
            device_props.insert(prop.name.clone(), prop);
        }

        state.properties.insert(name.clone(), device_props);
        state.devices.insert(name, Box::new(driver));

        Ok(())
    }

    /// Starts the server
    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.config.bind_address)
            .await
            .map_err(Error::Io)?;

        info!("INDI server listening on {}", self.config.bind_address);

        let mut shutdown_rx = self.shutdown.subscribe();

        loop {
            tokio::select! {
                Ok((socket, addr)) = listener.accept() => {
                    if let Err(e) = self.handle_client(socket, addr).await {
                        error!("Failed to handle client {}: {}", addr, e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Server shutdown requested");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handles a client connection
    async fn handle_client(&self, socket: TcpStream, addr: SocketAddr) -> Result<()> {
        let state = self.state.read().await;
        if state.clients.len() >= self.config.max_clients {
            return Err(Error::Connection(
                "Maximum number of clients reached".to_string(),
            ));
        }
        drop(state);

        info!("New client connection from {}", addr);

        let (reader, writer) = socket.into_split();
        let reader = BufReader::new(reader);
        let writer = Arc::new(Mutex::new(writer));

        let (tx, mut rx) = mpsc::channel(32);

        // Store client connection
        let connection = ClientConnection { sender: tx };

        {
            let mut state = self.state.write().await;
            state.clients.insert(addr, connection);
        }

        // Spawn writer task
        let writer_clone = Arc::clone(&writer);
        let mut shutdown_rx = self.shutdown.subscribe();

        let writer_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(message) = rx.recv() => {
                        let mut writer = writer_clone.lock().await;
                        if let Ok(xml) = message.to_xml() {
                            if let Err(e) = writer.write_all(xml.as_bytes()).await {
                                error!("Failed to write to client {}: {}", addr, e);
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        // Handle incoming messages
        let mut lines = reader.lines();
        let state_clone = Arc::clone(&self.state);

        while let Ok(Some(line)) = lines.next_line().await {
            if line.len() > self.config.max_message_size {
                warn!("Message from {} exceeds maximum size", addr);
                continue;
            }

            match Message::from_xml(&line) {
                Ok(message) => {
                    debug!("Received message from {}: {:?}", addr, message);
                    if let Err(e) = self.handle_message(message, addr).await {
                        error!("Failed to handle message from {}: {}", addr, e);
                    }
                }
                Err(e) => {
                    error!("Failed to parse message from {}: {}", addr, e);
                }
            }
        }

        // Client disconnected
        {
            let mut state = state_clone.write().await;
            state.clients.remove(&addr);
        }

        writer_task.abort();
        info!("Client {} disconnected", addr);

        Ok(())
    }

    /// Handles an INDI message
    async fn handle_message(&self, message: Message, sender: SocketAddr) -> Result<()> {
        let mut state = self.state.write().await;

        match message {
            Message::GetProperty(_xml) => {
                // Parse device and property name from XML
                // TODO: Implement proper XML parsing
                let device = "device"; // Extract from XML
                let name = "property"; // Extract from XML

                if let Some(device_props) = state.properties.get(device) {
                    if let Some(prop) = device_props.get(name) {
                        // Send property value back to client
                        if let Some(client) = state.clients.get(&sender) {
                            let response = Message::DefProperty(format!(
                                "<defProperty device='{}' name='{}'>{}</defProperty>",
                                device, name, prop.value
                            ));
                            client.sender.send(response).await.map_err(|e| {
                                Error::Io(std::io::Error::new(
                                    std::io::ErrorKind::BrokenPipe,
                                    format!("Failed to send response: {}", e),
                                ))
                            })?;
                        }
                    }
                }
            }
            Message::SetProperty(_xml) => {
                // Parse device, property name, and value from XML
                // TODO: Implement proper XML parsing
                let device = "device"; // Extract from XML
                let name = "property"; // Extract from XML
                let value = PropertyValue::Text("value".to_string()); // Extract from XML

                if let Some(device_driver) = state.get_device_mut(device).await {
                    let property = Property::new(
                        device.to_string(),
                        name.to_string(),
                        value.clone(),
                        PropertyState::Ok,
                        crate::property::PropertyPerm::RW,
                    );

                    // Update property
                    device_driver.handle_property(property).await?;

                    // Notify all clients about the property update
                    let update = Message::NewProperty(format!(
                        "<newProperty device='{}' name='{}'>{}</newProperty>",
                        device, name, value
                    ));

                    for client in state.clients.values() {
                        client.sender.send(update.clone()).await.map_err(|e| {
                            Error::Io(std::io::Error::new(
                                std::io::ErrorKind::BrokenPipe,
                                format!("Failed to send update: {}", e),
                            ))
                        })?;
                    }
                }
            }
            _ => {
                // Handle other message types
            }
        }

        Ok(())
    }

    /// Shuts down the server
    pub fn shutdown(&self) {
        let _ = self.shutdown.send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct TestDevice {
        name: String,
        properties: Vec<Property>,
    }

    impl TestDevice {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                properties: vec![Property::new(
                    name.to_string(),
                    "TEST_PROP".to_string(),
                    PropertyValue::Text("test".to_string()),
                    PropertyState::Ok,
                    crate::property::PropertyPerm::RW,
                )],
            }
        }
    }

    #[async_trait::async_trait]
    impl DeviceDriver for TestDevice {
        fn name(&self) -> &str {
            &self.name
        }

        fn properties(&self) -> Vec<Property> {
            self.properties.clone()
        }

        async fn handle_property(&mut self, property: Property) -> Result<()> {
            self.properties = vec![property];
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_server_startup() {
        let config = ServerConfig {
            bind_address: "127.0.0.1:0".parse().unwrap(),
            ..Default::default()
        };

        let server = Server::new(config);
        let device = TestDevice::new("TestDevice");

        server.register_device(device).await.unwrap();

        // Run server in background
        tokio::spawn(async move {
            server.run().await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
