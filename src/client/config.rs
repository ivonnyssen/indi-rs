/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Host to connect to
    pub host: String,
    /// Port to connect to
    pub port: u16,
}

impl ClientConfig {
    /// Create a new client configuration
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    /// Default INDI server port (7624)
    pub const DEFAULT_PORT: u16 = 7624;
}
