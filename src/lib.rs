//! INDI Protocol Implementation in Rust
//! 
//! This crate provides a pure Rust implementation of the INDI (Instrument Neutral Distributed Interface)
//! protocol, designed for controlling astronomical instruments.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod error;
pub mod message;
pub mod property;
pub mod client;
pub mod server;

/// Re-export of common types
pub mod prelude {
    pub use crate::error::Error;
    pub use crate::message::Message;
    pub use crate::property::{Property, PropertyState, PropertyPerm};
    pub use crate::client::{Client, ClientConfig};
    pub use crate::server::{Server, ServerConfig, DeviceDriver};
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
        assert!(!PROTOCOL_VERSION.is_empty());
    }
}
