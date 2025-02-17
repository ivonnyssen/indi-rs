use super::*;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_client_connect() {
    // Start a mock INDI server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut client = Client::new(ClientConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
    });

    // Spawn a task to accept the connection
    let _handle = tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
    });

    assert!(client.connect().await.is_ok());
}

#[tokio::test]
async fn test_client_disconnect() {
    // Start a mock INDI server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut client = Client::new(ClientConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
    });

    // Spawn a task to accept the connection
    let _handle = tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
    });

    // Connect first
    client.connect().await.unwrap();

    assert!(client.disconnect().await.is_ok());
}

#[tokio::test]
async fn test_get_properties() {
    // Start a mock INDI server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut client = Client::new(ClientConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
    });

    // Spawn a task to accept the connection
    let _handle = tokio::spawn(async move {
        let (_socket, _) = listener.accept().await.unwrap();
    });

    // Connect first
    client.connect().await.unwrap();

    assert!(client.get_properties(None, None).await.is_ok());
}

#[tokio::test]
async fn test_client_state() {
    let state = ClientState::default();
    assert!(state.properties.is_empty());
    assert!(state.last_message.is_none());
}
