use crate::error::Result;
use std::future::Future;

/// INDI message handler trait
///
/// This trait defines the message handling behavior for INDI clients.
/// Implementors must provide a way to send messages to the server.
pub trait MessageHandler {
    /// Send a message to the INDI server
    ///
    /// Returns a Future that resolves to a Result indicating success or failure.
    /// The Future is Send to allow for async execution across thread boundaries.
    fn send_message(&mut self, message: &str) -> impl Future<Output = Result<()>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        pub MessageHandler {}
        impl MessageHandler for MessageHandler {
            fn send_message(&mut self, message: &str) -> impl Future<Output = Result<()>> + Send;
        }
    }
}
