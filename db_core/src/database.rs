use std::time::{Duration, Instant};

use db_handler::validation::validate_pagination;
use db_handler::{
    constants::HEALTH_TIMEOUT_MS, Column, Connection, DatabaseInfo, DatabaseInfoProvider, DbError,
    ExecuteResult, Executor, HealthCheck, HealthDetails, HealthStatus, Index, Migration,
    MigrationRunner, MigrationState, MigrationStatus, PageMeta, PageRequest, Paginated, PoolConfig,
    Query, QueryResult, Result, Row, SchemaManager, Table, TableChange, TransactionManager,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::{MySql, Transaction as SqlxTransaction};

use crate::internal::{bind_value, ensure_migration_table, MySqlRowWrapper};
use crate::transaction::MySqlTransaction;

#[derive(Debug)]
pub struct Database {
    pool: MySqlPool,
    migrations: Vec<Migration>,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        Self::connect_with(url, PoolConfig::default()).await
    }

    pub async fn connect_with(url: &str, config: PoolConfig) -> Result<Self> {
        config.validate()?;
        let mut opts = MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_millis(config.acquire_timeout_ms));

        if config.max_lifetime_secs > 0 {
            opts = opts.max_lifetime(Some(Duration::from_secs(config.max_lifetime_secs)));
        }
        if config.idle_timeout_secs > 0 {
            opts = opts.idle_timeout(Some(Duration::from_secs(config.idle_timeout_secs)));
        }

        let pool = match tokio::time::timeout(
            Duration::from_millis(config.connect_timeout_ms),
            opts.connect(url),
        )
        .await
        {
            Ok(Ok(pool)) => pool,
            Ok(Err(e)) => {
                return Err(DbError::Connection {
                    message: e.to_string(),
                });
            }
            Err(e) => {
                return Err(DbError::Connection {
                    message: format!(
                        "connect timed out after {} ms: {e}",
                        config.connect_timeout_ms
                    ),
                });
            }
        };
        Ok(Self {
            pool,
            migrations: Vec::new(),
        })
    }

    pub async fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();
        let url = match std::env::var("DATABASE_URL") {
            Ok(v) => v,
            Err(e) => {
                return Err(DbError::MissingEnv {
                    name: format!("DATABASE_URL ({e})"),
                });
            }
        };
        Self::connect_with(&url, PoolConfig::from_env()?).await
    }

    #[must_use]
    pub fn with_migrations(mut self, migrations: Vec<Migration>) -> Self {
        self.migrations = migrations;
        self
    }

    pub async fn begin(&self) -> Result<SqlxTransaction<'_, MySql>> {
        self.pool.begin().await.map_err(|e| DbError::Database {
            message: e.to_string(),
        })
    }

    #[must_use]
    pub const fn from_pool(pool: MySqlPool) -> Self {
        Self {
            pool,
            migrations: Vec::new(),
        }
    }

    #[must_use]
    pub const fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[async_trait::async_trait]
impl Connection for Database {
    async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| DbError::Connection {
                message: e.to_string(),
            })
    }

    async fn version(&self) -> Result<String> {
        let row = sqlx::query("SELECT VERSION()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Query {
                message: e.to_string(),
            })?;
        sqlx::Row::try_get(&row, 0).map_err(|e| DbError::Query {
            message: e.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl HealthCheck for Database {
    async fn health(&self) -> Result<HealthDetails> {
        let start = Instant::now();
        let timeout = Duration::from_millis(HEALTH_TIMEOUT_MS);
        let result = tokio::time::timeout(
            timeout,
            sqlx::query("SELECT VERSION()").fetch_one(&self.pool),
        )
        .await;
        let latency_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(match result {
            Ok(Ok(row)) => {
                let version: String = sqlx::Row::try_get(&row, 0).unwrap_or_default();
                let status = if latency_ms > HEALTH_TIMEOUT_MS / 2 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                };
                HealthDetails {
                    status,
                    latency_ms,
                    server_version: version,
                    message: None,
                }
            }
            Ok(Err(e)) => HealthDetails {
                status: HealthStatus::Unhealthy,
                latency_ms,
                server_version: String::new(),
                message: Some(e.to_string()),
            },
            Err(_) => HealthDetails {
                status: HealthStatus::Unhealthy,
                latency_ms,
                server_version: String::new(),
                message: Some(format!(
                    "health check timed out after {HEALTH_TIMEOUT_MS} ms"
                )),
            },
        })
    }
}

#[async_trait::async_trait]
impl DatabaseInfoProvider for Database {
    async fn info(&self) -> Result<DatabaseInfo> {
        let row = sqlx::query("SELECT DATABASE(), VERSION()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Query {
                message: e.to_string(),
            })?;
        let database_name: String = sqlx::Row::try_get(&row, 0).map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;
        let server_version: String = sqlx::Row::try_get(&row, 1).map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;
        let idle = u32::try_from(self.pool.num_idle()).unwrap_or(0);
        Ok(DatabaseInfo {
            database_name,
            server_version,
            pool_size: self.pool.size(),
            pool_idle: idle,
            active_connections: self.pool.size().saturating_sub(idle),
        })
    }
}

