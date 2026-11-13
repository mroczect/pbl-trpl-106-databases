use std::time::{Duration, Instant};

use db_handler::{
    constants::HEALTH_TIMEOUT_MS, json, Connection, DatabaseInfo, DatabaseInfoProvider, DbError,
    HealthCheck, HealthDetails, HealthStatus, Json, Result, Transaction,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

#[derive(Debug)]
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await
            .map_err(|e| DbError::Connection {
                message: e.to_string(),
            })?;
        Ok(Self { pool })
    }

    #[must_use]
    pub const fn from_pool(pool: MySqlPool) -> Self {
        Self { pool }
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
impl HealthCheck for Database {
    async fn health(&self) -> Result<Json> {
        let start = Instant::now();
        let timeout = Duration::from_millis(HEALTH_TIMEOUT_MS);
        let result = tokio::time::timeout(
            timeout,
            sqlx::query("SELECT VERSION()").fetch_one(&self.pool),
        )
        .await;

        let latency_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        let details = match result {
            Ok(Ok(row)) => {
                let version: String = sqlx::Row::try_get(&row, 0).unwrap_or_default();
                HealthDetails {
                    status: HealthStatus::Healthy,
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
        };

        json::to_json(&details)
    }
}

#[async_trait::async_trait]
impl DatabaseInfoProvider for Database {
    async fn info(&self) -> Result<Json> {
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
        let info = DatabaseInfo {
            database_name,
            server_version,
            pool_size: self.pool.size(),
            pool_idle: idle,
            active_connections: self.pool.size().saturating_sub(idle),
        };
        json::to_json(&info)
    }
}

#[async_trait::async_trait]
impl Connection for Database {
    async fn ping(&self) -> Result<Json> {
        let _ = sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Connection {
                message: e.to_string(),
            })?;
        Ok(json::empty_object())
    }

    async fn version(&self) -> Result<Json> {
        let row = sqlx::query("SELECT VERSION()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Query {
                message: e.to_string(),
            })?;
        let version: String = sqlx::Row::try_get(&row, 0).map_err(|e| DbError::Query {
            message: e.to_string(),
        })?;
        Ok(serde_json::json!({ "version": version }))
    }

    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        let tx = self.pool.begin().await.map_err(|e| DbError::Database {
            message: e.to_string(),
        })?;
        Ok(Box::new(crate::transaction::MySqlTransaction::new(tx)))
    }
}
