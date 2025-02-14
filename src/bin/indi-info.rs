use clap::Parser;
use colored::{ColoredString, Colorize};
use indi_rs::client::{Client, ClientConfig};
use indi_rs::error::Result;
use indi_rs::property::{PropertyState, PropertyValue};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn, Level};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug output
    #[arg(short, long)]
    debug: bool,

    /// Connect to all found cameras
    #[arg(short, long)]
    connect_cameras: bool,
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

    if let Some(properties) = client.get_device_properties(device).await {
        // Look for common camera properties
        let is_cam = properties.keys().any(|prop| {
            prop.contains("CCD_")
                || prop.contains("CAMERA_")
                || prop.contains("GUIDER_")
                || prop.contains("FOCUS_")
        });

        debug!(
            "Device {} {} a camera",
            device,
            if is_cam { "is" } else { "is not" }
        );
        is_cam
    } else {
        debug!("Could not get properties for device {}", device);
        false
    }
}

/// Connect to a camera device
async fn connect_camera(client: &Client, device: &str) -> Result<bool> {
    info!("Attempting to connect to camera: {}", device);

    // Wait for properties to be defined
    debug!("Waiting for properties to be defined");
    let mut retries = 0;
    let properties = loop {
        if let Some(props) = client.get_device_properties(device).await {
            if props.contains_key("CONNECTION") {
                break props;
            }
        }
        if retries > 10 {
            warn!("Timeout waiting for CONNECTION property from {}", device);
            return Ok(false);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;
    };

    // Check if already connected
    if let Some(conn) = properties.get("CONNECTION") {
        if let PropertyValue::Switch(state) = conn.value {
            if state {
                info!("Device {} is already connected", device);
                Ok(true)
            } else {
                // Try to connect
                debug!("Attempting to connect to {}", device);
                let switches = vec![
                    ("CONNECT".to_string(), PropertyValue::Switch(true)),
                    ("DISCONNECT".to_string(), PropertyValue::Switch(false)),
                ];
                client
                    .set_property_array(device, "CONNECTION", &switches)
                    .await?;

                // Wait for the connection to be established
                debug!("Waiting for connection to be established");
                let mut retries = 0;
                loop {
                    if let Some(props) = client.get_device_properties(device).await {
                        if let Some(conn) = props.get("CONNECTION") {
                            if let PropertyValue::Switch(state) = conn.value {
                                if state {
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
            }
        } else {
            warn!(
                "CONNECTION property for {} is not a switch property",
                device
            );
            Ok(false)
        }
    } else {
        warn!("Device {} does not have a CONNECTION property", device);
        Ok(false)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing with debug if requested
    let level = if args.debug { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    debug!("Creating new client");
    let config = ClientConfig {
        server_addr: "localhost:7624".to_string(),
    };

    let client = Client::new(config).await?;
    debug!("Connecting client");
    client.connect().await?;

    // Wait for initial properties
    debug!("Waiting for initial properties");
    tokio::time::sleep(Duration::from_secs(1)).await;

    debug!("Getting devices");
    let devices = match timeout(Duration::from_secs(5), client.get_devices()).await {
        Ok(result) => result?,
        Err(_) => {
            warn!("Timeout while getting devices");
            return Ok(());
        }
    };
    debug!("Found {} devices", devices.len());

    // Track cameras for potential connection
    let mut cameras = Vec::new();

    for device in devices {
        debug!("Processing device: {}", &device);
        if let Some(properties) = client.get_device_properties(&device).await {
            debug!(device = %device, property_count = %properties.len(), "Got device properties");
            println!("\n{}", format!("Device: {}", device).bold());

            for (name, prop) in properties {
                println!(
                    "  {} [{}]",
                    name,
                    format_property_state(&prop.state)
                );
            }

            if is_camera(&client, &device).await {
                info!("Found camera device: {}", device);
                cameras.push(device.clone());
            }
        } else {
            warn!("Could not get properties for device {}", device);
        }
    }

    // Connect to cameras if requested
    if args.connect_cameras && !cameras.is_empty() {
        println!("\n{}", "Connecting to cameras:".bold());
        for camera in cameras {
            match connect_camera(&client, &camera).await {
                Ok(true) => println!("  {} {}", "✓".green(), camera),
                Ok(false) => println!("  {} {}", "✗".red(), camera),
                Err(e) => println!("  {} {} ({})", "✗".red(), camera, e),
            }
        }
    } else if !cameras.is_empty() {
        println!("\n{}", format!("Found {} cameras:", cameras.len()).bold());
        for camera in cameras {
            println!("  {}", camera);
        }
    } else {
        println!("\nNo cameras found");
    }

    Ok(())
}
