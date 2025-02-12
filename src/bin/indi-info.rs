use clap::Parser;
use colored::*;
use indi_rs::client::{Client, ClientConfig};
use indi_rs::property::{PropertyState, PropertyValue};
use std::error::Error;
use tracing::{debug, info, Level};

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
    let client = Client::new(config).await?;
    client.connect().await?;

    info!("Getting properties...");
    let devices = client.get_devices().await?;

    for device in devices {
        if let Some(properties) = client.get_device_properties(&device).await {
            debug!(device = %device, property_count = %properties.len(), "Got device properties");
            println!("\n{}", format!("Device: {}", device).bold());
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

    Ok(())
}
