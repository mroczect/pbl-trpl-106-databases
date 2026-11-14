use async_trait::async_trait;

use crate::types::core::{Migration, MigrationStatus};
use crate::Result;

#[async_trait]
pub trait MigrationRunner: Send + Sync {
    async fn run_pending(&self) -> Result<Vec<Migration>>;
    async fn rollback_last(&self) -> Result<Option<Migration>>;
    async fn status(&self) -> Result<Vec<MigrationStatus>>;
}
