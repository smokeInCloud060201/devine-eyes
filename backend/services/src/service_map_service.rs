use eyes_devine_shared::{
    ConnectionType, ContainerInfo, ContainerNetworkInfo, ServiceConnection, ServiceEdge,
    ServiceMap, ServiceNode,
};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use chrono::Utc;

pub struct ServiceMapService {
    docker_service: Arc<crate::DockerService>,
}

impl ServiceMapService {
    pub fn new(docker_service: Arc<crate::DockerService>) -> Self {
        Self { docker_service }
    }

    /// Generate service map with all detected connections
    pub async fn generate_service_map(&self) -> Result<ServiceMap> {
        self.generate_service_map_for_service(None).await
    }

    /// Generate service map for a specific service (or all services if service_id is None)
    pub async fn generate_service_map_for_service(
        &self,
        service_id: Option<&str>,
    ) -> Result<ServiceMap> {
        // Get all containers
        let containers = self.docker_service.list_containers().await?;

        // Collect network info and environment for all containers
        let mut container_network_info = HashMap::new();
        let mut container_env_vars = HashMap::new();
        let mut container_stats = HashMap::new();

        for container in &containers {
            // Get network info
            if let Ok(net_info) = self
                .docker_service
                .get_container_network_info(&container.id)
                .await
            {
                container_network_info.insert(container.id.clone(), net_info);
            }

            // Get environment variables
            if let Ok(env_vars) = self
                .docker_service
                .get_container_environment(&container.id)
                .await
            {
                container_env_vars.insert(container.id.clone(), env_vars);
            }

            // Get stats for traffic analysis
            if let Ok(stats) = self.docker_service.get_container_stats(&container.id).await {
                container_stats.insert(container.id.clone(), stats);
            }
        }

        // Build nodes
        let nodes: Vec<ServiceNode> = containers
            .iter()
            .map(|container| {
                let networks = container_network_info
                    .get(&container.id)
                    .map(|info| {
                        info.networks
                            .iter()
                            .map(|n| n.network_name.clone())
                            .collect()
                    })
                    .unwrap_or_default();

                let image_family = Self::extract_image_family(&container.image);

                ServiceNode {
                    container_id: container.id.clone(),
                    container_name: container.name.clone(),
                    image: container.image.clone(),
                    image_family: image_family.clone(),
                    status: container.status.clone(),
                    networks,
                }
            })
            .collect();

        // Build edges (connections)
        let mut edges = Vec::new();
        let mut processed_pairs = HashSet::new();

        // Find the target service if filtering
        let target_service_id = if let Some(svc_id) = service_id {
            // Check if service exists
            if let Some(target_container) = containers.iter().find(|c| c.id == svc_id || c.name == svc_id) {
                Some(target_container.id.clone())
            } else {
                // Service not found, return empty map
                return Ok(ServiceMap {
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    timestamp: Utc::now(),
                });
            }
        } else {
            None
        };

        for (i, source) in containers.iter().enumerate() {
            for (j, target) in containers.iter().enumerate() {
                if i >= j {
                    continue; // Avoid duplicate pairs
                }

                // If filtering by service, only process connections involving that service
                if let Some(ref target_id) = target_service_id {
                    if source.id != *target_id && target.id != *target_id {
                        continue; // Skip pairs that don't involve the target service
                    }
                }

                let pair_key = if source.id < target.id {
                    (source.id.clone(), target.id.clone())
                } else {
                    (target.id.clone(), source.id.clone())
                };

                if processed_pairs.contains(&pair_key) {
                    continue;
                }
                processed_pairs.insert(pair_key.clone());

                // Detect connections
                let connections = self.detect_connections(
                    source,
                    target,
                    &container_network_info,
                    &container_env_vars,
                    &container_stats,
                );

                for connection in connections {
                    edges.push(ServiceEdge {
                        from: connection.source_container_id.clone(),
                        to: connection.target_container_id.clone(),
                        connection_type: connection.connection_type.clone(),
                        confidence: connection.confidence,
                        evidence: connection.evidence.clone(),
                    });
                }
            }
        }

        // Filter nodes to only include the selected service and its connected services
        let filtered_nodes = if let Some(ref target_id) = target_service_id {
            let connected_service_ids: HashSet<String> = edges
                .iter()
                .flat_map(|e| vec![e.from.clone(), e.to.clone()])
                .collect();
            
            nodes
                .into_iter()
                .filter(|n| n.container_id == *target_id || connected_service_ids.contains(&n.container_id))
                .collect()
        } else {
            nodes
        };

        // Filter edges to only include edges between filtered nodes
        let filtered_node_ids: HashSet<String> = filtered_nodes
            .iter()
            .map(|n| n.container_id.clone())
            .collect();
        
        let filtered_edges: Vec<ServiceEdge> = edges
            .into_iter()
            .filter(|e| filtered_node_ids.contains(&e.from) && filtered_node_ids.contains(&e.to))
            .collect();

        Ok(ServiceMap {
            nodes: filtered_nodes,
            edges: filtered_edges,
            timestamp: Utc::now(),
        })
    }

