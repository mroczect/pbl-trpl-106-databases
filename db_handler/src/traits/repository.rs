use async_trait::async_trait;

use crate::{Json, Result};

#[async_trait]
pub trait Repository: Send + Sync {
        async fn create(&self, data: &Json) -> Result<Json>;

        async fn get(&self, id: &Json) -> Result<Json>;

        async fn update(&self, id: &Json, data: &Json) -> Result<Json>;

        async fn delete(&self, id: &Json) -> Result<Json>;

            async fn list(&self, filter: &Json) -> Result<Json>;

        async fn count(&self, filter: &Json) -> Result<Json>;
}
