use tokio::io::AsyncWriteExt;

use super::Client;
use crate::error::Result;

impl Client {
    /// Disconnect from the INDI server
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await?;
        }
        Ok(())
    }
}
