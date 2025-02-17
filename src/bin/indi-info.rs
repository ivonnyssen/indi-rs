use clap::Parser;
use tracing::{info, warn};

use indi_rs::client::{Client, ClientConfig};
use indi_rs::error::{Error, Result};
use indi_rs::message::new::{OneNumber, OneSwitch};
use indi_rs::property::{PropertyState, PropertyValue, SwitchState};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// INDI server host
    #[arg(short, long, default_value = "localhost")]
    host: String,

    /// INDI server port
    #[arg(short, long, default_value_t = 7624)]
    port: u16,

    /// Device to connect to
    #[arg(short, long)]
    device: Option<String>,

    /// Exposure time in seconds
    #[arg(short, long)]
    exposure: Option<f64>,
}

/// Connect to a camera device and take an image
async fn process_camera(client: &mut Client, device: &str, exposure: Option<f64>) -> Result<()> {
    // Get device properties
    let props = client
        .get_device_properties(device)
        .await
        .ok_or_else(|| Error::Property("No properties found".to_string()))?;
    if props.is_empty() {
        return Err(Error::Property("No properties found".to_string()));
    }

    // Wait for device properties
    let mut retries = 0;
    while retries < 30 {
        let props = client
            .get_device_properties(device)
            .await
            .ok_or_else(|| Error::Property("No properties found".to_string()))?;
        if props.is_empty() {
            return Err(Error::Property("No properties found".to_string()));
        }
        if !props.is_empty() {
            info!("Found {} properties for device {}", props.len(), device);
            break;
        }
        if retries == 29 {
            warn!("Timeout waiting for device properties");
            return Ok(());
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        retries += 1;
    }

    // Connect to the device
    let mut retries = 0;
    while retries < 30 {
        let numbers = vec![OneNumber {
            name: "CONNECTION".to_string(),
            value: "1".to_string(),
        }];

        client
            .set_number_vector(device, "CONNECTION", numbers)
            .await?;

        let props = client
            .get_device_properties(device)
            .await
            .ok_or_else(|| Error::Property("No properties found".to_string()))?;
        if props.is_empty() {
            return Err(Error::Property("No properties found".to_string()));
        }
        for (name, prop) in props {
            if name == "CONNECTION" && prop.state == PropertyState::Ok {
                info!("Connected to device {}", device);
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        retries += 1;
    }

    // If exposure time is provided, take an image
    if let Some(exposure) = exposure {
        info!("Taking image with {} second exposure", exposure);

        // Enable BLOB mode
        client.enable_blob(device, "Also").await?;

        // Set exposure time
        let numbers = vec![OneNumber {
            name: "CCD_EXPOSURE_VALUE".to_string(),
            value: exposure.to_string(),
        }];

        client
            .set_number_vector(device, "CCD_EXPOSURE", numbers)
            .await?;

        // Wait for exposure to complete
        let mut retries = 0;
        while retries < 300 {
            let props = client
                .get_device_properties(device)
                .await
                .ok_or_else(|| Error::Property("No properties found".to_string()))?;
            if props.is_empty() {
                return Err(Error::Property("No properties found".to_string()));
            }
            for (name, prop) in props {
                if name == "CCD_EXPOSURE" {
                    match prop.state {
                        PropertyState::Idle => {
                            info!("Exposure complete");
                            return Ok(());
                        }
                        PropertyState::Alert => {
                            warn!("Exposure failed");
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            retries += 1;
        }
    }

    Ok(())
}

/// Find all camera devices
async fn find_cameras(client: &mut Client) -> Result<Vec<String>> {
    let mut cameras = Vec::new();

    // Get all devices
    let devices = client.get_devices().await?;

    // Process each device
    for device in devices {
        // Enable BLOB for this device
        client.enable_blob(&device, "Also").await?;

        // Check if this is a camera
        let switches = vec![OneSwitch {
            name: "CONNECT".to_string(),
            value: SwitchState::On,
        }];

        client
            .set_switch_vector(&device, "CONNECTION", switches)
            .await?;

        // Wait for connection
        let mut connected = false;
        let mut sub_retries = 0;
        while sub_retries < 10 {
            let props = client
                .get_device_properties(&device)
                .await
                .ok_or_else(|| Error::Property("No properties found".to_string()))?;
            if props.is_empty() {
                return Err(Error::Property("No properties found".to_string()));
            }
            for (name, prop) in props {
                if name == "CONNECTION" {
                    if let PropertyValue::Switch(state) = prop.value {
                        if state == SwitchState::On {
                            connected = true;
                            break;
                        }
                    }
                }
            }
            if connected {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            sub_retries += 1;
        }

        if connected {
            // Disconnect
            let switches = vec![OneSwitch {
                name: "DISCONNECT".to_string(),
                value: SwitchState::On,
            }];

            client
                .set_switch_vector(&device, "CONNECTION", switches)
                .await?;
            cameras.push(device);
        }
    }

    Ok(cameras)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut client = Client::new(ClientConfig::new(args.host, args.port));
    client.connect().await?;

    if let Some(device) = args.device {
        process_camera(&mut client, &device, args.exposure).await?;
    } else {
        let cameras = find_cameras(&mut client).await?;
        if cameras.is_empty() {
            warn!("No cameras found");
        } else {
            for camera in cameras {
                process_camera(&mut client, &camera, args.exposure).await?;
            }
        }
    }
    Ok(())
}