#[async_trait::async_trait]
impl Executor for Database {
    async fn execute(&self, query: Query) -> Result<ExecuteResult> {
        let mut q = sqlx::query(&query.sql);
        for p in query.params {
            q = bind_value(q, p);
        }
        let r = q.execute(&self.pool).await.map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;
        Ok(ExecuteResult::new(
            r.rows_affected(),
            Some(r.last_insert_id()).filter(|&id| id > 0),
        ))
    }

    async fn fetch_one(&self, query: Query) -> Result<Box<dyn Row>> {
        let mut q = sqlx::query(&query.sql);
        for p in query.params {
            q = bind_value(q, p);
        }
        let row = q.fetch_one(&self.pool).await.map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;
        Ok(Box::new(MySqlRowWrapper::from_row(&row)?))
    }

    async fn fetch_optional(&self, query: Query) -> Result<Option<Box<dyn Row>>> {
        let mut q = sqlx::query(&query.sql);
        for p in query.params {
            q = bind_value(q, p);
        }
        let row = q
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Query {
                message: e.to_string(),
            })?;
        match row {
            Some(r) => Ok(Some(Box::new(MySqlRowWrapper::from_row(&r)?))),
            None => Ok(None),
        }
    }

    async fn fetch_all(&self, query: Query) -> Result<QueryResult> {
        let mut q = sqlx::query(&query.sql);
        for p in query.params {
            q = bind_value(q, p);
        }
        let rows = q.fetch_all(&self.pool).await.map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;
        let columns: Vec<Column> = rows
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
        validate_pagination(&page)?;
        let limit = i64::try_from(page.per_page).map_err(|e| DbError::Validation {
            message: format!("per_page overflows i64: {e}"),
        })?;
        let offset = i64::try_from(page.offset()).map_err(|e| DbError::Validation {
            message: format!("offset overflows i64: {e}"),
        })?;
        let sql = format!("{} LIMIT ? OFFSET ?", query.sql);
        let mut q = sqlx::query(&sql);
        for p in query.params {
            q = bind_value(q, p);
        }
        q = q.bind(limit).bind(offset);
        let rows = q.fetch_all(&self.pool).await.map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;
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

#[async_trait::async_trait]
impl TransactionManager for Database {
    async fn begin(&self) -> Result<Box<dyn db_handler::Transaction>> {
        let mut conn = self.pool.acquire().await.map_err(|e| DbError::Connection {
            message: e.to_string(),
        })?;
        sqlx::query("START TRANSACTION")
            .execute(&mut *conn)
            .await
            .map(|_| ())
            .map_err(|e| DbError::Database {
                message: e.to_string(),
            })?;
        Ok(Box::new(MySqlTransaction::new(conn)))
    }

    async fn begin_with(
        &self,
        level: db_handler::IsolationLevel,
    ) -> Result<Box<dyn db_handler::Transaction>> {
        use db_handler::IsolationLevel;
        let level_sql = match level {
            IsolationLevel::ReadUncommitted => "READ UNCOMMITTED",
            IsolationLevel::ReadCommitted => "READ COMMITTED",
            IsolationLevel::RepeatableRead => "REPEATABLE READ",
            IsolationLevel::Serializable => "SERIALIZABLE",
        };
        let mut conn = self.pool.acquire().await.map_err(|e| DbError::Connection {
            message: e.to_string(),
        })?;
        crate::internal::exec_raw_text(
            &mut *conn,
            &format!("SET TRANSACTION ISOLATION LEVEL {level_sql}"),
        )
        .await?;
        crate::internal::exec_raw_text(&mut *conn, "START TRANSACTION").await?;
        Ok(Box::new(MySqlTransaction::new(conn)))
    }
}

#[async_trait::async_trait]
impl SchemaManager for Database {
    async fn create_table(&self, table: Table) -> Result<()> {
        let sql = crate::internal::build_create_table(&table)?;
        sqlx::query(&sql)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| DbError::Database {
                message: e.to_string(),
            })
    }

    async fn drop_table(&self, name: &str, if_exists: bool) -> Result<()> {
        db_handler::validation::validate_identifier(name)?;
        let exists = if if_exists { "IF EXISTS " } else { "" };
        let sql = format!("DROP TABLE {exists}`{name}`");
        sqlx::query(&sql)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| DbError::Database {
                message: e.to_string(),
            })
    }

    async fn alter_table(&self, name: &str, changes: Vec<TableChange>) -> Result<()> {
        db_handler::validation::validate_identifier(name)?;
        if changes.is_empty() {
            return Ok(());
        }
        let mut parts = Vec::new();
        for c in &changes {
            parts.push(crate::internal::table_change_sql(c)?);
        }
        let sql = format!("ALTER TABLE `{name}` {}", parts.join(", "));
        sqlx::query(&sql)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| DbError::Database {
                message: e.to_string(),
            })
    }

    async fn create_index(&self, table: &str, index: Index) -> Result<()> {
        db_handler::validation::validate_identifier(table)?;
        db_handler::validation::validate_identifier(&index.name)?;
        for col in &index.columns {
            db_handler::validation::validate_identifier(col)?;
        }
        let cols = index
            .columns
            .iter()
            .map(|c| format!("`{c}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let unique = if index.unique { "UNIQUE " } else { "" };
        let sql = format!(
            "CREATE {unique}INDEX `{}` ON `{table}` ({cols})",
            index.name
        );
        sqlx::query(&sql)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| DbError::Database {
                message: e.to_string(),
            })
    }

    async fn drop_index(&self, table: &str, name: &str, if_exists: bool) -> Result<()> {
        db_handler::validation::validate_identifier(table)?;
        db_handler::validation::validate_identifier(name)?;
        if if_exists {
            let sql_check =
                "SELECT COUNT(*) FROM information_schema.statistics WHERE table_schema = DATABASE() AND table_name = ? AND index_name = ?";
            let count = sqlx::query_scalar::<_, i64>(sql_check)
                .bind(table)
                .bind(name)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DbError::Query {
                    message: e.to_string(),
                })?;
            if count == 0 {
                return Ok(());
            }
        }
        let sql = format!("DROP INDEX `{name}` ON `{table}`");
        sqlx::query(&sql)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| DbError::Database {
                message: e.to_string(),
            })
    }

    async fn table_exists(&self, name: &str) -> Result<bool> {
        db_handler::validation::validate_identifier(name)?;
        let sql =
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = ?";
        let count = sqlx::query_scalar::<_, i64>(sql)
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Query {
                message: e.to_string(),
            })?;
        Ok(count > 0)
    }

    async fn list_tables(&self) -> Result<Vec<String>> {
        let sql = "SELECT CAST(table_name AS CHAR) AS table_name \
                   FROM information_schema.tables \
                   WHERE table_schema = DATABASE()";
        sqlx::query_scalar::<_, String>(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Query {
                message: e.to_string(),
            })
    }
}

