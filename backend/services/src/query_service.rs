use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder, QuerySelect};
use chrono::{DateTime, Utc, FixedOffset};
use anyhow::Result;
use eyes_devine_shared::{ContainerStats, ContainerInfo, ImageInfo, HttpRequest};
use crate::entity::{container_stats, container_info, docker_images, image_versions, http_requests};

pub struct QueryService {
    db: DatabaseConnection,
}

impl QueryService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get latest stats for a specific container
    pub async fn get_latest_container_stats(
        &self,
        container_id: &str,
    ) -> Result<Option<ContainerStats>> {
        let stats = container_stats::Entity::find()
            .filter(container_stats::Column::ContainerId.eq(container_id))
            .order_by_desc(container_stats::Column::Timestamp)
            .limit(1)
            .one(&self.db)
            .await?;

        Ok(stats.map(|s| Self::entity_to_container_stats(&s)))
    }

    /// Get latest stats for all containers
    pub async fn get_latest_all_container_stats(&self) -> Result<Vec<ContainerStats>> {
        // Get distinct container IDs first
        let containers = container_info::Entity::find()
            .order_by_desc(container_info::Column::CollectedAt)
            .all(&self.db)
            .await?;

        let mut latest_stats = Vec::new();
        for container in containers {
            if let Ok(Some(stats)) = self.get_latest_container_stats(&container.container_id).await {
                latest_stats.push(stats);
            }
        }

        Ok(latest_stats)
    }

    /// Get historical stats for a container within a time range
    pub async fn get_container_stats_history(
        &self,
        container_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<ContainerStats>> {
        let fixed_offset = FixedOffset::east_opt(0).unwrap();
        
        let mut query = container_stats::Entity::find()
            .filter(container_stats::Column::ContainerId.eq(container_id));

        if let Some(from_dt) = from {
            let from_tz = from_dt.with_timezone(&fixed_offset);
            query = query.filter(container_stats::Column::Timestamp.gte(from_tz));
        }

        if let Some(to_dt) = to {
            let to_tz = to_dt.with_timezone(&fixed_offset);
            query = query.filter(container_stats::Column::Timestamp.lte(to_tz));
        }

        query = query.order_by_desc(container_stats::Column::Timestamp);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let stats = query.all(&self.db).await?;

        Ok(stats.iter().map(|s| Self::entity_to_container_stats(s)).collect())
    }

    /// Get aggregated total stats from latest container stats
    pub async fn get_total_stats(&self) -> Result<eyes_devine_shared::TotalStats> {
        let all_stats = self.get_latest_all_container_stats().await?;

        let total_containers = all_stats.len();
        if total_containers == 0 {
            return Ok(eyes_devine_shared::TotalStats {
                total_containers: 0,
                total_cpu_usage_percent: 0.0,
                total_memory_usage_bytes: 0,
                total_memory_limit_bytes: 0,
                total_memory_usage_percent: 0.0,
                total_network_rx_bytes: 0,
                total_network_tx_bytes: 0,
                total_block_read_bytes: 0,
                total_block_write_bytes: 0,
                timestamp: Utc::now(),
            });
        }

        let total_cpu = all_stats.iter().map(|s| s.cpu_usage_percent).sum::<f64>() / total_containers as f64;
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

        Ok(eyes_devine_shared::TotalStats {
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

    /// Get latest container info for all containers
    pub async fn get_all_containers(&self) -> Result<Vec<ContainerInfo>> {
        // Get the most recent info for each container
        // This is a simplified version - in production, you might want to use a window function
        let containers = container_info::Entity::find()
            .order_by_desc(container_info::Column::CollectedAt)
            .all(&self.db)
            .await?;

        // Deduplicate by container_id, keeping the most recent
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        for container in containers {
            if seen.insert(container.container_id.clone()) {
                result.push(ContainerInfo {
                    id: container.container_id,
                    name: container.container_name,
                    image: container.image,
                    status: container.status,
                    created: container.created.map(|dt| dt.with_timezone(&Utc)),
                });
            }
        }

        Ok(result)
    }

    /// Get all images
    pub async fn get_all_images(&self) -> Result<Vec<ImageInfo>> {
        let images = docker_images::Entity::find()
            .all(&self.db)
            .await?;

        Ok(images.iter().map(|img| Self::entity_to_image_info(img)).collect())
    }

    /// Get image by ID
    pub async fn get_image(&self, image_id: &str) -> Result<Option<ImageInfo>> {
        let image = docker_images::Entity::find()
            .filter(docker_images::Column::ImageId.eq(image_id))
            .one(&self.db)
            .await?;

        Ok(image.map(|img| Self::entity_to_image_info(&img)))
    }

    /// Get image version history
    pub async fn get_image_history(
        &self,
        image_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<ImageInfo>> {
        let fixed_offset = FixedOffset::east_opt(0).unwrap();
        
        let mut query = image_versions::Entity::find()
            .filter(image_versions::Column::ImageId.eq(image_id));

        if let Some(from_dt) = from {
            let from_tz = from_dt.with_timezone(&fixed_offset);
            query = query.filter(image_versions::Column::Timestamp.gte(from_tz));
        }

        if let Some(to_dt) = to {
            let to_tz = to_dt.with_timezone(&fixed_offset);
            query = query.filter(image_versions::Column::Timestamp.lte(to_tz));
        }

        query = query.order_by_desc(image_versions::Column::Timestamp);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let versions = query.all(&self.db).await?;

        Ok(versions.iter().map(|v| Self::entity_version_to_image_info(v)).collect())
    }

    // Helper: Convert entity to ContainerStats
    fn entity_to_container_stats(entity: &container_stats::Model) -> ContainerStats {
        ContainerStats {
            container_id: entity.container_id.clone(),
            container_name: entity.container_name.clone(),
            cpu_usage_percent: entity.cpu_usage_percent,
            memory_usage_bytes: entity.memory_usage_bytes as u64,
            memory_limit_bytes: entity.memory_limit_bytes as u64,
            memory_usage_percent: entity.memory_usage_percent,
            network_rx_bytes: entity.network_rx_bytes as u64,
            network_tx_bytes: entity.network_tx_bytes as u64,
            block_read_bytes: entity.block_read_bytes as u64,
            block_write_bytes: entity.block_write_bytes as u64,
            timestamp: entity.timestamp.with_timezone(&Utc),
        }
    }

    // Helper: Convert entity to ImageInfo
    fn entity_to_image_info(entity: &docker_images::Model) -> ImageInfo {
        use serde_json::Value;
        
        let repo_tags: Vec<String> = match &entity.repo_tags {
            Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        };

        ImageInfo {
            id: entity.image_id.clone(),
            repo_tags,
            size: entity.size_bytes as u64,
            created: entity.created_at.map(|dt| dt.with_timezone(&Utc)),
            architecture: entity.architecture.clone(),
            os: entity.os.clone(),
        }
    }

    // Helper: Convert version entity to ImageInfo
    fn entity_version_to_image_info(entity: &image_versions::Model) -> ImageInfo {
        use serde_json::Value;
        
        let repo_tags: Vec<String> = match &entity.repo_tags {
            Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        };

        ImageInfo {
            id: entity.image_id.clone(),
            repo_tags,
            size: entity.size_bytes as u64,
            created: Some(entity.timestamp.with_timezone(&Utc)),
            architecture: None,
            os: None,
        }
    }

    /// Get HTTP requests for a specific container
    pub async fn get_container_http_requests(
        &self,
        container_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<HttpRequest>> {
        let fixed_offset = FixedOffset::east_opt(0).unwrap();
        
        let mut query = http_requests::Entity::find()
            .filter(http_requests::Column::ContainerId.eq(container_id));

        if let Some(from_dt) = from {
            let from_tz = from_dt.with_timezone(&fixed_offset);
            query = query.filter(http_requests::Column::Timestamp.gte(from_tz));
        }

        if let Some(to_dt) = to {
            let to_tz = to_dt.with_timezone(&fixed_offset);
            query = query.filter(http_requests::Column::Timestamp.lte(to_tz));
        }

        query = query.order_by_desc(http_requests::Column::Timestamp);

        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }

        let requests = query.all(&self.db).await?;

        Ok(requests.iter().map(|r| Self::entity_to_http_request(r)).collect())
    }

    // Helper: Convert entity to HttpRequest
    fn entity_to_http_request(entity: &http_requests::Model) -> HttpRequest {
        HttpRequest {
            container_id: entity.container_id.clone(),
            container_name: entity.container_name.clone(),
            endpoint: entity.endpoint.clone(),
            method: entity.method.clone(),
            http_status: entity.http_status as u16,
            response_time_ms: entity.response_time_ms,
            timestamp: entity.timestamp.with_timezone(&Utc),
        }
    }
}

