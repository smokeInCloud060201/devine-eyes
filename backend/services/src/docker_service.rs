use eyes_devine_shared::{ContainerInfo, ContainerStats, TotalStats};
use anyhow::{Context, Result};
use bollard::query_parameters::{ListContainersOptions, LogsOptions, StatsOptions};
use bollard::Docker;
use chrono::Utc;
use futures::StreamExt;

pub struct DockerService {
    docker: Docker,
}

impl DockerService {
    pub async fn new() -> Result<Self> {
        // Try to detect Docker socket path
        // Docker Desktop uses ~/.docker/desktop/docker.sock
        // Standard Docker uses /var/run/docker.sock
        // Also check DOCKER_HOST environment variable
        let docker = if let Ok(docker_host) = std::env::var("DOCKER_HOST") {
            log::info!("Using DOCKER_HOST: {}", docker_host);
            // DOCKER_HOST can be unix:///path or tcp://host:port
            if docker_host.starts_with("unix://") {
                let socket_path = docker_host.strip_prefix("unix://").unwrap_or(&docker_host);
                Docker::connect_with_socket(socket_path, 120, bollard::API_DEFAULT_VERSION)
                    .context(format!("Failed to connect to Docker socket: {}", socket_path))?
            } else {
                Docker::connect_with_http(&docker_host, 120, bollard::API_DEFAULT_VERSION)
                    .context(format!("Failed to connect to Docker host: {}", docker_host))?
            }
        } else if let Ok(home) = std::env::var("HOME") {
            let desktop_path = format!("{}/.docker/desktop/docker.sock", home);
            if std::path::Path::new(&desktop_path).exists() {
                log::info!("Connecting to Docker Desktop socket: {}", desktop_path);
                match Docker::connect_with_socket(&desktop_path, 120, bollard::API_DEFAULT_VERSION) {
                    Ok(d) => d,
                    Err(e) => {
                        log::warn!("Failed to connect to Docker Desktop socket ({}), trying default: {}", desktop_path, e);
                        Docker::connect_with_local_defaults()
                            .context("Failed to connect to Docker daemon")?
                    }
                }
            } else {
                log::info!("Using Docker local defaults");
                Docker::connect_with_local_defaults()
                    .context("Failed to connect to Docker daemon")?
            }
        } else {
            log::info!("Using Docker local defaults");
            Docker::connect_with_local_defaults()
                .context("Failed to connect to Docker daemon")?
        };
        
        // Test the connection by listing containers
        let test_options = ListContainersOptions {
            all: true,
            ..Default::default()
        };
        let test_containers = docker.list_containers(Some(test_options)).await;
        match test_containers {
            Ok(containers) => {
                log::info!("Docker connection successful. Found {} containers on initial connection test", containers.len());
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to connect to Docker daemon: {}. Make sure Docker is running and accessible.", e));
            }
        }
        
        Ok(Self { docker })
    }

    pub async fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        let options = ListContainersOptions {
            all: true,
            ..Default::default()
        };

        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .map_err(|e| {
                log::error!("Docker API error: {:?}", e);
                anyhow::anyhow!("Failed to list containers: {}", e)
            })?;

        log::info!("Docker API returned {} containers", containers.len());
        if containers.is_empty() {
            log::warn!("No containers returned from Docker API, but containers are running. This might indicate a connection or permissions issue.");
        }

