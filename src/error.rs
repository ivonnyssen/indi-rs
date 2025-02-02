//! Error types for the INDI protocol implementation

use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

/// Error type for INDI operations
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// UTF-8 conversion error
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// String conversion error
    #[error("String conversion error: {0}")]
    FromUtf8(#[from] FromUtf8Error),

    /// XML parsing error
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// Property error
    #[error("Property error: {0}")]
    Property(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Message error
    #[error("Message error: {0}")]
    Message(String),
}

/// Shorthand for Result with our Error type
pub type Result<T> = std::result::Result<T, Error>;
