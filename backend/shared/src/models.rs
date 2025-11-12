use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub container_id: String,
    pub container_name: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_usage_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub block_read_bytes: u64,
    pub block_write_bytes: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalStats {
    pub total_containers: usize,
    pub total_cpu_usage_percent: f64,
    pub total_memory_usage_bytes: u64,
    pub total_memory_limit_bytes: u64,
    pub total_memory_usage_percent: f64,
    pub total_network_rx_bytes: u64,
    pub total_network_tx_bytes: u64,
    pub total_block_read_bytes: u64,
    pub total_block_write_bytes: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerLog {
    pub container_id: String,
    pub container_name: String,
    pub log_line: String,
    pub timestamp: DateTime<Utc>,
    pub stream: String, // "stdout" or "stderr"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    #[serde(default)]
    pub container_id: Option<String>,
    #[serde(default)]
    pub container_name: Option<String>,
    #[serde(default)]
    pub stream: Option<String>, // "stdout" or "stderr"
    #[serde(default)]
    pub since: Option<DateTime<Utc>>,
    #[serde(default)]
    pub until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub limit: Option<u64>,
}

