use clap::Parser;
use colored::{ColoredString, Colorize};
use indi_rs::client::{Client, ClientConfig};
use indi_rs::error::Result;
use indi_rs::property::{PropertyState, PropertyValue};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Server address
    #[arg(short, long, default_value = "localhost:7624")]
    addr: String,

    /// Enable verbose (debug) logging
    #[arg(short, long)]
    verbose: bool,
}

fn format_property_state(state: &PropertyState) -> ColoredString {
    match state {
        PropertyState::Idle => "Idle".yellow(),
        PropertyState::Ok => "Ok".green(),
        PropertyState::Busy => "Busy".blue(),
        PropertyState::Alert => "Alert".red(),
    }
}

/// Check if a device is a camera by looking for typical camera properties
async fn is_camera(client: &Client, device: &str) -> bool {
    debug!("Checking if {} is a camera device", device);

    // Wait for all properties to be exposed
    for i in 0..10 {
        debug!("Attempt {} to get properties for {}", i + 1, device);
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        if let Some(properties) = client.get_device_properties(device).await {
            debug!("Found {} properties for {}", properties.len(), device);

            // Print all properties for debugging
            for (key, prop) in properties.iter() {
                debug!("Property: {} = {:?}", key, prop);
            }

            // Look for common camera properties or check device name
            if device.to_uppercase().contains("CCD") || device.to_uppercase().contains("CAMERA") {
                debug!("Device name indicates it's a camera");
                return true;
            }

            // Check for any property that might indicate it's a camera
            for prop_name in properties.keys() {
                let prop_upper = prop_name.to_uppercase();
                if prop_upper.contains("CCD")
                    || prop_upper.contains("CAMERA")
                    || prop_upper.contains("GUIDER")
                    || prop_upper.contains("FOCUS")
                    || prop_upper.contains("EXPOSURE")
                    || prop_upper.contains("FRAME")
                    || prop_upper.contains("CAPTURE")
                {
                    debug!("Found camera property: {}", prop_name);
                    return true;
                }
            }
        } else {
            debug!("No properties found for {} on attempt {}", device, i + 1);
        }
    }

    debug!("No camera properties found for {}", device);
    false
}

/// Disable simulation mode for a device
async fn disable_simulation(client: &mut Client, device: &str) -> Result<bool> {
    info!("Disabling simulation mode for: {}", device);

    // Set SIMULATION to Off
    let mut switches = HashMap::new();
    switches.insert("ENABLE".to_string(), false);
    switches.insert("DISABLE".to_string(), true);

    client
        .set_switch_vector(device, "SIMULATION", switches)
        .await?;

    Ok(true)
}

/// Enable simulation mode for a device
#[allow(dead_code)]
async fn enable_simulation(client: &mut Client, device: &str) -> Result<()> {
    info!("Enabling simulation mode for: {}", device);

    // Set SIMULATION to On
    let mut switches = HashMap::new();
    switches.insert("ENABLE".to_string(), true);
    switches.insert("DISABLE".to_string(), false);

    client
        .set_switch_vector(device, "SIMULATION", switches)
        .await?;

    Ok(())
}

/// Connect to a camera device
async fn connect_camera(client: &mut Client, device: &str) -> Result<bool> {
    info!("Attempting to connect to camera: {}", device);

    // Wait for properties to be defined
    debug!("Waiting for properties to be defined");
    let mut retries = 0;
    let properties = loop {
        // Request CONNECTION property explicitly
        if retries % 5 == 0 {
            debug!("Requesting CONNECTION property for device {}", device);
            client
                .get_properties(Some(device), Some("CONNECTION"))
                .await?;
        }

        if let Some(props) = client.get_device_properties(device).await {
            debug!("Found properties for {}: {:?}", device, props.keys());
            if props.contains_key("CONNECTION") {
                break props;
            }
        }
        if retries > 30 {
            warn!("Timeout waiting for CONNECTION property from {}", device);
            return Ok(false);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;
    };

    // Check if already connected
    if let Some(conn) = properties.get("CONNECTION") {
        debug!("Found CONNECTION property: {:?}", conn);

        // Try to connect
        debug!("Attempting to connect to {}", device);
        let mut switches = HashMap::new();
        switches.insert("CONNECT".to_string(), true);
        switches.insert("DISCONNECT".to_string(), false);

        client
            .set_switch_vector(device, "CONNECTION", switches)
            .await?;

        // Wait for the connection to be established
        debug!("Waiting for connection to be established");
        let mut retries = 0;
        loop {
            if let Some(props) = client.get_device_properties(device).await {
                if let Some(conn) = props.get("CONNECTION") {
                    debug!("Connection state: {:?}", conn);
                    if let PropertyValue::Switch(state) = &conn.value {
                        if bool::from(*state) {
                            info!("Successfully connected to {}", device);
                            return Ok(true);
                        }
                    }
                }
            }
            if retries > 10 {
                warn!("Timeout waiting for {} to connect", device);
                return Ok(false);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
            retries += 1;
        }
    } else {
        warn!("Device {} does not have a CONNECTION property", device);
        Ok(false)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    // Create client
    let config = ClientConfig {
        server_addr: args.addr,
    };

    let mut client = Client::new(config).await?;
    client.connect().await?;

    // Get devices
    info!("Sending initial GetProperties message");
    client.get_properties(None, None).await?;

    // Wait longer for devices to be discovered
    info!("Waiting for devices to be discovered...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Get list of devices
    let devices = client.get_devices().await?;
    debug!("Found {} devices", devices.len());
    for device in &devices {
        debug!("Device: {}", device);
    }

    let mut cameras = Vec::new();

    // Print device info and check for cameras
    for device in devices {
        if let Some(properties) = client.get_device_properties(&device).await {
            println!("\nDevice: {}", device);
            for (name, prop) in properties.iter() {
                println!("  {} [{}]", name, format_property_state(&prop.state));
            }
        }

        // Check if it's a camera
        if is_camera(&client, &device).await {
            info!("Found camera device: {}", device);
            cameras.push(device);
        }
    }

    if cameras.is_empty() {
        println!("\nNo cameras found");
    } else {
        println!("\nFound {} cameras:", cameras.len());
        for camera in &cameras {
            println!("  {}", camera);
        }

        // Try to connect to each camera
        println!("\nAttempting to connect to cameras:");
        for camera in &cameras {
            // First disable simulation mode
            match disable_simulation(&mut client, camera).await {
                Ok(_) => debug!("Disabled simulation mode for {}", camera),
                Err(e) => warn!("Failed to disable simulation mode for {}: {}", camera, e),
            }

            // Wait a bit for the simulation mode to take effect
            tokio::time::sleep(Duration::from_millis(1000)).await;

            // Now try to connect
            match connect_camera(&mut client, camera).await {
                Ok(true) => println!("  {} - Connected", camera),
                Ok(false) => println!("  {} - Failed to connect", camera),
                Err(e) => println!("  {} - Error: {}", camera, e),
            }
        }
    }

    Ok(())
}
