use clap::Parser;
use indi_rs::client::{Client, ClientConfig};
use std::error::Error;
use tracing::info;
use tracing_subscriber::fmt;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// INDI server host
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// INDI server port
    #[arg(short = 'P', long, default_value_t = 7624)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    fmt::init();

    let args = Args::parse();

    info!("Connecting to INDI server at {}:{}", args.host, args.port);
    let config = ClientConfig {
        server_addr: format!("{}:{}", args.host, args.port),
    };
    let client = Client::new(config).await?;
    client.connect().await?;

    info!("Getting properties...");
    let devices = client.get_devices().await?;

    for device in devices {
        if let Some(properties) = client.get_device_properties(&device).await {
            info!("Device: {}", device);
            for (name, prop) in properties {
                info!("  Property: {} = {:?}", name, prop.value);
            }
        }
    }

    Ok(())
}
