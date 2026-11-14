use async_trait::async_trait;

use crate::types::core::HealthDetails;
use crate::Result;

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn health(&self) -> Result<HealthDetails>;
}
