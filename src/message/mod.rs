//! INDI Protocol Message Types
//! 
//! This module implements the INDI (Instrument Neutral Distributed Interface) protocol message types
//! as defined in the [INDI Protocol Specification](https://www.indilib.org/develop/developer-manual/104-indi-protocol.html).
//! 
//! The protocol is XML-based and defines several types of messages for device control and property management:
//! 
//! - Property Definitions (defXXXVector)
//! - Property Updates (setXXXVector, newXXXVector)
//! - Property Queries (getProperties)
//! - Property Deletion (delProperty)
//! - BLOB Transfer Control (enableBLOB)
//! 
//! Each property type (Text, Number, Switch, Light, BLOB) has its own module with consistent structure:
//! - `define.rs`: Property definition types
//! - `set.rs`: Property update request types
//! - `new.rs`: Property update notification types
//! 
//! Common functionality is shared through the `common` module, including the `INDIVector` trait
//! which provides a unified interface for all vector types.
//! 
//! # Examples
//! 
//! ```rust
//! use indi::message::MessageType;
//! use std::str::FromStr;
//! 
//! // Parse a getProperties message
//! let xml = r#"<getProperties version="1.7" device="CCD Simulator"/>"#;
//! let message = MessageType::from_str(xml).unwrap();
//! 
//! // Create and serialize a message
//! let xml = message.to_xml().unwrap();
//! ```
//! 
//! For more examples, see the test module.

use crate::error::{Error, Result};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod blob;
pub mod common;
pub mod light;
pub mod number;
pub mod switch;
pub mod text;

pub use blob::{DefBLOB, DefBLOBVector};
pub use common::vector::INDIVector;
pub use common::message::{Message, MessageType};
pub use light::{DefLight, DefLightVector};
pub use number::{DefNumber, DefNumberVector};
pub use switch::{DefSwitch, DefSwitchVector};
pub use text::{DefText, DefTextVector};
