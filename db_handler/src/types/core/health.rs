use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDetails {
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub server_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
