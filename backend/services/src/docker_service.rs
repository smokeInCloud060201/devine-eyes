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
        let docker = Docker::connect_with_local_defaults()
            .context("Failed to connect to Docker daemon")?;
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
            .context("Failed to list containers")?;

        let mut result = Vec::new();
        for container in containers {
            let names = container.names.unwrap_or_default();
            let name = names.first().map(|n| n.trim_start_matches('/')).unwrap_or("unknown").to_string();
            
            result.push(ContainerInfo {
                id: container.id.unwrap_or_default(),
                name,
                image: container.image.unwrap_or_default(),
                status: container.status.unwrap_or_default(),
                created: container.created.map(|ts| {
                    chrono::DateTime::from_timestamp(ts, 0)
                        .unwrap_or_else(Utc::now)
                }),
            });
        }

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
}

