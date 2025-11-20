use crate::{QueryService, CacheService};
use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use chrono::{DateTime, Utc};
use eyes_devine_shared::{ContainerStats, ContainerInfo, ImageInfo, HttpRequest};

/// Wrapper around QueryService that adds Redis caching
pub struct CachedQueryService {
    query_service: Arc<QueryService>,
    cache_service: Arc<CacheService>,
    cache_ttl_containers: Duration,
    cache_ttl_stats: Duration,
    cache_ttl_images: Duration,
    cache_ttl_history: Duration,
}

impl CachedQueryService {
    pub fn new(
        query_service: Arc<QueryService>,
        cache_service: Arc<CacheService>,
        cache_ttl_containers: Duration,
        cache_ttl_stats: Duration,
        cache_ttl_images: Duration,
        cache_ttl_history: Duration,
    ) -> Self {
        Self {
            query_service,
            cache_service,
            cache_ttl_containers,
            cache_ttl_stats,
            cache_ttl_images,
            cache_ttl_history,
        }
    }

    /// Get latest stats for a specific container (cached)
    pub async fn get_latest_container_stats(
        &self,
        container_id: &str,
    ) -> Result<Option<ContainerStats>> {
        let cache_key = format!("stats:container:{}:latest", container_id);

        // Try cache first
        if let Some(cached) = self.cache_service.get::<ContainerStats>(&cache_key).await? {
            return Ok(Some(cached));
        }

        // Cache miss - query database
        let result = self.query_service.get_latest_container_stats(container_id).await?;

        // Store in cache if found
        if let Some(ref stats) = result {
            let _ = self.cache_service.set(&cache_key, stats, Some(self.cache_ttl_stats)).await;
        }

        Ok(result)
    }

