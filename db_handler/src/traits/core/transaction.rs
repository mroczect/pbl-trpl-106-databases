use async_trait::async_trait;

use crate::{Json, Result};

#[async_trait]
pub trait Transaction: Send + Sync {
        async fn commit(self: Box<Self>) -> Result<Json>;

        async fn rollback(self: Box<Self>) -> Result<Json>;
}