#[async_trait::async_trait]
impl MigrationRunner for Database {
    async fn run_pending(&self) -> Result<Vec<Migration>> {
        ensure_migration_table(&self.pool).await?;
        let applied_versions: Vec<i64> = sqlx::query_scalar("SELECT version FROM `_migrations`")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Query {
                message: e.to_string(),
            })?;

        let mut applied = Vec::new();
        for m in &self.migrations {
            let version_i64 = i64::try_from(m.version).map_err(|e| DbError::Validation {
                message: format!("migration version {} does not fit in i64: {e}", m.version),
            })?;
            if !applied_versions.contains(&version_i64) {
                sqlx::query(&m.up_sql)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                    .map_err(|e| DbError::Database {
                        message: format!("failed to apply migration {}: {}", m.version, e),
                    })?;
                sqlx::query("INSERT INTO `_migrations` (version, name) VALUES (?, ?)")
                    .bind(version_i64)
                    .bind(&m.name)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                    .map_err(|e| DbError::Database {
                        message: e.to_string(),
                    })?;
                applied.push(m.clone());
            }
        }
        Ok(applied)
    }

    async fn rollback_last(&self) -> Result<Option<Migration>> {
        ensure_migration_table(&self.pool).await?;
        let row: Option<(i64, String)> =
            sqlx::query_as("SELECT version, name FROM `_migrations` ORDER BY version DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| DbError::Query {
                    message: e.to_string(),
                })?;

        if let Some((version, _name)) = row {
            let version_u64 = u64::try_from(version).map_err(|e| DbError::Query {
                message: format!("migration version is negative or too large: {e}"),
            })?;
            if let Some(m) = self.migrations.iter().find(|m| m.version == version_u64) {
                if let Some(down_sql) = &m.down_sql {
                    sqlx::query(down_sql)
                        .execute(&self.pool)
                        .await
                        .map(|_| ())
                        .map_err(|e| DbError::Database {
                            message: format!("failed to rollback migration {}: {}", m.version, e),
                        })?;
                }
                sqlx::query("DELETE FROM `_migrations` WHERE version = ?")
                    .bind(version)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                    .map_err(|e| DbError::Database {
                        message: e.to_string(),
                    })?;
                return Ok(Some(m.clone()));
            }
        }
        Ok(None)
    }

    async fn status(&self) -> Result<Vec<MigrationStatus>> {
        ensure_migration_table(&self.pool).await?;
        let applied: std::collections::HashMap<i64, String> =
            sqlx::query_as("SELECT version, CAST(applied_at AS CHAR) FROM `_migrations`")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DbError::Query {
                    message: e.to_string(),
                })?
                .into_iter()
                .collect();

        let mut statuses = Vec::new();
        for m in &self.migrations {
            let applied_at = i64::try_from(m.version)
                .ok()
                .and_then(|v| applied.get(&v))
                .cloned();
            let state = if applied_at.is_some() {
                MigrationState::Applied
            } else {
                MigrationState::Pending
            };
            statuses.push(MigrationStatus::new(m.clone(), state, applied_at));
        }
        Ok(statuses)
    }
}
