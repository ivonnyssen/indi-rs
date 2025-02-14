//! Error types for the INDI protocol implementation

use thiserror::Error;

/// Error type for INDI protocol operations
#[derive(Debug, Error)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Message error
    #[error("Message error: {0}")]
    Message(String),
    /// Property error
    #[error("Property error: {0}")]
    Property(String),
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    /// XML error
    #[error("XML error: {0}")]
    Xml(String),
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    /// Not connected to server
    #[error("Not connected to server")]
    NotConnected,
}

/// Result type for INDI operations
pub type Result<T> = std::result::Result<T, Error>;
