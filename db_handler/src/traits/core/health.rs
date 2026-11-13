use async_trait::async_trait;

use crate::{Json, Result};

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn health(&self) -> Result<Json>;
}
