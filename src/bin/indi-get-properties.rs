use indi_rs::client::{Client, ClientConfig};
use indi_rs::client::connection::Connection;
use indi_rs::client::message::MessageHandler;
use tracing::{debug, info};
use clap::Parser;

/// INDI getProperties command line tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// INDI server host
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// INDI server port
    #[arg(short = 'p', long, default_value_t = 7624)]
    port: u16,

    /// Device name (optional)
    #[arg(short = 'd', long)]
    device: Option<String>,

    /// Property name (optional)
    #[arg(short = 'n', long)]
    property: Option<String>,

    /// Wait time in seconds before disconnecting
    #[arg(short = 'w', long, default_value_t = 2)]
    wait: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    // Create a client config with settings from arguments
    let config = ClientConfig {
        host: args.host.clone(),
        port: args.port,
    };

    info!("Connecting to INDI server at {}:{}", config.host, config.port);
    
    // Connect to the INDI server
    let mut client = Client::new(config).await?;
    
    // Build getProperties message
    let mut message = String::from("<getProperties version='1.7'");
    if let Some(device) = args.device {
        message.push_str(&format!(" device='{}'", device));
    }
    if let Some(property) = args.property {
        message.push_str(&format!(" name='{}'", property));
    }
    message.push_str("/>\n");

    // Send getProperties message
    debug!("Sending message: {}", message.trim());
    client.send_message(&message).await?;

    info!("Sent getProperties message to server");

    // Start reading messages in a separate task
    let client_clone = client.clone();
    tokio::spawn(async move {
        info!("Starting message reader task");
        if let Err(e) = client_clone.read_messages().await {
            eprintln!("Error reading messages: {}", e);
        }
    });

    // Wait for responses
    info!("Waiting for responses...");
    tokio::time::sleep(tokio::time::Duration::from_secs(args.wait)).await;

    // Disconnect from server
    Connection::disconnect(&mut client).await?;
    info!("Disconnected from server");

    Ok(())
}
