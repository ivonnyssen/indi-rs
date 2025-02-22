use crate::error::Result;
use std::future::Future;

/// INDI connection trait
///
/// This trait defines the connection behavior for INDI clients.
/// Implementors must provide a way to disconnect from the server.
pub trait Connection {
    /// Disconnect from the INDI server
    ///
    /// Returns a Future that resolves to a Result indicating success or failure.
    /// The Future is Send to allow for async execution across thread boundaries.
    fn disconnect(&mut self) -> impl Future<Output = Result<()>> + Send;
}
