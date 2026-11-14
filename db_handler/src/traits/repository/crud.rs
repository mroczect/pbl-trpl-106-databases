use async_trait::async_trait;

use crate::types::core::{Filter, InsertInput, PageRequest, Paginated, UpdateInput, Value};
use crate::Result;

#[async_trait]
pub trait CrudRepository<T>: Send + Sync
where
    T: Send + Sync + 'static,
{
    async fn create(&self, input: InsertInput) -> Result<T>;
    async fn get_by_id(&self, id: &Value) -> Result<Option<T>>;
    async fn update(&self, id: &Value, input: UpdateInput) -> Result<T>;
    async fn delete(&self, id: &Value) -> Result<()>;
    async fn list(&self, filter: Filter, page: PageRequest) -> Result<Paginated<T>>;
    async fn count(&self, filter: Filter) -> Result<u64>;
}