        let mut result = Vec::new();
        for (idx, container) in containers.iter().enumerate() {
            log::debug!("Container {}: id={:?}, names={:?}, image={:?}, status={:?}", 
                idx,
                container.id,
                container.names,
                container.image,
                container.status
            );
            
            let container_id = container.id.clone().unwrap_or_else(|| {
                log::warn!("Container at index {} has no ID, using empty string", idx);
                String::new()
            });
            let names = container.names.clone().unwrap_or_default();
            let name = names.first().map(|n| n.trim_start_matches('/')).unwrap_or("unknown").to_string();
            let image = container.image.clone().unwrap_or_else(|| {
                log::warn!("Container {} has no image", name);
                String::new()
            });
            let status = container.status.clone().unwrap_or_else(|| {
                log::warn!("Container {} has no status", name);
                String::new()
            });
            
            log::debug!("Processing container: id={}, name={}, image={}, status={}", 
                container_id, name, image, status);
            
            // Skip containers with empty IDs (shouldn't happen, but be safe)
            if container_id.is_empty() {
                log::warn!("Skipping container with empty ID: name={}, image={}", name, image);
                continue;
            }
            
            result.push(ContainerInfo {
                id: container_id,
                name,
                image,
                status,
                created: container.created.map(|ts| {
                    chrono::DateTime::from_timestamp(ts, 0)
                        .unwrap_or_else(Utc::now)
                }),
            });
        }