    /// Detect connections between two containers
    fn detect_connections(
        &self,
        source: &ContainerInfo,
        target: &ContainerInfo,
        network_info: &HashMap<String, ContainerNetworkInfo>,
        env_vars: &HashMap<String, Vec<(String, String)>>,
        stats: &HashMap<String, eyes_devine_shared::ContainerStats>,
    ) -> Vec<ServiceConnection> {
        let mut connections = Vec::new();

        // 1. Environment variable based detection
        if let Some(env_vars_source) = env_vars.get(&source.id) {
            for (key, value) in env_vars_source {
                if Self::is_service_reference_env_var(key) {
                    if Self::matches_container(value, target) {
                        connections.push(ServiceConnection {
                            source_container_id: source.id.clone(),
                            source_container_name: source.name.clone(),
                            source_image: source.image.clone(),
                            target_container_id: target.id.clone(),
                            target_container_name: target.name.clone(),
                            target_image: target.image.clone(),
                            connection_type: ConnectionType::EnvironmentVariable,
                            confidence: 0.9,
                            evidence: vec![format!("{}={}", key, value)],
                        });
                    }
                }
            }
        }

        // 2. Same network detection
        if let (Some(source_net), Some(target_net)) =
            (network_info.get(&source.id), network_info.get(&target.id))
        {
            let source_networks: HashSet<String> = source_net
                .networks
                .iter()
                .map(|n| n.network_name.clone())
                .collect();
            let target_networks: HashSet<String> = target_net
                .networks
                .iter()
                .map(|n| n.network_name.clone())
                .collect();

            let common_networks: Vec<String> = source_networks
                .intersection(&target_networks)
                .cloned()
                .collect();

            if !common_networks.is_empty() {
                let evidence: Vec<String> = common_networks
                    .iter()
                    .map(|n| format!("Same network: {}", n))
                    .collect();

                connections.push(ServiceConnection {
                    source_container_id: source.id.clone(),
                    source_container_name: source.name.clone(),
                    source_image: source.image.clone(),
                    target_container_id: target.id.clone(),
                    target_container_name: target.name.clone(),
                    target_image: target.image.clone(),
                    connection_type: ConnectionType::SameNetwork,
                    confidence: 0.7,
                    evidence,
                });
            }
        }

        // 3. Network traffic based detection
        if let (Some(source_stats), Some(target_stats)) =
            (stats.get(&source.id), stats.get(&target.id))
        {
            // If both containers have significant network traffic, they might be communicating
            let source_traffic = source_stats.network_rx_bytes + source_stats.network_tx_bytes;
            let target_traffic = target_stats.network_rx_bytes + target_stats.network_tx_bytes;
            let threshold = 1024 * 1024; // 1MB threshold

            if source_traffic > threshold && target_traffic > threshold {
                connections.push(ServiceConnection {
                    source_container_id: source.id.clone(),
                    source_container_name: source.name.clone(),
                    source_image: source.image.clone(),
                    target_container_id: target.id.clone(),
                    target_container_name: target.name.clone(),
                    target_image: target.image.clone(),
                    connection_type: ConnectionType::NetworkTraffic,
                    confidence: 0.6, // Lower confidence for traffic-based
                    evidence: vec![
                        format!(
                            "Source traffic: {} bytes",
                            source_traffic
                        ),
                        format!(
                            "Target traffic: {} bytes",
                            target_traffic
                        ),
                    ],
                });
            }
        }

        // 4. Host port mapping detection (services communicating via host network)
        // This detects when source connects to target via host-exposed ports
        if let Some(target_net) = network_info.get(&target.id) {
            // Get all host ports exposed by target
            let target_host_ports: Vec<u16> = target_net
                .ports
                .iter()
                .filter_map(|p| p.host_port)
                .collect();

            if !target_host_ports.is_empty() {
                // Check if source references these host ports via environment variables
                if let Some(env_vars_source) = env_vars.get(&source.id) {
                    let mut found_port_matches = Vec::new();
                    let mut found_host_references = Vec::new();

                    for (key, value) in env_vars_source {
                        // Check for localhost/127.0.0.1/host.docker.internal references
                        let value_lower = value.to_lowercase();
                        
                        // Check for host network patterns
                        let is_host_reference = value_lower.contains("localhost")
                            || value_lower.contains("127.0.0.1")
                            || value_lower.contains("host.docker.internal")
                            || value_lower.starts_with("http://")
                            || value_lower.starts_with("https://")
                            || value_lower.starts_with("tcp://");

                        if is_host_reference {
                            // Extract port from URL/address
                            for &host_port in &target_host_ports {
                                let port_str = host_port.to_string();
                                
                                // Check if the port appears in the value
                                // Match patterns like :8080, :8080/, localhost:8080, etc.
                                if value_lower.contains(&format!(":{}", port_str))
                                    || value_lower.ends_with(&port_str)
                                {
                                    found_port_matches.push(host_port);
                                    found_host_references.push(format!("{}={}", key, value));
                                }
                            }
                        }
                    }

                    // If we found matches, create connection with higher confidence
                    if !found_port_matches.is_empty() {
                        let port_evidence: Vec<String> = target_net
                            .ports
                            .iter()
                            .filter_map(|p| {
                                if let Some(host_port) = p.host_port {
                                    if found_port_matches.contains(&host_port) {
                                        Some(format!("{}:{}", p.container_port, host_port))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();

                        let mut evidence = vec![format!(
                            "Target exposes host ports: {}",
                            port_evidence.join(", ")
                        )];
                        evidence.extend(found_host_references);

                        connections.push(ServiceConnection {
                            source_container_id: source.id.clone(),
                            source_container_name: source.name.clone(),
                            source_image: source.image.clone(),
                            target_container_id: target.id.clone(),
                            target_container_name: target.name.clone(),
                            target_image: target.image.clone(),
                            connection_type: ConnectionType::PortMapping,
                            confidence: 0.85, // High confidence - source explicitly references target's host port
                            evidence,
                        });
                    } else if !target_host_ports.is_empty() {
                        // Fallback: target exposes ports but no explicit reference found
                        // Lower confidence - might be connected via host network
                        let port_evidence: Vec<String> = target_net
                            .ports
                            .iter()
                            .map(|p| {
                                if let Some(host_port) = p.host_port {
                                    format!("{}:{}", p.container_port, host_port)
                                } else {
                                    format!("{}", p.container_port)
                                }
                            })
                            .collect();

                        connections.push(ServiceConnection {
                            source_container_id: source.id.clone(),
                            source_container_name: source.name.clone(),
                            source_image: source.image.clone(),
                            target_container_id: target.id.clone(),
                            target_container_name: target.name.clone(),
                            target_image: target.image.clone(),
                            connection_type: ConnectionType::PortMapping,
                            confidence: 0.4, // Lower confidence - ports exist but no explicit reference
                            evidence: vec![format!("Target exposes host ports: {}", port_evidence.join(", "))],
                        });
                    }
                } else if !target_host_ports.is_empty() {
                    // No env vars for source, but target exposes ports
                    let port_evidence: Vec<String> = target_net
                        .ports
                        .iter()
                        .map(|p| {
                            if let Some(host_port) = p.host_port {
                                format!("{}:{}", p.container_port, host_port)
                            } else {
                                format!("{}", p.container_port)
                            }
                        })
                        .collect();

                    connections.push(ServiceConnection {
                        source_container_id: source.id.clone(),
                        source_container_name: source.name.clone(),
                        source_image: source.image.clone(),
                        target_container_id: target.id.clone(),
                        target_container_name: target.name.clone(),
                        target_image: target.image.clone(),
                        connection_type: ConnectionType::PortMapping,
                        confidence: 0.3, // Very low confidence - just ports, no evidence
                        evidence: vec![format!("Target exposes host ports: {}", port_evidence.join(", "))],
                    });
                }
            }
        }

        // 5. Image-based detection (same image family)
        let source_family = Self::extract_image_family(&source.image);
        let target_family = Self::extract_image_family(&target.image);

        if source_family == target_family && !source_family.is_empty() {
            connections.push(ServiceConnection {
                source_container_id: source.id.clone(),
                source_container_name: source.name.clone(),
                source_image: source.image.clone(),
                target_container_id: target.id.clone(),
                target_container_name: target.name.clone(),
                target_image: target.image.clone(),
                connection_type: ConnectionType::ImageBased,
                confidence: 0.4, // Lowest confidence
                evidence: vec![format!("Same image family: {}", source_family)],
            });
        }

        connections
    }

    /// Check if environment variable name suggests a service reference
    fn is_service_reference_env_var(key: &str) -> bool {
        let key_upper = key.to_uppercase();
        key_upper.contains("HOST")
            || key_upper.contains("URL")
            || key_upper.contains("ADDRESS")
            || key_upper.contains("SERVER")
            || key_upper.contains("SERVICE")
            || key_upper == "DATABASE_URL"
            || key_upper == "DB_URL"
            || key_upper == "REDIS_URL"
            || key_upper == "POSTGRES_URL"
            || key_upper == "MYSQL_URL"
            || key_upper == "MONGO_URL"
            || key_upper == "CONSUL_HOST"
            || key_upper == "ETCD_HOST"
    }

    /// Check if a value matches a container (by name, IP, or alias)
    fn matches_container(value: &str, container: &ContainerInfo) -> bool {
        let value_lower = value.to_lowercase();
        let container_name_lower = container.name.to_lowercase();

        // Direct name match
        if value_lower == container_name_lower {
            return true;
        }

        // Match without common prefixes/suffixes
        let normalized_value = value_lower
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_start_matches("tcp://")
            .split(':')
            .next()
            .unwrap_or("")
            .trim_end_matches('/');

        if normalized_value == container_name_lower {
            return true;
        }

        // Match image name
        if value_lower.contains(&container.image.to_lowercase()) {
            return true;
        }

        false
    }

    /// Extract image family from image name (e.g., "postgres:14" -> "postgres")
    fn extract_image_family(image: &str) -> String {
        image
            .split(':')
            .next()
            .unwrap_or(image)
            .split('/')
            .last()
            .unwrap_or(image)
            .to_string()
    }
}

