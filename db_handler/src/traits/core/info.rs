use async_trait::async_trait;

use crate::types::core::DatabaseInfo;
use crate::Result;

#[async_trait]
pub trait DatabaseInfoProvider: Send + Sync {
    /// Return database metadata: name, version, and pool statistics.
    async fn info(&self) -> Result<DatabaseInfo>;
}
