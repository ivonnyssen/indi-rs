use clap::Parser;
use colored::*;
use indi_rs::client::{Client, ClientConfig};
use indi_rs::property::{PropertyState, PropertyValue};
use std::error::Error;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn, Level};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// INDI server host
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// INDI server port
    #[arg(short = 'P', long, default_value_t = 7624)]
    port: u16,

    /// Enable debug output
    #[arg(short = 'd', long)]
    debug: bool,

    /// Connect to cameras if found
    #[arg(short = 'c', long)]
    connect_cameras: bool,
}

fn format_property_value(value: &PropertyValue) -> String {
    match value {
        PropertyValue::Text(text) => text.to_string(),
        PropertyValue::Number(num, format) => match format {
            Some(fmt) => format!("{} {}", num, fmt),
            None => num.to_string(),
        },
        PropertyValue::Switch(state) => if *state { "On".green() } else { "Off".red() }.to_string(),
        PropertyValue::Light(state) => match state {
            PropertyState::Idle => "Idle".yellow(),
            PropertyState::Ok => "Ok".green(),
            PropertyState::Busy => "Busy".blue(),
            PropertyState::Alert => "Alert".red(),
        }
        .to_string(),
        PropertyValue::Blob { format, size, .. } => {
            format!("[BLOB format={} size={}]", format, size)
        }
    }
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
            prop.contains("CCD_") || prop.contains("CAMERA_") || device.to_lowercase().contains("ccd")
                || device.to_lowercase().contains("camera")
        });
        debug!("Device {} {} a camera", device, if is_cam { "is" } else { "is not" });
        is_cam
    } else {
        debug!("Could not get properties for device {}", device);
        false
    }
}

/// Connect to a camera device and wait for it to be ready
async fn connect_camera(client: &Client, device: &str) -> Result<bool, Box<dyn Error>> {
    info!("Attempting to connect to camera: {}", device);
    
    // Wait for properties to be defined
    debug!("Waiting for properties to be defined for {}", device);
    let mut retries = 0;
    let properties = loop {
        if let Some(props) = client.get_device_properties(device).await {
            if props.contains_key("CONNECTION") {
                break props;
            }
        }
        if retries >= 10 {
            warn!("Timeout waiting for CONNECTION property from {}", device);
            return Ok(false);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;
    };
    
    debug!("Got properties for {}", device);
    if let Some(connection) = properties.get("CONNECTION") {
        debug!("Found CONNECTION property for {}", device);
        match &connection.value {
            PropertyValue::Switch(connected) => {
                if !connected {
                    // Set the CONNECTION property to ON
                    info!("Connecting to {}", device);
                    // For OneOfMany rule, we need to send an array of switches
                    let switches = vec![
                        ("CONNECT".to_string(), PropertyValue::Switch(true)),
                        ("DISCONNECT".to_string(), PropertyValue::Switch(false)),
                    ];
                    client.set_property_array(device, "CONNECTION", &switches).await?;
                    
                    // Wait for the connection to be established
                    debug!("Waiting for connection to be established");
                    retries = 0;
                    loop {
                        if let Some(props) = client.get_device_properties(device).await {
                            if let Some(conn) = props.get("CONNECTION") {
                                if let PropertyValue::Switch(state) = &conn.value {
                                    if *state {
                                        info!("Successfully connected to {}", device);
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                        if retries >= 20 {
                            warn!("Timeout waiting for {} to connect", device);
                            return Ok(false);
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        retries += 1;
                    }
                } else {
                    info!("Device {} is already connected", device);
                    Ok(true)
                }
            }
            _ => {
                warn!("Unexpected CONNECTION property type for {}", device);
                Ok(false)
            }
        }
    } else {
        warn!("No CONNECTION property found for {}", device);
        Ok(false)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Initialize tracing with debug if requested
    if args.debug {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    }

    info!("Connecting to INDI server at {}:{}", args.host, args.port);
    let config = ClientConfig {
        server_addr: format!("{}:{}", args.host, args.port),
    };
    
    debug!("Creating new client");
    let client = Client::new(config).await?;
    debug!("Connecting client");
    client.connect().await?;

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
        debug!("Processing device: {}", device);
        if let Some(properties) = client.get_device_properties(&device).await {
            debug!(device = %device, property_count = %properties.len(), "Got device properties");
            println!("\n{}", format!("Device: {}", device).bold());
            
            // Check if this is a camera
            if is_camera(&client, &device).await {
                info!("Found camera device: {}", device);
                cameras.push(device.clone());
            }
            
            for (name, prop) in properties {
                debug!(
                    property = %name,
                    value = ?prop.value,
                    state = ?prop.state,
                    perm = %prop.perm,
                    "Processing property"
                );
                println!("  {}", name.bold());
                println!(
                    "    Type: {}",
                    match prop.value {
                        PropertyValue::Text(_) => "Text",
                        PropertyValue::Number(_, _) => "Number",
                        PropertyValue::Switch(_) => "Switch",
                        PropertyValue::Light(_) => "Light",
                        PropertyValue::Blob { .. } => "BLOB",
                    }
                );
                println!("    Value: {}", format_property_value(&prop.value));
                println!("    State: {}", format_property_state(&prop.state));
                println!("    Permissions: {}", prop.perm.to_string().cyan());
                if let Some(label) = prop.label {
                    println!("    Label: {}", label);
                }
                if let Some(group) = prop.group {
                    println!("    Group: {}", group);
                }
            }
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
    }

    Ok(())
}
