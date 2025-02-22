use clap::Parser;
use indi_rs::{
    client::{Client, ClientConfig},
    error::Result,
};
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// INDI server host
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// INDI server port
    #[arg(short = 'P', long, default_value_t = 7624)]
    port: u16,

    /// Device to connect to
    #[arg(short, long)]
    device: Option<String>,

    /// Exposure time in seconds
    #[arg(short, long)]
    exposure: Option<f64>,
}

async fn process_camera(_client: &mut Client, device: &str, _exposure: Option<f64>) -> Result<()> {
    info!("Processing camera {}", device);
    Ok(())
}

async fn find_cameras(_client: &mut Client) -> Result<Vec<String>> {
    Ok(vec![])
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Create client config
    let config = ClientConfig {
        host: args.host,
        port: args.port,
    };

    // Connect to INDI server
    let mut client = Client::new(config).await?;

    // Process specific device or find all cameras
    if let Some(device) = args.device {
        process_camera(&mut client, &device, args.exposure).await?;
    } else {
        let cameras = find_cameras(&mut client).await?;
        if cameras.is_empty() {
            info!("No cameras found");
        } else {
            for camera in cameras {
                process_camera(&mut client, &camera, args.exposure).await?;
            }
        }
    }

    Ok(())
}
