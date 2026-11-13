use db_handler::{json, DbError, Json, Result, Transaction};
use sqlx::{MySql, Transaction as SqlxTransaction};

pub(crate) struct MySqlTransaction {
    inner: Option<SqlxTransaction<'static, MySql>>,
}

impl MySqlTransaction {
    pub(crate) const fn new(tx: SqlxTransaction<'static, MySql>) -> Self {
        Self { inner: Some(tx) }
    }

    fn take(&mut self) -> Result<SqlxTransaction<'static, MySql>> {
        self.inner.take().ok_or_else(|| DbError::Database {
            message: "transaction already consumed".into(),
        })
    }
}

#[async_trait::async_trait]
impl Transaction for MySqlTransaction {
    async fn commit(mut self: Box<Self>) -> Result<Json> {
        self.take()?.commit().await.map_err(|e| DbError::Database {
            message: e.to_string(),
        })?;
        Ok(json::empty_object())
    }

    async fn rollback(mut self: Box<Self>) -> Result<Json> {
        self.take()?
            .rollback()
            .await
            .map_err(|e| DbError::Database {
                message: e.to_string(),
            })?;
        Ok(json::empty_object())
    }
}
