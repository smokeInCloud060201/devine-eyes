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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerEnvironment {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub size: u64,
    pub created: Option<DateTime<Utc>>,
    pub architecture: Option<String>,
    pub os: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDetails {
    pub container_id: String,
    pub container_name: String,
    pub image: String,
    pub status: String,
    pub is_running: bool,
    pub environment: Vec<ContainerEnvironment>,
    pub image_info: Option<ImageInfo>,
    pub stats: ContainerStats,
    pub created: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveStats {
    pub total_containers: usize,
    pub containers_up: usize,
    pub containers_down: usize,
    pub total_stats: TotalStats,
    pub containers: Vec<ContainerDetails>,
    pub timestamp: DateTime<Utc>,
}

// Service Communication Detection Models

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConnectionType {
    #[serde(rename = "environment_variable")]
    EnvironmentVariable,
    #[serde(rename = "same_network")]
    SameNetwork,
    #[serde(rename = "port_mapping")]
    PortMapping,
    #[serde(rename = "network_traffic")]
    NetworkTraffic,
    #[serde(rename = "image_based")]
    ImageBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub network_name: String,
    pub network_id: String,
    pub ip_address: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub host_port: Option<u16>,
    pub protocol: String, // "tcp", "udp"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerNetworkInfo {
    pub container_id: String,
    pub container_name: String,
    pub networks: Vec<NetworkInfo>,
    pub ports: Vec<PortMapping>,
    pub ip_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConnection {
    pub source_container_id: String,
    pub source_container_name: String,
    pub source_image: String,
    pub target_container_id: String,
    pub target_container_name: String,
    pub target_image: String,
    pub connection_type: ConnectionType,
    pub confidence: f64, // 0.0 to 1.0
    pub evidence: Vec<String>, // e.g., ["DB_HOST=postgres", "Same network: bridge"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNode {
    pub container_id: String,
    pub container_name: String,
    pub image: String,
    pub image_family: String, // e.g., "postgres", "redis", "nginx"
    pub status: String,
    pub networks: Vec<String>, // Network names
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEdge {
    pub from: String, // container_id
    pub to: String,   // container_id
    pub connection_type: ConnectionType,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMap {
    pub nodes: Vec<ServiceNode>,
    pub edges: Vec<ServiceEdge>,
    pub timestamp: DateTime<Utc>,
}

// HTTP Request Tracking Models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub container_id: String,
    pub container_name: String,
    pub endpoint: String,        // e.g., "/api/users", "/health"
    pub method: String,          // e.g., "GET", "POST", "PUT", "DELETE"
    pub http_status: u16,        // e.g., 200, 404, 500
    pub response_time_ms: f64,   // Response time in milliseconds
    pub timestamp: DateTime<Utc>,
}