    /// Get latest stats for all containers (cached)
    pub async fn get_latest_all_container_stats(&self) -> Result<Vec<ContainerStats>> {
        let cache_key = "stats:containers:all:latest";

        // Try cache first
        if let Some(cached) = self.cache_service.get::<Vec<ContainerStats>>(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - query database
        let result = self.query_service.get_latest_all_container_stats().await?;

        // Store in cache
        let _ = self.cache_service.set(&cache_key, &result, Some(self.cache_ttl_stats)).await;

        Ok(result)
    }

    /// Get total stats (cached)
    pub async fn get_total_stats(&self) -> Result<eyes_devine_shared::TotalStats> {
        let cache_key = "stats:total:latest";

        // Try cache first
        if let Some(cached) = self.cache_service.get::<eyes_devine_shared::TotalStats>(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - query database
        let result = self.query_service.get_total_stats().await?;

        // Store in cache
        let _ = self.cache_service.set(&cache_key, &result, Some(self.cache_ttl_stats)).await;

        Ok(result)
    }

    /// Get all containers (cached)
    pub async fn get_all_containers(&self) -> Result<Vec<ContainerInfo>> {
        let cache_key = "containers:list";

        // Try cache first
        if let Some(cached) = self.cache_service.get::<Vec<ContainerInfo>>(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - query database
        let result = self.query_service.get_all_containers().await?;

        // Store in cache
        let _ = self.cache_service.set(&cache_key, &result, Some(self.cache_ttl_containers)).await;

        Ok(result)
    }

    /// Get all images (cached)
    pub async fn get_all_images(&self) -> Result<Vec<ImageInfo>> {
        let cache_key = "images:list";

        // Try cache first
        if let Some(cached) = self.cache_service.get::<Vec<ImageInfo>>(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - query database
        let result = self.query_service.get_all_images().await?;

        // Store in cache
        let _ = self.cache_service.set(&cache_key, &result, Some(self.cache_ttl_images)).await;

        Ok(result)
    }

    /// Get image by ID (cached)
    pub async fn get_image(&self, image_id: &str) -> Result<Option<ImageInfo>> {
        let cache_key = format!("image:{}", image_id);

        // Try cache first
        if let Some(cached) = self.cache_service.get::<ImageInfo>(&cache_key).await? {
            return Ok(Some(cached));
        }

        // Cache miss - query database
        let result = self.query_service.get_image(image_id).await?;

        // Store in cache if found
        if let Some(ref image) = result {
            let _ = self.cache_service.set(&cache_key, image, Some(self.cache_ttl_images)).await;
        }

        Ok(result)
    }

    /// Get historical stats (cached with query-specific key)
    pub async fn get_container_stats_history(
        &self,
        container_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<ContainerStats>> {
        // Create cache key from query parameters
        let cache_key = format!(
            "stats:history:{}:{}:{}:{}",
            container_id,
            from.map(|d| d.timestamp().to_string()).unwrap_or_else(|| "none".to_string()),
            to.map(|d| d.timestamp().to_string()).unwrap_or_else(|| "none".to_string()),
            limit.unwrap_or(0)
        );

        // Try cache first
        if let Some(cached) = self.cache_service.get::<Vec<ContainerStats>>(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - query database
        let result = self.query_service.get_container_stats_history(container_id, from, to, limit).await?;

        // Store in cache (shorter TTL for historical queries)
        let _ = self.cache_service.set(&cache_key, &result, Some(self.cache_ttl_history)).await;

        Ok(result)
    }

    /// Get image history (cached)
    pub async fn get_image_history(
        &self,
        image_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<ImageInfo>> {
        // Create cache key from query parameters
        let cache_key = format!(
            "image:history:{}:{}:{}:{}",
            image_id,
            from.map(|d| d.timestamp().to_string()).unwrap_or_else(|| "none".to_string()),
            to.map(|d| d.timestamp().to_string()).unwrap_or_else(|| "none".to_string()),
            limit.unwrap_or(0)
        );

        // Try cache first
        if let Some(cached) = self.cache_service.get::<Vec<ImageInfo>>(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - query database
        let result = self.query_service.get_image_history(image_id, from, to, limit).await?;

        // Store in cache
        let _ = self.cache_service.set(&cache_key, &result, Some(self.cache_ttl_history)).await;

        Ok(result)
    }

    /// Invalidate cache for a container (call when container data changes)
    /// Note: This invalidates specific keys. Wildcard deletion would require Redis SCAN.
    pub async fn invalidate_container_cache(&self, container_id: &str) -> Result<()> {
        let keys = vec![
            format!("stats:container:{}:latest", container_id),
            "stats:containers:all:latest".to_string(),
            "stats:total:latest".to_string(),
            "containers:list".to_string(),
        ];

        for key in keys {
            let _ = self.cache_service.delete(&key).await;
        }

        Ok(())
    }

    /// Invalidate cache for an image
    pub async fn invalidate_image_cache(&self, image_id: &str) -> Result<()> {
        let keys = vec![
            format!("image:{}", image_id),
            "images:list".to_string(),
        ];

        for key in keys {
            let _ = self.cache_service.delete(&key).await;
        }

        Ok(())
    }

    /// Get HTTP requests for a container (cached)
    pub async fn get_container_http_requests(
        &self,
        container_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<HttpRequest>> {
        // Create cache key from query parameters
        let cache_key = format!(
            "http_requests:{}:{}:{}:{}",
            container_id,
            from.map(|d| d.timestamp().to_string()).unwrap_or_else(|| "none".to_string()),
            to.map(|d| d.timestamp().to_string()).unwrap_or_else(|| "none".to_string()),
            limit.unwrap_or(0)
        );

        // Try cache first
        if let Some(cached) = self.cache_service.get::<Vec<HttpRequest>>(&cache_key).await? {
            return Ok(cached);
        }

        // Cache miss - query database
        let result = self.query_service.get_container_http_requests(container_id, from, to, limit).await?;

        // Store in cache (shorter TTL for request queries)
        let _ = self.cache_service.set(&cache_key, &result, Some(self.cache_ttl_history)).await;

        Ok(result)
    }
}

