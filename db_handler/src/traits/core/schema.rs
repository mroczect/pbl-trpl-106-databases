use async_trait::async_trait;

use crate::types::core::{Index, Table, TableChange};
use crate::Result;

#[async_trait]
pub trait SchemaManager: Send + Sync {
    async fn create_table(&self, table: Table) -> Result<()>;
    async fn drop_table(&self, name: &str, if_exists: bool) -> Result<()>;
    async fn alter_table(&self, name: &str, changes: Vec<TableChange>) -> Result<()>;
    async fn create_index(&self, table: &str, index: Index) -> Result<()>;
    async fn drop_index(&self, table: &str, name: &str, if_exists: bool) -> Result<()>;
    async fn table_exists(&self, name: &str) -> Result<bool>;
    async fn list_tables(&self) -> Result<Vec<String>>;
}
