#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

//! INDI Protocol Implementation in Rust
//!
//! This crate provides a Rust implementation of the INDI (Instrument Neutral Distributed Interface) protocol,
//! commonly used in astronomy for device control and automation.
//!
//! # Features
//! - Async client implementation
//! - XML message handling
//! - Property management
//! - Error handling
//! - Logging support

/// Client implementation for INDI protocol
pub mod client;
/// Error types and handling
pub mod error;
/// Message types and handling
pub mod message;
/// Property types and handling
pub mod property;
/// Server implementation for the INDI protocol.
/// This module provides functionality for running an INDI server that can handle
/// device connections and property updates.
pub mod server;

/// Common types and traits
pub mod prelude {
    pub use crate::client::{Client, ClientConfig};
    pub use crate::error::Error;
    pub use crate::message::MessageType;
    pub use crate::property::{Property, PropertyPerm, PropertyState};
    pub use crate::server::{Server, ServerConfig};
}

/// Result type for INDI operations
pub type Result<T> = std::result::Result<T, error::Error>;

/// Version of the INDI protocol implemented by this library
pub const PROTOCOL_VERSION: &str = "1.7";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, "1.7");
    }
}
