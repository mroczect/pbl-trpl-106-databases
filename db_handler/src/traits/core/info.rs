use async_trait::async_trait;

use crate::{Json, Result};

#[async_trait]
pub trait DatabaseInfoProvider: Send + Sync {
    async fn info(&self) -> Result<Json>;
}
