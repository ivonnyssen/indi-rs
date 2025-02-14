//! INDI Protocol Implementation in Rust
//!
//! This crate provides a pure Rust implementation of the INDI (Instrument Neutral Distributed Interface)
//! protocol, designed for controlling astronomical instruments.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod client;
pub mod error;
/// Message handling for the INDI protocol.
/// This module provides functionality for parsing and generating XML messages
/// according to the INDI protocol specification.
pub mod message;
pub mod property;
/// Server implementation for the INDI protocol.
/// This module provides functionality for running an INDI server that can handle
/// device connections and property updates.
pub mod server;

/// Re-export of common types
pub mod prelude {
    pub use crate::client::{Client, ClientConfig};
    pub use crate::error::Error;
    pub use crate::message::Message;
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
