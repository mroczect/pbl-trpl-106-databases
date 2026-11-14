use async_trait::async_trait;

use crate::Result;

/// Basic connection liveness and version probe.
#[async_trait]
pub trait Connection: Send + Sync {
    async fn ping(&self) -> Result<()>;

    async fn version(&self) -> Result<String>;
}
