use async_trait::async_trait;

use crate::traits::core::Transaction;
use crate::{Json, Result};

#[async_trait]
pub trait Connection: Send + Sync {
        async fn ping(&self) -> Result<Json>;

        async fn version(&self) -> Result<Json>;

        async fn begin(&self) -> Result<Box<dyn Transaction>>;
}
