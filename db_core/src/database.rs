use std::time::{Duration, Instant};

use db_handler::{
    constants::HEALTH_TIMEOUT_MS, Connection, DatabaseInfo, DatabaseInfoProvider, DbError,
    HealthCheck, HealthDetails, HealthStatus, PoolConfig, Result,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::{MySql, Transaction as SqlxTransaction};

#[derive(Debug)]
pub struct Database {
    pool: MySqlPool,
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

        Ok(Self { pool })
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

    pub async fn begin(&self) -> Result<SqlxTransaction<'_, MySql>> {
        self.pool.begin().await.map_err(|e| DbError::Database {
            message: e.to_string(),
        })
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