        log::info!("Returning {} containers", result.len());
        Ok(result)
    }

    pub async fn get_container_stats(&self, container_id: &str) -> Result<ContainerStats> {
        let options = StatsOptions {
            stream: false,
            ..Default::default()
        };

        let mut stats_stream = self
            .docker
            .stats(container_id, Some(options));

        let stats = stats_stream
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("No stats available"))?
            .context("Failed to read stats")?;

        let cpu_stats = stats.cpu_stats.as_ref().ok_or_else(|| anyhow::anyhow!("No CPU stats"))?;
        let precpu_stats = stats.precpu_stats.as_ref();
        
        let cpu_delta = cpu_stats.cpu_usage.as_ref()
            .and_then(|cu| cu.total_usage
                .and_then(|tu| precpu_stats.and_then(|pc| pc.cpu_usage.as_ref())
                    .and_then(|pcu| pcu.total_usage)
                    .map(|ptu| tu.saturating_sub(ptu))))
            .unwrap_or(0);
        
        let system_delta = cpu_stats.system_cpu_usage
            .and_then(|scu| precpu_stats.and_then(|pc| pc.system_cpu_usage)
                .map(|pscu| scu.saturating_sub(pscu)))
            .unwrap_or(0);

        let cpu_percent = if system_delta > 0 && cpu_delta > 0 {
            (cpu_delta as f64 / system_delta as f64) * 100.0
                * cpu_stats.online_cpus.unwrap_or(1) as f64
        } else {
            0.0
        };

        let memory_stats = stats.memory_stats.as_ref().ok_or_else(|| anyhow::anyhow!("No memory stats"))?;
        let memory_usage = memory_stats.usage.unwrap_or(0);
        let memory_limit = memory_stats.limit.unwrap_or(1);
        let memory_percent = (memory_usage as f64 / memory_limit as f64) * 100.0;

        let network_rx = stats.networks.as_ref()
            .map(|n| n.values().map(|net| net.rx_bytes.unwrap_or(0)).sum())
            .unwrap_or(0);
        let network_tx = stats.networks.as_ref()
            .map(|n| n.values().map(|net| net.tx_bytes.unwrap_or(0)).sum())
            .unwrap_or(0);

        let block_read = stats.blkio_stats.as_ref()
            .and_then(|bs| bs.io_service_bytes_recursive.as_ref())
            .map(|io| io.iter()
                .filter(|s| s.op.as_deref() == Some("Read"))
                .filter_map(|s| s.value)
                .sum::<u64>())
            .unwrap_or(0);
        let block_write = stats.blkio_stats.as_ref()
            .and_then(|bs| bs.io_service_bytes_recursive.as_ref())
            .map(|io| io.iter()
                .filter(|s| s.op.as_deref() == Some("Write"))
                .filter_map(|s| s.value)
                .sum::<u64>())
            .unwrap_or(0);

        let container_name = stats.name.as_ref()
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| container_id.to_string());

        Ok(ContainerStats {
            container_id: container_id.to_string(),
            container_name,
            cpu_usage_percent: cpu_percent,
            memory_usage_bytes: memory_usage,
            memory_limit_bytes: memory_limit,
            memory_usage_percent: memory_percent,
            network_rx_bytes: network_rx,
            network_tx_bytes: network_tx,
            block_read_bytes: block_read,
            block_write_bytes: block_write,
            timestamp: Utc::now(),
        })
    }

    pub async fn get_all_container_stats(&self) -> Result<Vec<ContainerStats>> {
        let containers = self.list_containers().await?;
        let mut all_stats = Vec::new();

        for container in containers {
            if let Ok(stats) = self.get_container_stats(&container.id).await {
                all_stats.push(stats);
            }
        }

        Ok(all_stats)
    }

    pub async fn get_total_stats(&self) -> Result<TotalStats> {
        let all_stats = self.get_all_container_stats().await?;

        let total_containers = all_stats.len();
        let total_cpu = all_stats.iter().map(|s| s.cpu_usage_percent).sum::<f64>() / total_containers.max(1) as f64;
        let total_memory_usage = all_stats.iter().map(|s| s.memory_usage_bytes).sum();
        let total_memory_limit = all_stats.iter().map(|s| s.memory_limit_bytes).sum();
        let total_memory_percent = if total_memory_limit > 0 {
            (total_memory_usage as f64 / total_memory_limit as f64) * 100.0
        } else {
            0.0
        };
        let total_network_rx = all_stats.iter().map(|s| s.network_rx_bytes).sum();
        let total_network_tx = all_stats.iter().map(|s| s.network_tx_bytes).sum();
        let total_block_read = all_stats.iter().map(|s| s.block_read_bytes).sum();
        let total_block_write = all_stats.iter().map(|s| s.block_write_bytes).sum();

        Ok(TotalStats {
            total_containers,
            total_cpu_usage_percent: total_cpu,
            total_memory_usage_bytes: total_memory_usage,
            total_memory_limit_bytes: total_memory_limit,
            total_memory_usage_percent: total_memory_percent,
            total_network_rx_bytes: total_network_rx,
            total_network_tx_bytes: total_network_tx,
            total_block_read_bytes: total_block_read,
            total_block_write_bytes: total_block_write,
            timestamp: Utc::now(),
        })
    }

    pub async fn get_container_logs(
        &self,
        container_id: &str,
        since: Option<i64>,
        until: Option<i64>,
        tail: Option<u64>,
    ) -> Result<Vec<String>> {
        let mut options = LogsOptions {
            stdout: true,
            stderr: true,
            follow: false,
            timestamps: true,
            ..Default::default()
        };

        if let Some(since_ts) = since {
            options.since = since_ts as i32;
        }
        if let Some(until_ts) = until {
            options.until = until_ts as i32;
        }
        if let Some(tail_count) = tail {
            options.tail = tail_count.to_string();
        }

        let mut logs_stream = self
            .docker
            .logs(container_id, Some(options));

        let mut logs = Vec::new();

        while let Some(log_result) = logs_stream.next().await {
            match log_result {
                Ok(log_chunk) => {
                    let log_line = String::from_utf8_lossy(&log_chunk.into_bytes()).to_string();
                    logs.push(log_line);
                }
                Err(e) => {
                    log::warn!("Error reading log chunk: {}", e);
                }
            }
        }

        Ok(logs)
    }

    pub async fn get_container_environment(&self, container_id: &str) -> Result<Vec<(String, String)>> {
        use bollard::query_parameters::InspectContainerOptions;
        let inspect = self
            .docker
            .inspect_container(container_id, None::<InspectContainerOptions>)
            .await
            .context("Failed to inspect container")?;

        let env_vars = inspect
            .config
            .and_then(|config| config.env)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|env_str| {
                let parts: Vec<&str> = env_str.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    Some((env_str, String::new()))
                }
            })
            .collect();

        Ok(env_vars)
    }

    pub async fn get_image_info(&self, image_id: &str) -> Result<Option<eyes_devine_shared::ImageInfo>> {
        let inspect = match self
            .docker
            .inspect_image(image_id)
            .await
        {
            Ok(img) => img,
            Err(e) => {
                log::warn!("Failed to inspect image {}: {}", image_id, e);
                return Ok(None);
            }
        };

        let repo_tags = inspect
            .repo_tags
            .unwrap_or_default();

        let size = inspect.size.map(|s| s as u64).unwrap_or(0);
        
        // bollard's ImageInspect.created is Option<String> in ISO 8601 format
        let created = inspect.created
            .and_then(|ts_str| {
                // Try to parse as RFC3339/ISO8601 timestamp
                chrono::DateTime::parse_from_rfc3339(&ts_str)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            });

        let architecture = inspect.architecture;
        let os = inspect.os;

        // bollard's ImageInspect.id is Option<String>
        let image_id_str = inspect.id
            .unwrap_or_else(|| image_id.to_string());

        Ok(Some(eyes_devine_shared::ImageInfo {
            id: image_id_str,
            repo_tags,
            size,
            created,
            architecture,
            os,
        }))
    }

    pub async fn get_comprehensive_stats(&self) -> Result<eyes_devine_shared::ComprehensiveStats> {
        let containers = self.list_containers().await?;
        let total_containers = containers.len();
        
        let mut containers_up = 0;
        let mut containers_down = 0;
        let mut container_details = Vec::new();

        for container in &containers {
            let is_running = container.status.to_lowercase().contains("up") 
                || container.status.to_lowercase().contains("running");
            
            if is_running {
                containers_up += 1;
            } else {
                containers_down += 1;
            }

            // Get container stats (only if running)
            let stats = if is_running {
                self.get_container_stats(&container.id).await.unwrap_or_else(|e| {
                    log::warn!("Failed to get stats for container {}: {}", container.id, e);
                    // Return empty stats if we can't get them
                    eyes_devine_shared::ContainerStats {
                        container_id: container.id.clone(),
                        container_name: container.name.clone(),
                        cpu_usage_percent: 0.0,
                        memory_usage_bytes: 0,
                        memory_limit_bytes: 0,
                        memory_usage_percent: 0.0,
                        network_rx_bytes: 0,
                        network_tx_bytes: 0,
                        block_read_bytes: 0,
                        block_write_bytes: 0,
                        timestamp: chrono::Utc::now(),
                    }
                })
            } else {
                // Return empty stats for stopped containers
                eyes_devine_shared::ContainerStats {
                    container_id: container.id.clone(),
                    container_name: container.name.clone(),
                    cpu_usage_percent: 0.0,
                    memory_usage_bytes: 0,
                    memory_limit_bytes: 0,
                    memory_usage_percent: 0.0,
                    network_rx_bytes: 0,
                    network_tx_bytes: 0,
                    block_read_bytes: 0,
                    block_write_bytes: 0,
                    timestamp: chrono::Utc::now(),
                }
            };

            // Get environment variables
            let env_vars = self.get_container_environment(&container.id).await.unwrap_or_default();
            let environment: Vec<eyes_devine_shared::ContainerEnvironment> = env_vars
                .into_iter()
                .map(|(key, value)| eyes_devine_shared::ContainerEnvironment { key, value })
                .collect();

            // Get image information
            let image_info = self.get_image_info(&container.image).await.unwrap_or(None);

            container_details.push(eyes_devine_shared::ContainerDetails {
                container_id: container.id.clone(),
                container_name: container.name.clone(),
                image: container.image.clone(),
                status: container.status.clone(),
                is_running,
                environment,
                image_info,
                stats,
                created: container.created,
            });
        }

        // Get total stats
        let total_stats = self.get_total_stats().await?;

        Ok(eyes_devine_shared::ComprehensiveStats {
            total_containers,
            containers_up,
            containers_down,
            total_stats,
            containers: container_details,
            timestamp: chrono::Utc::now(),
        })
    }
}

