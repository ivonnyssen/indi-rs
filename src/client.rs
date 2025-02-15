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
use crate::message::{DefNumber, DefSwitchVector, DefText, Message, OneSwitch};
use crate::property::timestamp;
use crate::property::{
    Property, PropertyPerm, PropertyState, PropertyValue, SwitchRule, SwitchState,
};

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
                debug!(
                    "Got property definition for device '{}', property '{}'",
                    prop.device, prop.name
                );
                let device_props = self.devices.entry(prop.device.clone()).or_default();
                device_props.insert(prop.name.clone(), prop.clone());
            }
            Message::DefSwitchVector(def_switch) => {
                debug!(
                    "Got switch vector for device '{}', property '{}'",
                    def_switch.device, def_switch.name
                );
                debug!("Switches: {:?}", def_switch.switches);
                let device_props = self.devices.entry(def_switch.device.clone()).or_default();

                // Create parent property with switches
                let prop = Property::new(
                    def_switch.device.clone(),
                    def_switch.name.clone(),
                    PropertyValue::SwitchVector(
                        def_switch
                            .switches
                            .iter()
                            .map(|s| {
                                (
                                    s.name.clone(),
                                    if s.value.trim() == "On" {
                                        SwitchState::On
                                    } else {
                                        SwitchState::Off
                                    },
                                )
                            })
                            .collect::<HashMap<_, _>>(),
                    ),
                    def_switch.state,
                    PropertyPerm::from_str(&def_switch.perm).unwrap_or(PropertyPerm::ReadWrite),
                    def_switch.timestamp.clone(),
                );
                device_props.insert(def_switch.name.clone(), prop);
            }
            Message::DefTextVector {
                device,
                name,
                state: prop_state,
                perm,
                texts,
                ..
            } => {
                debug!(
                    "Got text vector for device '{}', property '{}'",
                    device, name
                );
                debug!("Texts: {:?}", texts);
                let device_props = self.devices.entry(device.clone()).or_default();

                // Create parent property with texts
                let prop = Property::new(
                    device.clone(),
                    name.clone(),
                    PropertyValue::TextVector(
                        texts
                            .iter()
                            .map(|t| (t.name.clone(), t.value.clone()))
                            .collect::<HashMap<_, _>>(),
                    ),
                    *prop_state,
                    PropertyPerm::from_str(perm).unwrap_or(PropertyPerm::ReadWrite),
                    timestamp::now(),
                );
                device_props.insert(name.clone(), prop);
            }
            Message::DefNumberVector {
                device,
                name,
                state: prop_state,
                perm,
                numbers,
                ..
            } => {
                debug!(
                    "Got number vector for device '{}', property '{}'",
                    device, name
                );
                debug!("Numbers: {:?}", numbers);
                let device_props = self.devices.entry(device.clone()).or_default();

                // Create parent property with numbers
                let prop = Property::new(
                    device.clone(),
                    name.clone(),
                    PropertyValue::NumberVector(
                        numbers
                            .iter()
                            .map(|n| (n.name.clone(), n.value.parse().unwrap_or(0.0)))
                            .collect::<HashMap<_, _>>(),
                    ),
                    *prop_state,
                    PropertyPerm::from_str(perm).unwrap_or(PropertyPerm::ReadWrite),
                    timestamp::now(),
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
            timestamp::now(),
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
                timestamp::now(),
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
                        timestamp::now(),
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
            timestamp::now(),
        );

        let message = Message::NewProperty(array_prop);
        self.write_message(&message).await?;
        Ok(())
    }

    /// Set a switch vector property for a device
    pub async fn set_switch_vector(
        &self,
        device: &str,
        name: &str,
        switches: HashMap<String, bool>,
    ) -> Result<()> {
        let switches = switches
            .into_iter()
            .map(|(name, state)| {
                let name_clone = name.clone();
                OneSwitch {
                    name,
                    label: name_clone,
                    value: if state {
                        "On".to_string()
                    } else {
                        "Off".to_string()
                    },
                }
            })
            .collect();

        self.update_switch_vector(
            device.to_string(),
            name.to_string(),
            PropertyState::Ok,
            switches,
        )
        .await
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

        #[derive(Debug)]
        struct XmlParseState {
            depth: i32,
            in_tag: bool,
            skip_decl: bool,
            last_char: Option<char>,
            prev_char: Option<char>,
            is_closing_tag: bool,
            is_self_closing: bool,
        }

        impl XmlParseState {
            fn new() -> Self {
                Self {
                    depth: 0,
                    in_tag: false,
                    skip_decl: false,
                    last_char: None,
                    prev_char: None,
                    is_closing_tag: false,
                    is_self_closing: false,
                }
            }

            fn process_char(&mut self, c: char) {
                match c {
                    '<' if !self.in_tag => {
                        self.in_tag = true;
                        self.is_closing_tag = false;
                        self.is_self_closing = false;
                        self.last_char = None;
                    }
                    '/' if self.in_tag => {
                        if self.prev_char == Some('<') {
                            self.is_closing_tag = true;
                            self.depth -= 1;
                        } else if self.prev_char != Some(' ') {
                            // Self-closing tag
                            self.is_self_closing = true;
                        }
                        self.last_char = Some('/');
                    }
                    '?' if self.in_tag && self.prev_char == Some('<') => {
                        self.skip_decl = true;
                        self.last_char = Some('?');
                    }
                    '>' if self.in_tag => {
                        self.in_tag = false;
                        if self.skip_decl {
                            self.skip_decl = false;
                        } else if self.is_self_closing {
                            // Do nothing, depth already adjusted
                        } else if !self.is_closing_tag {
                            self.depth += 1;
                        }
                        self.last_char = None;
                        self.is_closing_tag = false;
                        self.is_self_closing = false;
                    }
                    _ => {
                        self.last_char = None;
                    }
                }
                self.prev_char = Some(c);
            }

            fn is_complete(&self) -> bool {
                self.depth == 0 && !self.in_tag && !self.skip_decl
            }
        }

        let mut state = XmlParseState::new();
        for c in xml_buffer.chars() {
            state.process_char(c);
            if state.is_complete() {
                // Found a complete message
                let message = quick_xml::de::from_str(xml_buffer).ok();
                return (message, true);
            }
        }

        (None, false)
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

    /// Creates a new property with the given device, name, and value
    pub fn new_property(&self, device: String, name: String, value: PropertyValue) -> Result<()> {
        let prop = Property::new(
            device,
            name,
            value,
            PropertyState::Idle,
            PropertyPerm::ReadWrite,
            timestamp::now(),
        );
        self.send_message(Message::NewProperty(prop))
    }

    /// Sets a switch state for a specific device and property
    pub fn set_switch(
        &self,
        device: &str,
        name: &str,
        switch_name: &str,
        state: bool,
    ) -> Result<()> {
        let mut switches = HashMap::new();
        switches.insert(switch_name.to_string(), SwitchState::from(state));

        let prop = Property::new(
            device.to_string(),
            name.to_string(),
            PropertyValue::SwitchVector(switches),
            PropertyState::Ok,
            PropertyPerm::ReadWrite,
            timestamp::now(),
        );

        self.send_message(Message::NewProperty(prop))
    }

    /// Sets a text vector property for a device with the given texts and state
    pub fn set_text_vector(
        &self,
        device: &str,
        name: &str,
        texts: Vec<DefText>,
        prop_state: &PropertyState,
        perm: &str,
    ) -> Result<()> {
        let prop = Property::new(
            device.to_string(),
            name.to_string(),
            PropertyValue::TextVector(
                texts
                    .iter()
                    .map(|t| (t.name.clone(), t.value.clone()))
                    .collect::<HashMap<_, _>>(),
            ),
            *prop_state,
            PropertyPerm::from_str(perm).unwrap_or(PropertyPerm::ReadWrite),
            timestamp::now(),
        );

        self.send_message(Message::NewProperty(prop))
    }

    /// Sets a number vector property for a device with the given numbers and state
    pub fn set_number_vector(
        &self,
        device: &str,
        name: &str,
        numbers: Vec<DefNumber>,
        prop_state: &PropertyState,
        perm: &str,
    ) -> Result<()> {
        let prop = Property::new(
            device.to_string(),
            name.to_string(),
            PropertyValue::NumberVector(
                numbers
                    .iter()
                    .map(|n| (n.name.clone(), n.value.parse().unwrap_or(0.0)))
                    .collect::<HashMap<_, _>>(),
            ),
            *prop_state,
            PropertyPerm::from_str(perm).unwrap_or(PropertyPerm::ReadWrite),
            timestamp::now(),
        );

        self.send_message(Message::NewProperty(prop))
    }

    /// Handles incoming device messages and updates the client state accordingly
    pub async fn handle_device_message(&self, message: Message) -> Result<()> {
        match message {
            Message::DefSwitchVector(def_switch) => {
                let device = def_switch.device.clone();
                let name = def_switch.name.clone();
                let mut state = self.state.lock().await;
                let device_props = state.devices.entry(device.clone()).or_default();
                let switches: HashMap<String, SwitchState> = def_switch
                    .switches
                    .into_iter()
                    .map(|s| (s.name, s.value.parse().unwrap_or(SwitchState::Off)))
                    .collect();

                let prop = Property::new(
                    device,
                    name.clone(),
                    PropertyValue::SwitchVector(switches),
                    def_switch.state,
                    PropertyPerm::from_str(&def_switch.perm).unwrap_or(PropertyPerm::ReadWrite),
                    def_switch.timestamp,
                );
                device_props.insert(name, prop);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Updates a switch vector property with new switch states
    pub async fn update_switch_vector(
        &self,
        device: String,
        name: String,
        prop_state: PropertyState,
        switches: Vec<OneSwitch>,
    ) -> Result<()> {
        let def_switch = DefSwitchVector {
            device,
            name,
            label: String::new(),
            group: String::new(),
            state: prop_state,
            perm: "rw".to_string(),
            rule: SwitchRule::OneOfMany,
            timeout: 0,
            timestamp: timestamp::now(),
            switches,
        };
        self.send_message(Message::DefSwitchVector(def_switch))
    }

    /// Defines a new switch vector property for a device
    pub async fn define_switch_vector(
        &self,
        device: String,
        name: String,
        prop_state: PropertyState,
        perm: String,
        switches: Vec<OneSwitch>,
    ) -> Result<()> {
        let def_switch = DefSwitchVector {
            device,
            name,
            label: String::new(),
            group: String::new(),
            state: prop_state,
            perm,
            rule: SwitchRule::OneOfMany,
            timeout: 0,
            timestamp: timestamp::now(),
            switches,
        };
        self.send_message(Message::DefSwitchVector(def_switch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[test]
    fn test_try_parse_xml_complete_message() {
        let xml = r#"<defSwitchVector device="CCD Simulator" name="CONNECTION" state="Ok" perm="rw" rule="OneOfMany">
<oneSwitch name="CONNECT" label="Connect">On</oneSwitch>
<oneSwitch name="DISCONNECT" label="Disconnect">Off</oneSwitch>
</defSwitchVector>"#;

        let (message, is_complete) = Client::try_parse_xml(xml);
        assert!(is_complete);
        assert!(message.is_some());

        if let Some(Message::DefSwitchVector(def_switch)) = message {
            assert_eq!(def_switch.device, "CCD Simulator");
            assert_eq!(def_switch.name, "CONNECTION");
            assert_eq!(def_switch.switches.len(), 2);
            assert_eq!(def_switch.switches[0].name, "CONNECT");
            assert_eq!(def_switch.switches[0].label, "Connect");
            assert_eq!(def_switch.switches[0].value.trim(), "On");
            assert_eq!(def_switch.switches[1].name, "DISCONNECT");
            assert_eq!(def_switch.switches[1].label, "Disconnect");
            assert_eq!(def_switch.switches[1].value.trim(), "Off");
        } else {
            panic!("Expected DefSwitchVector message");
        }
    }

    #[test]
    fn test_try_parse_xml_incomplete_message() {
        let xml = r#"<defSwitchVector device="CCD Simulator" name="CONNECTION" state="Ok" perm="rw" rule="OneOfMany">
<oneSwitch name="CONNECT" label="Connect">On</oneSwitch>"#;

        let (message, is_complete) = Client::try_parse_xml(xml);
        assert!(!is_complete);
        assert!(message.is_none());
    }

    #[test]
    fn test_try_parse_xml_self_closing_tag() {
        let xml = r#"<getProperties version="1.7" device="CCD Simulator"/>"#;

        let (message, is_complete) = Client::try_parse_xml(xml);
        assert!(is_complete);
        assert!(message.is_some());

        if let Some(Message::GetProperties {
            version, device, ..
        }) = message
        {
            assert_eq!(version, "1.7");
            assert_eq!(device.as_deref(), Some("CCD Simulator"));
        } else {
            panic!("Expected GetProperties message");
        }
    }

    #[test]
    fn test_try_parse_xml_multiple_messages() {
        let xml = r#"<getProperties version="1.7" device="CCD Simulator"/>
<defSwitchVector device="CCD Simulator" name="CONNECTION" state="Ok" perm="rw" rule="OneOfMany">
<oneSwitch name="CONNECT" label="Connect">Off</oneSwitch>
<oneSwitch name="DISCONNECT" label="Disconnect">On</oneSwitch>
</defSwitchVector>"#;

        // First message
        let (first_message, is_complete) = Client::try_parse_xml(xml);
        assert!(is_complete);
        assert!(first_message.is_some());

        if let Some(Message::GetProperties {
            version, device, ..
        }) = first_message
        {
            assert_eq!(version, "1.7");
            assert_eq!(device.as_deref(), Some("CCD Simulator"));
        } else {
            panic!("Expected GetProperties message");
        }

        // Find end of first message and parse second
        let pos = xml.find("<defSwitchVector").unwrap();
        let remaining = &xml[pos..];
        let (second_message, is_complete) = Client::try_parse_xml(remaining);
        assert!(is_complete);
        assert!(second_message.is_some());

        if let Some(Message::DefSwitchVector(def_switch)) = second_message {
            assert_eq!(def_switch.device, "CCD Simulator");
            assert_eq!(def_switch.name, "CONNECTION");
            assert_eq!(def_switch.switches.len(), 2);
            assert_eq!(def_switch.switches[0].name, "CONNECT");
            assert_eq!(def_switch.switches[0].label, "Connect");
            assert_eq!(def_switch.switches[0].value.trim(), "Off");
            assert_eq!(def_switch.switches[1].name, "DISCONNECT");
            assert_eq!(def_switch.switches[1].label, "Disconnect");
            assert_eq!(def_switch.switches[1].value.trim(), "On");
        } else {
            panic!("Expected DefSwitchVector message");
        }
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
        let result = Client::new(ClientConfig {
            server_addr: "127.0.0.1:0".to_string(),
        })
        .await;

        match result {
            Ok(client) => {
                let connect_result = client.connect().await;
                assert!(connect_result.is_err());
            }
            Err(_) => panic!("Failed to create client"),
        }
    }

    #[tokio::test]
    async fn test_parse_xml_multiple_messages() {
        let xml = r#"<getProperties version="1.7" device="CCD Simulator"/>
<defSwitchVector device="CCD Simulator" name="CONNECTION" state="Ok" perm="rw" rule="OneOfMany">
<oneSwitch name="CONNECT" label="Connect">Off</oneSwitch>
<oneSwitch name="DISCONNECT" label="Disconnect">On</oneSwitch>
</defSwitchVector>"#;

        // First message
        let (first_message, is_complete) = Client::try_parse_xml(xml);
        assert!(is_complete);
        assert!(first_message.is_some());

        if let Some(Message::GetProperties {
            version, device, ..
        }) = first_message
        {
            assert_eq!(version, "1.7");
            assert_eq!(device.as_deref(), Some("CCD Simulator"));
        } else {
            panic!("Expected GetProperties message");
        }

        // Find end of first message and parse second
        let pos = xml.find("<defSwitchVector").unwrap();
        let remaining = &xml[pos..];
        let (second_message, is_complete) = Client::try_parse_xml(remaining);
        assert!(is_complete);
        assert!(second_message.is_some());

        if let Some(Message::DefSwitchVector(def_switch)) = second_message {
            assert_eq!(def_switch.device, "CCD Simulator");
            assert_eq!(def_switch.name, "CONNECTION");
            assert_eq!(def_switch.switches.len(), 2);
        } else {
            panic!("Expected DefSwitchVector message");
        }
    }

    #[tokio::test]
    async fn test_set_switch_vector() {
        let (sender, _rx) = broadcast::channel(32);
        let stream = Arc::new(Mutex::new(None));
        let state = Arc::new(Mutex::new(ClientState::new()));
        let client = Client {
            config: ClientConfig {
                server_addr: "127.0.0.1:7624".to_string(),
            },
            stream: stream.clone(),
            state: state.clone(),
            sender,
        };

        let mut switches = HashMap::new();
        switches.insert("CONNECT".to_string(), true);
        switches.insert("DISCONNECT".to_string(), false);

        // Initialize the state with a device and property
        {
            let mut state = state.lock().await;
            let mut device_props = HashMap::new();
            let mut switch_map = HashMap::new();
            switch_map.insert("CONNECT".to_string(), SwitchState::On);
            switch_map.insert("DISCONNECT".to_string(), SwitchState::Off);
            device_props.insert(
                "CONNECTION".to_string(),
                Property {
                    device: "CCD Simulator".to_string(),
                    name: "CONNECTION".to_string(),
                    label: Some("Connection".to_string()),
                    group: Some("Main Control".to_string()),
                    state: PropertyState::Ok,
                    perm: PropertyPerm::ReadWrite,
                    timeout: None,
                    timestamp: timestamp::now(),
                    elements: Some(vec![]),
                    value: PropertyValue::SwitchVector(switch_map),
                },
            );
            state
                .devices
                .insert("CCD Simulator".to_string(), device_props);
        }

        client
            .set_switch_vector("CCD Simulator", "CONNECTION", switches)
            .await
            .unwrap();

        let state = state.lock().await;
        let device_props = state.devices.get("CCD Simulator").unwrap();
        let prop = device_props.get("CONNECTION").unwrap();

        match &prop.value {
            PropertyValue::SwitchVector(switches) => {
                assert_eq!(switches.len(), 2);
                assert_eq!(switches.get("CONNECT"), Some(&SwitchState::On));
                assert_eq!(switches.get("DISCONNECT"), Some(&SwitchState::Off));
            }
            _ => panic!("Wrong property type"),
        }
    }

    #[tokio::test]
    async fn test_handle_device_message() {
        let (sender, _rx) = broadcast::channel(32);
        let stream = Arc::new(Mutex::new(None));
        let state = Arc::new(Mutex::new(ClientState::new()));
        let client = Client {
            config: ClientConfig {
                server_addr: "127.0.0.1:7624".to_string(),
            },
            stream: stream.clone(),
            state: state.clone(),
            sender,
        };

        let def_switch = DefSwitchVector {
            device: "CCD Simulator".to_string(),
            name: "CONNECTION".to_string(),
            label: "Connection".to_string(),
            group: "Main Control".to_string(),
            state: PropertyState::Idle,
            perm: "rw".to_string(),
            rule: SwitchRule::OneOfMany,
            timeout: 60,
            timestamp: "2025-02-14T00:42:55".to_string(),
            switches: vec![
                OneSwitch {
                    name: "CONNECT".to_string(),
                    label: "Connect".to_string(),
                    value: "Off".to_string(),
                },
                OneSwitch {
                    name: "DISCONNECT".to_string(),
                    label: "Disconnect".to_string(),
                    value: "On".to_string(),
                },
            ],
        };

        client
            .handle_device_message(Message::DefSwitchVector(def_switch))
            .await
            .unwrap();

        let state = state.lock().await;
        let device_props = state.devices.get("CCD Simulator").unwrap();
        let prop = device_props.get("CONNECTION").unwrap();

        match &prop.value {
            PropertyValue::SwitchVector(switches) => {
                assert_eq!(switches.len(), 2);
                assert_eq!(switches.get("CONNECT"), Some(&SwitchState::Off));
                assert_eq!(switches.get("DISCONNECT"), Some(&SwitchState::On));
            }
            _ => panic!("Wrong property type"),
        }
    }
}
