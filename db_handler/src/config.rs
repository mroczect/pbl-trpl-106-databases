use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_connections: u32,

    pub min_connections: u32,

    pub acquire_timeout_ms: u64,

    pub connect_timeout_ms: u64,

    pub max_lifetime_secs: u64,

    pub idle_timeout_secs: u64,
}

impl PoolConfig {
    pub fn validate(&self) -> crate::Result<()> {
        if self.max_connections == 0 {
            return Err(crate::DbError::Validation {
                message: "max_connections must be > 0".into(),
            });
        }
        if self.min_connections > self.max_connections {
            return Err(crate::DbError::Validation {
                message: format!(
                    "min_connections ({}) cannot exceed max_connections ({})",
                    self.min_connections, self.max_connections
                ),
            });
        }
        if self.acquire_timeout_ms == 0 {
            return Err(crate::DbError::Validation {
                message: "acquire_timeout_ms must be > 0".into(),
            });
        }
        if self.connect_timeout_ms == 0 {
            return Err(crate::DbError::Validation {
                message: "connect_timeout_ms must be > 0".into(),
            });
        }
        Ok(())
    }

    pub fn from_env() -> crate::Result<Self> {
        use std::env;

        fn parse_or_default<T>(name: &str, default: T) -> crate::Result<T>
        where
            T: std::str::FromStr,
            T::Err: std::fmt::Display,
        {
            match env::var(name) {
                Ok(val) => val.parse::<T>().map_err(|e| crate::DbError::InvalidEnv {
                    name: name.to_string(),
                    reason: format!("{e}"),
                }),
                Err(env::VarError::NotPresent) => Ok(default),
                Err(env::VarError::NotUnicode(_)) => Err(crate::DbError::InvalidEnv {
                    name: name.to_string(),
                    reason: "value is not valid UTF-8".into(),
                }),
            }
        }

        let d = Self::default();
        let config = Self {
            max_connections: parse_or_default("DATABASE_MAX_CONNECTIONS", d.max_connections)?,
            min_connections: parse_or_default("DATABASE_MIN_CONNECTIONS", d.min_connections)?,
            acquire_timeout_ms: parse_or_default(
                "DATABASE_ACQUIRE_TIMEOUT_MS",
                d.acquire_timeout_ms,
            )?,
            connect_timeout_ms: parse_or_default(
                "DATABASE_CONNECT_TIMEOUT_MS",
                d.connect_timeout_ms,
            )?,
            max_lifetime_secs: parse_or_default("DATABASE_MAX_LIFETIME_SECS", d.max_lifetime_secs)?,
            idle_timeout_secs: parse_or_default("DATABASE_IDLE_TIMEOUT_SECS", d.idle_timeout_secs)?,
        };

        config.validate()?;
        Ok(config)
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 0,
            acquire_timeout_ms: 30_000,
            connect_timeout_ms: 10_000,
            max_lifetime_secs: 1_800,
            idle_timeout_secs: 600,
        }
    }
}
