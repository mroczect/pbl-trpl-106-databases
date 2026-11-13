use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub database_name: String,
    pub server_version: String,
    pub pool_size: u32,
    pub pool_idle: u32,
    pub active_connections: u32,
}
