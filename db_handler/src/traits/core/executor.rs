use async_trait::async_trait;

use crate::types::core::{ExecuteResult, PageRequest, Paginated, Query, QueryResult, Row};
use crate::Result;

#[async_trait]
pub trait Executor: Send + Sync {
    async fn execute(&self, query: Query) -> Result<ExecuteResult>;

    async fn fetch_one(&self, query: Query) -> Result<Box<dyn Row>>;

    async fn fetch_optional(&self, query: Query) -> Result<Option<Box<dyn Row>>>;

    async fn fetch_all(&self, query: Query) -> Result<QueryResult>;

    async fn fetch_paginated(
        &self,
        query: Query,
        page: PageRequest,
    ) -> Result<Paginated<Box<dyn Row>>>;
}
