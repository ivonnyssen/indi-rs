use super::*;

#[tokio::test]
async fn test_client_connect() {
    let mut client = Client::new(ClientConfig {
        host: "localhost".to_string(),
        port: ClientConfig::DEFAULT_PORT,
    });

    assert!(client.connect().await.is_ok());
}

#[tokio::test]
async fn test_client_disconnect() {
    let mut client = Client::new(ClientConfig {
        host: "localhost".to_string(),
        port: ClientConfig::DEFAULT_PORT,
    });

    assert!(client.disconnect().await.is_ok());
}

#[tokio::test]
async fn test_get_properties() {
    let mut client = Client::new(ClientConfig {
        host: "localhost".to_string(),
        port: ClientConfig::DEFAULT_PORT,
    });

    assert!(client.get_properties(None, None).await.is_ok());
}

#[tokio::test]
async fn test_client_state() {
    let state = ClientState::default();
    assert!(state.properties.is_empty());
    assert!(state.last_message.is_none());
}
