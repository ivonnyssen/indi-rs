//! Error types for the INDI protocol implementation

use quick_xml::de::DeError as XmlDeError;
use quick_xml::events::attributes::AttrError;
use quick_xml::Error as XmlError;
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

/// Result type for the crate
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for INDI protocol operations
#[derive(Error, Debug)]
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Message error
    #[error("Message error: {0}")]
    Message(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Property error
    #[error("Property error: {0}")]
    Property(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// XML error
    #[error("XML error: {0}")]
    Xml(#[from] XmlError),

    /// Invalid switch state
    #[error("Invalid switch state: {0}")]
    InvalidSwitchState(String),

    /// UTF-8 error
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] FromUtf8Error),

    /// XML deserialization error
    #[error("XML deserialization error: {0}")]
    XmlDe(#[from] XmlDeError),

    /// XML attribute error
    #[error("XML attribute error: {0}")]
    XmlAttr(#[from] AttrError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
