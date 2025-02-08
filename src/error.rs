//! Error types for the INDI protocol implementation

use std::fmt;

/// Result type for INDI operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for INDI protocol operations
#[derive(Debug)]
pub enum Error {
    /// Message error
    Message(String),
    /// IO error
    Io(std::io::Error),
    /// Property error
    Property(String),
    /// Connection error
    Connection(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "Message error: {}", msg),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Property(msg) => write!(f, "Property error: {}", msg),
            Error::Connection(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Message(_) => None,
            Error::Io(err) => Some(err),
            Error::Property(_) => None,
            Error::Connection(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
