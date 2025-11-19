// Shared types matching the backend API

export interface ContainerStats {
  container_id: string;
  container_name: string;
  cpu_usage_percent: number;
  memory_usage_bytes: number;
  memory_limit_bytes: number;
  memory_usage_percent: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
  block_read_bytes: number;
  block_write_bytes: number;
  timestamp: string; // ISO 8601 string
}

export interface TotalStats {
  total_containers: number;
  total_cpu_usage_percent: number;
  total_memory_usage_bytes: number;
  total_memory_limit_bytes: number;
  total_memory_usage_percent: number;
  total_network_rx_bytes: number;
  total_network_tx_bytes: number;
  total_block_read_bytes: number;
  total_block_write_bytes: number;
  timestamp: string;
}

export interface ContainerInfo {
  id: string;
  name: string;
  image: string;
  status: string;
  created?: string;
}

export interface ContainerLog {
  container_id: string;
  container_name: string;
  log_line: string;
  timestamp: string;
  stream: string; // "stdout" or "stderr"
}

export interface ImageInfo {
  id: string;
  repo_tags: string[];
  size: number;
  created?: string;
  architecture?: string;
  os?: string;
}

// Chart data point
export interface DataPoint {
  timestamp: number; // Unix timestamp in seconds
  cpu: number;
  memory: number;
  network: number; // KB/s
}

// Service Communication Detection Types
export type ConnectionType =
  | 'environment_variable'
  | 'same_network'
  | 'port_mapping'
  | 'network_traffic'
  | 'image_based';

export interface ServiceNode {
  container_id: string;
  container_name: string;
  image: string;
  image_family: string;
  status: string;
  networks: string[];
}

export interface ServiceEdge {
  from: string; // container_id
  to: string; // container_id
  connection_type: ConnectionType;
  confidence: number;
  evidence: string[];
}

export interface ServiceMap {
  nodes: ServiceNode[];
  edges: ServiceEdge[];
  timestamp: string;
}

// HTTP Request Tracking Types
export interface HttpRequest {
  container_id: string;
  container_name: string;
  endpoint: string;
  method: string; // GET, POST, PUT, DELETE, etc.
  http_status: number;
  response_time_ms: number;
  timestamp: string;
}

