
use crate::internal::{bind_value, MySqlRowWrapper};
use db_handler::validation::validate_identifier;
use db_handler::{
    DbError, ExecuteResult, Executor, PageMeta, PageRequest, Paginated, Query, QueryResult, Result,
    Row,
};
use sqlx::mysql::MySql;
use sqlx::pool::PoolConnection;
use tokio::sync::Mutex;

pub struct MySqlTransaction {
    conn: Mutex<Option<PoolConnection<MySql>>>,
}

impl std::fmt::Debug for MySqlTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.conn.try_lock().map_or("locked", |guard| {
            if guard.is_some() {
                "active"
            } else {
                "consumed"
            }
        });

        f.debug_struct("MySqlTransaction")
            .field("state", &state)
            .finish()
    }
}

impl Drop for MySqlTransaction {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.conn.try_lock() {
            if let Some(mut conn) = guard.take() {
                conn.close_on_drop();
                drop(conn);
            }
        }
    }
}

impl MySqlTransaction {
    pub(crate) fn new(conn: PoolConnection<MySql>) -> Self {
        Self {
            conn: Mutex::new(Some(conn)),
        }
    }

    async fn take_conn(&self) -> Result<PoolConnection<MySql>> {
        self.conn
            .lock()
            .await
            .take()
            .ok_or_else(|| DbError::Database {
                message: "transaction already consumed".into(),
            })
    }

    async fn run_savepoint_sql(&self, sql: String) -> Result<()> {
        let mut guard = self.conn.lock().await;

        let conn = guard.take().ok_or_else(|| DbError::Database {
            message: "transaction already consumed".into(),
        })?;

        let conn = crate::internal::exec_raw_text(conn, sql).await?;

        *guard = Some(conn);
        drop(guard);

        Ok(())
    }
}

#[async_trait::async_trait]
impl db_handler::Transaction for MySqlTransaction {
    async fn commit(self: Box<Self>) -> Result<()> {
        let mut conn = self.take_conn().await?;

        let result = sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map(|_| ())
            .map_err(|e| DbError::Database {
                message: e.to_string(),
            });

        if result.is_err() {
            conn.close_on_drop();
        }

        drop(conn);
        result
    }

    async fn rollback(self: Box<Self>) -> Result<()> {
        let mut conn = self.take_conn().await?;

        let result = sqlx::query("ROLLBACK")
            .execute(&mut *conn)
            .await
            .map(|_| ())
            .map_err(|e| DbError::Database {
                message: e.to_string(),
            });

        if result.is_err() {
            conn.close_on_drop();
        }

        drop(conn);
        result
    }

    async fn savepoint(&mut self, name: &str) -> Result<()> {
        validate_identifier(name)?;
        self.run_savepoint_sql(format!("SAVEPOINT `{name}`"))
            .await
    }

    async fn rollback_to(&mut self, name: &str) -> Result<()> {
        validate_identifier(name)?;
        self.run_savepoint_sql(format!("ROLLBACK TO SAVEPOINT `{name}`"))
            .await
    }

    async fn release_savepoint(&mut self, name: &str) -> Result<()> {
        validate_identifier(name)?;
        self.run_savepoint_sql(format!("RELEASE SAVEPOINT `{name}`"))
            .await
    }
}

#[async_trait::async_trait]
impl Executor for MySqlTransaction {
    async fn execute(&self, query: Query) -> Result<ExecuteResult> {
        let mut guard = self.conn.lock().await;

        let conn = guard.as_mut().ok_or_else(|| DbError::Database {
            message: "transaction already consumed".into(),
        })?;

        let mut q = sqlx::query(&query.sql);

        for p in query.params {
            q = bind_value(q, p);
        }

        let r = q.execute(&mut **conn).await.map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;

        drop(guard);

        Ok(ExecuteResult::new(
            r.rows_affected(),
            Some(r.last_insert_id()).filter(|&id| id > 0),
        ))
    }

    async fn fetch_one(&self, query: Query) -> Result<Box<dyn Row>> {
        let mut guard = self.conn.lock().await;

        let conn = guard.as_mut().ok_or_else(|| DbError::Database {
            message: "transaction already consumed".into(),
        })?;

        let mut q = sqlx::query(&query.sql);

        for p in query.params {
            q = bind_value(q, p);
        }

        let row = q.fetch_one(&mut **conn).await.map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;

        drop(guard);

        Ok(Box::new(MySqlRowWrapper::from_row(&row)?))
    }

    async fn fetch_optional(&self, query: Query) -> Result<Option<Box<dyn Row>>> {
        let mut guard = self.conn.lock().await;

        let conn = guard.as_mut().ok_or_else(|| DbError::Database {
            message: "transaction already consumed".into(),
        })?;

        let mut q = sqlx::query(&query.sql);

        for p in query.params {
            q = bind_value(q, p);
        }

        let row = q
            .fetch_optional(&mut **conn)
            .await
            .map_err(|e| DbError::Query {
                message: e.to_string(),
            })?;

        drop(guard);

        match row {
            Some(r) => Ok(Some(Box::new(MySqlRowWrapper::from_row(&r)?))),
            None => Ok(None),
        }
    }

    async fn fetch_all(&self, query: Query) -> Result<QueryResult> {
        let mut guard = self.conn.lock().await;

        let conn = guard.as_mut().ok_or_else(|| DbError::Database {
            message: "transaction already consumed".into(),
        })?;

        let mut q = sqlx::query(&query.sql);

        for p in query.params {
            q = bind_value(q, p);
        }

        let rows = q.fetch_all(&mut **conn).await.map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;

        drop(guard);

        let columns = rows
            .first()
            .map(MySqlRowWrapper::columns_from_row)
            .unwrap_or_default();

        let mut items: Vec<Box<dyn Row>> = Vec::with_capacity(rows.len());

        for r in &rows {
            items.push(Box::new(MySqlRowWrapper::from_row(r)?));
        }

        Ok(QueryResult::new(items, columns))
    }

    async fn fetch_paginated(
        &self,
        query: Query,
        page: PageRequest,
    ) -> Result<Paginated<Box<dyn Row>>> {
        db_handler::validation::validate_pagination(&page)?;

        let limit = i64::try_from(page.per_page).map_err(|e| DbError::Validation {
            message: format!("per_page overflows i64: {e}"),
        })?;

        let offset = i64::try_from(page.offset()).map_err(|e| DbError::Validation {
            message: format!("offset overflows i64: {e}"),
        })?;

        let sql = format!("{} LIMIT ? OFFSET ?", query.sql);

        let mut guard = self.conn.lock().await;

        let conn = guard.as_mut().ok_or_else(|| DbError::Database {
            message: "transaction already consumed".into(),
        })?;

        let mut q = sqlx::query(&sql);

        for p in query.params {
            q = bind_value(q, p);
        }

        q = q.bind(limit).bind(offset);

        let rows = q.fetch_all(&mut **conn).await.map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;

        drop(guard);

        let mut items: Vec<Box<dyn Row>> = Vec::with_capacity(rows.len());

        for r in &rows {
            items.push(Box::new(MySqlRowWrapper::from_row(r)?));
        }

        let page_len = u64::try_from(items.len()).unwrap_or(u64::MAX);
        let total = page.offset().saturating_add(page_len);
        let meta = PageMeta::new(page.page, page.per_page, total);

        Ok(Paginated { items, meta })
    }
}
