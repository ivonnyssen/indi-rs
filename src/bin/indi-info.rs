use clap::Parser;
use indi_rs::client::{Client, ClientConfig};
use std::error::Error;

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
    let args = Args::parse();

    println!("Connecting to INDI server at {}:{}", args.host, args.port);
    let config = ClientConfig {
        server_addr: format!("{}:{}", args.host, args.port),
    };
    let client = Client::new(config).await?;
    client.connect().await?;

    println!("Getting properties...");
    let devices = client.get_devices().await?;

    if devices.is_empty() {
        println!("No devices found");
    } else {
        println!("\nFound {} devices:", devices.len());
        for device in devices {
            println!("\nDevice: {}", device);

            if let Some(props) = client.get_device_properties(&device).await {
                for prop in props.values() {
                    println!("  Property: {} ({:?})", prop.name, prop.state);
                    println!("    Value: {:?}", prop.value);
                }
            } else {
                println!("  Failed to get properties");
            }
        }
    }

    Ok(())
}
