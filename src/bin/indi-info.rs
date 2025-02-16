use clap::Parser;
use colored::Colorize;
use indi_rs::{
    client::{Client, ClientConfig},
    error::Result,
    message::DefNumber,
    property::{PropertyState, PropertyValue, SwitchState},
};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Device to connect to
    #[arg(short, long)]
    device: Option<String>,

    /// Exposure time in seconds
    #[arg(short, long)]
    exposure: Option<f64>,

    /// Server address
    #[arg(short, long, default_value = "127.0.0.1:7624")]
    addr: String,
}

/// Connect to a camera device and take an image
async fn connect_camera(client: &mut Client, device: &str) -> Result<bool> {
    info!("Attempting to connect to camera: {}", device);

    // First get device properties
    debug!("Getting properties for {}", device);
    client.get_properties(Some(device), None).await?;

    // Wait for device to be ready
    debug!("Waiting for device to be ready");
    let mut retries = 0;
    while retries < 30 {
        if let Some(props) = client.get_device_properties(device).await {
            debug!("Current properties: {:?}", props.keys().collect::<Vec<_>>());
            if props.contains_key("CONNECTION") {
                debug!("Device is ready");
                break;
            }
        }
        if retries >= 30 {
            warn!("Timeout waiting for device properties");
            return Ok(false);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;

        // Re-request properties every few retries
        if retries % 5 == 0 {
            debug!("Re-requesting properties for {}", device);
            client.get_properties(Some(device), None).await?;
        }
    }

    // Enable BLOB mode before connecting
    debug!("Enabling BLOB mode for {}", device);
    client.enable_blob(device, "Also").await?;

    // Connect to the device
    debug!("Connecting to device");
    let mut switches = std::collections::HashMap::new();
    switches.insert("CONNECT".to_string(), SwitchState::On);
    switches.insert("DISCONNECT".to_string(), SwitchState::Off);
    client
        .set_switch_vector(device, "CONNECTION", switches)
        .await?;

    // Wait for connection to complete
    debug!("Waiting for connection to complete");
    retries = 0;
    while retries < 30 {
        if let Some(props) = client.get_device_properties(device).await {
            if let Some(connection) = props.get("CONNECTION") {
                debug!("Connection state: {:?}", connection);
                match &connection.value {
                    PropertyValue::Switch(state) => {
                        if (*state).into() {
                            info!("Successfully connected to {}", device);
                            return Ok(true);
                        }
                    }
                    _ => warn!("Unexpected property type for CONNECTION"),
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;

        // Re-request properties every few retries
        if retries % 5 == 0 {
            debug!("Re-requesting properties for {}", device);
            client.get_properties(Some(device), None).await?;
        }
    }

    warn!("Failed to connect to device");
    Ok(false)
}

/// Take an image with the camera
async fn take_image(client: &mut Client, device: &str, exposure: f64) -> Result<bool> {
    info!("Taking image with {} second exposure", exposure);

    // Set exposure time
    debug!("Setting exposure time to {} seconds", exposure);
    let numbers = vec![DefNumber {
        name: "CCD_EXPOSURE_VALUE".to_string(),
        value: exposure.to_string(),
        format: "%.6f".to_string(),
        min: "0.0".to_string(),
        max: "3600.0".to_string(),
        step: "0.000001".to_string(),
        label: "Duration (s)".to_string(),
    }];
    client.set_number_vector(
        device,
        "CCD_EXPOSURE",
        numbers,
        &PropertyState::Busy,
        "Taking exposure",
    )?;

    // Wait for exposure to complete and image to be ready
    debug!("Waiting for exposure to complete");
    let mut retries = 0;
    while retries < (exposure * 2.0 + 30.0) as i32 {
        if let Some(props) = client.get_device_properties(device).await {
            if let Some(exposure_prop) = props.get("CCD_EXPOSURE") {
                match &exposure_prop.value {
                    PropertyValue::Number(value, _) => {
                        if *value <= 0.0 {
                            debug!("Exposure complete");
                            break;
                        } else {
                            debug!("Exposure remaining: {:.1} seconds", value);
                        }
                    }
                    _ => warn!("Unexpected property type for CCD_EXPOSURE"),
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;
    }

    // Wait for BLOB data
    debug!("Waiting for image data");
    retries = 0;
    while retries < 30 {
        if let Some(props) = client.get_device_properties(device).await {
            if let Some(blob_prop) = props.get("CCD1") {
                match &blob_prop.value {
                    PropertyValue::Blob { data, .. } => {
                        if !data.is_empty() {
                            info!("Image data received ({} bytes)", data.len());
                            // TODO: Save image data to file
                            return Ok(true);
                        }
                    }
                    _ => warn!("Unexpected property type for CCD1"),
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;
    }

    warn!("Failed to receive image data");
    Ok(false)
}

/// Initialize the INDI client with logging configuration
async fn initialize_client(addr: &str) -> Result<Client> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .with_level(true)
        .with_thread_names(true)
        .init();

    info!("Initializing INDI client");
    let config = ClientConfig {
        server_addr: addr.to_string(),
    };
    let mut client = Client::new(config).await?;
    client.connect().await?;
    client.get_properties(None, None).await?;

    // Wait for devices to be discovered
    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(client)
}

/// Find all available cameras from the device list
async fn find_cameras(client: &mut Client) -> Result<Vec<String>> {
    let mut cameras = Vec::new();
    debug!("Searching for cameras...");
    
    // Wait for devices to be discovered
    let mut retries = 0;
    while retries < 10 {
        let devices = client.get_devices().await?;
        debug!("Found devices: {:?}", devices);
        
        if !devices.is_empty() {
            for device in devices {
                // Check if device name suggests it's a camera
                if device.to_uppercase().contains("CCD") || device.to_uppercase().contains("CAMERA") {
                    // Try to connect to the device
                    debug!("Found potential camera device: {}", device);
                    debug!("Attempting to connect to device: {}", device);
                    
                    // Enable BLOB mode before connecting
                    debug!("Enabling BLOB mode for {}", device);
                    client.enable_blob(&device, "Also").await?;
                    
                    // Connect to the device
                    let mut switches = std::collections::HashMap::new();
                    switches.insert("CONNECT".to_string(), SwitchState::On);
                    switches.insert("DISCONNECT".to_string(), SwitchState::Off);
                    client.set_switch_vector(&device, "CONNECTION", switches).await?;
                    
                    // Wait for connection to complete
                    debug!("Waiting for connection to complete...");
                    let mut connection_retries = 0;
                    let mut connected = false;
                    while connection_retries < 10 {
                        if let Some(props) = client.get_device_properties(&device).await {
                            if let Some(connection) = props.get("CONNECTION") {
                                debug!("Connection property value: {:?}", connection.value);
                                match &connection.value {
                                    PropertyValue::SwitchVector(switches) => {
                                        if let Some(state) = switches.get("CONNECT") {
                                            if state == &SwitchState::On {
                                                debug!("Successfully connected to {}", device);
                                                connected = true;
                                                break;
                                            }
                                        }
                                    }
                                    PropertyValue::Switch(state) => {
                                        if state == &SwitchState::On {
                                            debug!("Successfully connected to {}", device);
                                            connected = true;
                                            break;
                                        }
                                    }
                                    _ => {
                                        warn!("Unexpected property type for CONNECTION: {:?}", connection.value);
                                    }
                                }
                            }
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        connection_retries += 1;
                        debug!("Waiting for connection... ({}/10)", connection_retries);
                    }
                    
                    if !connected {
                        debug!("Failed to connect to device {}", device);
                        continue;
                    }
                    
                    // Request all properties for this device
                    client.get_properties(Some(&device), None).await?;
                    
                    // Wait longer for connection and properties
                    debug!("Waiting for device properties...");
                    let mut is_camera = false;
                    for i in 0..10 {
                        debug!("Checking properties attempt {}/10", i + 1);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        
                        // Request properties again
                        client.get_properties(Some(&device), None).await?;
                        
                        if let Some(props) = client.get_device_properties(&device).await {
                            debug!(
                                "Properties for {}: {:?}",
                                device,
                                props.keys().collect::<Vec<_>>()
                            );
                            // Check for any camera-related properties
                            if props.contains_key("CCD_EXPOSURE") || 
                               props.contains_key("CCD_FRAME") ||
                               props.contains_key("CCD_INFO") ||
                               props.contains_key("CCD_TEMPERATURE") {
                                info!("Found camera: {}", device);
                                is_camera = true;
                                break;
                            }
                        }
                    }
                    
                    // Handle camera status
                    if is_camera {
                        cameras.push(device);
                    } else {
                        debug!("Device {} is not a camera, disconnecting", device);
                        let mut switches = std::collections::HashMap::new();
                        switches.insert("CONNECT".to_string(), SwitchState::Off);
                        switches.insert("DISCONNECT".to_string(), SwitchState::On);
                        client.set_switch_vector(&device, "CONNECTION", switches).await?;
                    }
                }
            }
            if !cameras.is_empty() {
                break;
            }
        }
        
        debug!("No cameras found yet, retrying... ({}/10)", retries + 1);
        tokio::time::sleep(Duration::from_secs(1)).await;
        retries += 1;
    }
    
    Ok(cameras)
}

/// Process a single camera - connect and optionally take an image
async fn process_camera(client: &mut Client, camera: &str, exposure: Option<f64>) -> Result<()> {
    println!("Camera: {}", camera);
    let success = connect_camera(client, camera).await?;
    println!(
        "  Connection - {}",
        if success {
            "Success".green()
        } else {
            "Failed".red()
        }
    );

    // Take an image if exposure time is provided
    if success && exposure.is_some() {
        let success = take_image(client, camera, exposure.unwrap()).await?;
        println!(
            "  Image capture - {}",
            if success {
                "Success".green()
            } else {
                "Failed".red()
            }
        );
    }
    Ok(())
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut client = initialize_client(&args.addr).await?;

    // Handle single device mode vs discover mode
    if let Some(device) = args.device.as_ref() {
        process_camera(&mut client, device, args.exposure).await?;
    } else {
        let cameras = find_cameras(&mut client).await?;
        if cameras.is_empty() {
            println!("{}", "No cameras found".red());
            return Ok(());
        }

        // Process all discovered cameras
        for camera in cameras {
            process_camera(&mut client, &camera, args.exposure).await?;
        }
    }

    Ok(())
}
