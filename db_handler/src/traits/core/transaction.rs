use async_trait::async_trait;

use crate::traits::core::Executor;
use crate::types::core::IsolationLevel;
use crate::Result;

#[async_trait]
pub trait Transaction: Executor {
    async fn commit(self: Box<Self>) -> Result<()>;
    async fn rollback(self: Box<Self>) -> Result<()>;
    async fn savepoint(&mut self, name: &str) -> Result<()>;
    async fn rollback_to(&mut self, name: &str) -> Result<()>;
    async fn release_savepoint(&mut self, name: &str) -> Result<()>;
}

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn Transaction>>;
    async fn begin_with(&self, level: IsolationLevel) -> Result<Box<dyn Transaction>>;
}
