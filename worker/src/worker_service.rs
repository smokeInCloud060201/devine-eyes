use eyes_devine_services::{DockerService, NetworkMonitorService};
use eyes_devine_shared::{ContainerInfo, ContainerStats, ImageInfo, HttpRequest};
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use chrono::{Utc, FixedOffset};
use crate::config::Config;
use crate::entity::container_info;
use eyes_devine_services::{container_stats, docker_images, image_versions, http_requests};

pub struct WorkerService {
    docker_service: Arc<DockerService>,
    db: DatabaseConnection,
    config: Config,
}

// Batch buffers for collecting data before inserting
struct BatchBuffers {
    stats: Arc<Mutex<Vec<ContainerStats>>>,
    container_info: Arc<Mutex<Vec<ContainerInfo>>>,
    images: Arc<Mutex<Vec<ImageInfo>>>,
    http_requests: Arc<Mutex<Vec<HttpRequest>>>,
}

impl WorkerService {
    pub fn new(
        docker_service: Arc<DockerService>,
        db: DatabaseConnection,
        config: Config,
    ) -> Self {
        Self {
            docker_service,
            db,
            config,
        }
    }

    pub async fn start(&self) {
        log::info!("Worker service starting with configuration:");
        log::info!("  - Stats collection interval: {:?}", self.config.stats_collection_interval);
        log::info!("  - Status collection interval: {:?}", self.config.status_collection_interval);
        log::info!("  - Image collection interval: {:?}", self.config.image_collection_interval);
        log::info!("  - HTTP requests collection interval: {:?}", self.config.http_requests_collection_interval);
        log::info!("  - Batch size: {}", self.config.batch_size);
        log::info!("  - Batch timeout: {:?}", self.config.batch_timeout);

        // Initialize network monitor for HTTP request capture with database connection for direct insertion
        let db_for_monitor = Arc::new(self.db.clone());
        let network_monitor = Arc::new(NetworkMonitorService::with_database(
            self.docker_service.clone(),
            db_for_monitor,
        ));
        let network_monitor_for_start = Arc::clone(&network_monitor);
        
        // Start network monitoring in background (non-blocking)
        tokio::spawn(async move {
            match network_monitor_for_start.start_monitoring().await {
                Ok(()) => {
                    log::info!("Network monitoring started successfully");
                }
                Err(e) => {
                    log::error!("Failed to start network monitoring: {}", e);
                    log::error!("HTTP request capture from network packets will not be available.");
                    log::error!("To enable network capture:");
                    #[cfg(target_os = "windows")]
                    {
                        log::error!("  1. Install Npcap from https://nmap.org/npcap/");
                        log::error!("  2. Build with: cargo build --features network-capture");
                        log::error!("  3. Run as Administrator");
                    }
                    #[cfg(target_os = "linux")]
                    {
                        log::error!("  1. Install libpcap-dev: sudo apt-get install libpcap-dev");
                        log::error!("  2. Build with: cargo build --features network-capture");
                        log::error!("  3. Run with root or CAP_NET_RAW capability");
                    }
                    #[cfg(target_os = "macos")]
                    {
                        log::error!("  1. Install Xcode Command Line Tools: xcode-select --install");
                        log::error!("  2. Build with: cargo build --features network-capture");
                        log::error!("  3. Run with root or admin privileges");
                    }
                }
            }
        });

        // Create shared batch buffers
        let buffers = BatchBuffers {
            stats: Arc::new(Mutex::new(Vec::new())),
            container_info: Arc::new(Mutex::new(Vec::new())),
            images: Arc::new(Mutex::new(Vec::new())),
            http_requests: Arc::new(Mutex::new(Vec::new())),
        };

        // Spawn separate tasks for different collection types
        let stats_task = self.start_stats_collection(buffers.stats.clone());
        let status_task = self.start_status_collection(buffers.container_info.clone());
        let image_task = self.start_image_collection(buffers.images.clone());
        let http_requests_task = self.start_http_requests_collection(buffers.http_requests.clone(), network_monitor.clone());

        // Spawn batch insertion tasks
        let stats_insert_task = self.start_batch_insert_stats(buffers.stats.clone());
        let status_insert_task = self.start_batch_insert_container_info(buffers.container_info.clone());
        let image_insert_task = self.start_batch_insert_images(buffers.images.clone());
        let http_requests_insert_task = self.start_batch_insert_http_requests(buffers.http_requests.clone());

        // Wait for all tasks (they run forever)
        tokio::select! {
            _ = stats_task => log::error!("Stats collection task exited"),
            _ = status_task => log::error!("Status collection task exited"),
            _ = image_task => log::error!("Image collection task exited"),
            _ = http_requests_task => log::error!("HTTP requests collection task exited"),
            _ = stats_insert_task => log::error!("Stats batch insert task exited"),
            _ = status_insert_task => log::error!("Status batch insert task exited"),
            _ = image_insert_task => log::error!("Image batch insert task exited"),
            _ = http_requests_insert_task => log::error!("HTTP requests batch insert task exited"),
        }
    }

    // Stats collection task - collects container stats periodically
    async fn start_stats_collection(&self, buffer: Arc<Mutex<Vec<ContainerStats>>>) {
        let docker_service = self.docker_service.clone();
        let interval = self.config.stats_collection_interval;

        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            match Self::collect_container_stats(&docker_service).await {
                Ok(stats) => {
                    let stats_len = stats.len();
                    let mut buf = buffer.lock().await;
                    buf.extend(stats);
                    log::debug!("Collected {} stats, buffer size: {}", stats_len, buf.len());
                }
                Err(e) => {
                    log::warn!("Failed to collect container stats: {}", e);
                }
            }
        }
    }

    // Status collection task - collects container status periodically
    async fn start_status_collection(&self, buffer: Arc<Mutex<Vec<ContainerInfo>>>) {
        let docker_service = self.docker_service.clone();
        let interval = self.config.status_collection_interval;

        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            match docker_service.list_containers().await {
                Ok(containers) => {
                    let mut buf = buffer.lock().await;
                    buf.extend(containers);
                    log::debug!("Collected container status, buffer size: {}", buf.len());
                }
                Err(e) => {
                    log::warn!("Failed to collect container status: {}", e);
                }
            }
        }
    }

    // Image collection task - collects images periodically
    async fn start_image_collection(&self, buffer: Arc<Mutex<Vec<ImageInfo>>>) {
        let docker_service = self.docker_service.clone();
        let interval = self.config.image_collection_interval;

        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            match Self::collect_images(&docker_service).await {
                Ok(images) => {
                    let images_len = images.len();
                    let mut buf = buffer.lock().await;
                    buf.extend(images);
                    log::debug!("Collected {} images, buffer size: {}", images_len, buf.len());
                }
                Err(e) => {
                    log::warn!("Failed to collect images: {}", e);
                }
            }
        }
    }

    // Batch insert task for stats
    async fn start_batch_insert_stats(&self, buffer: Arc<Mutex<Vec<ContainerStats>>>) {
        let db = self.db.clone();
        let batch_size = self.config.batch_size;
        let batch_timeout = self.config.batch_timeout;

        loop {
            tokio::time::sleep(batch_timeout).await;

            let mut buf = buffer.lock().await;
            if buf.is_empty() {
                continue;
            }

            // Take up to batch_size items
            let to_insert: Vec<ContainerStats> = if buf.len() > batch_size {
                buf.drain(..batch_size).collect()
            } else {
                buf.drain(..).collect()
            };

            drop(buf); // Release lock before DB operation

            if !to_insert.is_empty() {
                if let Err(e) = Self::batch_insert_stats(&db, &to_insert).await {
                    log::error!("Failed to batch insert stats: {}", e);
                    // Optionally: re-add to buffer or queue for retry
                } else {
                    log::info!("Successfully inserted {} stats", to_insert.len());
                }
            }
        }
    }

    // Batch insert task for container info
    async fn start_batch_insert_container_info(&self, buffer: Arc<Mutex<Vec<ContainerInfo>>>) {
        let db = self.db.clone();
        let batch_size = self.config.batch_size;
        let batch_timeout = self.config.batch_timeout;

        loop {
            tokio::time::sleep(batch_timeout).await;

            let mut buf = buffer.lock().await;
            if buf.is_empty() {
                continue;
            }

            let to_insert: Vec<ContainerInfo> = if buf.len() > batch_size {
                buf.drain(..batch_size).collect()
            } else {
                buf.drain(..).collect()
            };

            drop(buf);

            if !to_insert.is_empty() {
                if let Err(e) = Self::batch_insert_container_info(&db, &to_insert).await {
                    log::error!("Failed to batch insert container info: {}", e);
                } else {
                    log::info!("Successfully inserted {} container info records", to_insert.len());
                }
            }
        }
    }

    // Batch insert task for images
    async fn start_batch_insert_images(&self, buffer: Arc<Mutex<Vec<ImageInfo>>>) {
        let db = self.db.clone();
        let batch_size = self.config.batch_size;
        let batch_timeout = self.config.batch_timeout;

        loop {
            tokio::time::sleep(batch_timeout).await;

            let mut buf = buffer.lock().await;
            if buf.is_empty() {
                continue;
            }

            let to_insert: Vec<ImageInfo> = if buf.len() > batch_size {
                buf.drain(..batch_size).collect()
            } else {
                buf.drain(..).collect()
            };

            drop(buf);

            if !to_insert.is_empty() {
                if let Err(e) = Self::batch_insert_images(&db, &to_insert).await {
                    log::error!("Failed to batch insert images: {}", e);
                } else {
                    log::info!("Successfully inserted {} image records", to_insert.len());
                }
            }
        }
    }

    // Helper: Collect container stats for all running containers
    async fn collect_container_stats(
        docker_service: &DockerService,
    ) -> anyhow::Result<Vec<ContainerStats>> {
        let containers = docker_service.list_containers().await?;
        let mut stats = Vec::new();

        for container in containers {
            // Only collect stats for running containers
            let is_running = container.status.to_lowercase().contains("up")
                || container.status.to_lowercase().contains("running");

            if is_running {
                match docker_service.get_container_stats(&container.id).await {
                    Ok(stat) => stats.push(stat),
                    Err(e) => {
                        log::debug!("Failed to get stats for container {}: {}", container.id, e);
                    }
                }
            }
        }

        Ok(stats)
    }

    // Helper: Collect all images
    async fn collect_images(docker_service: &DockerService) -> anyhow::Result<Vec<ImageInfo>> {
        // List containers and get unique images
        let containers = docker_service.list_containers().await?;
        let mut image_ids = std::collections::HashSet::new();
        let mut images = Vec::new();

        for container in containers {
            if !image_ids.contains(&container.image) {
                image_ids.insert(container.image.clone());
                if let Ok(Some(image_info)) = docker_service.get_image_info(&container.image).await {
                    images.push(image_info);
                }
            }
        }

        Ok(images)
    }

    // Helper: Batch insert stats
    async fn batch_insert_stats(
        db: &DatabaseConnection,
        stats: &[ContainerStats],
    ) -> anyhow::Result<()> {
        use sea_orm::ActiveValue::Set;

        let fixed_offset = FixedOffset::east_opt(0).unwrap();

        let active_models: Vec<container_stats::ActiveModel> = stats
            .iter()
            .map(|stat| {
                let timestamp = stat.timestamp.with_timezone(&fixed_offset);
                container_stats::ActiveModel {
                    container_id: Set(stat.container_id.clone()),
                    container_name: Set(stat.container_name.clone()),
                    cpu_usage_percent: Set(stat.cpu_usage_percent),
                    memory_usage_bytes: Set(stat.memory_usage_bytes as i64),
                    memory_limit_bytes: Set(stat.memory_limit_bytes as i64),
                    memory_usage_percent: Set(stat.memory_usage_percent),
                    network_rx_bytes: Set(stat.network_rx_bytes as i64),
                    network_tx_bytes: Set(stat.network_tx_bytes as i64),
                    block_read_bytes: Set(stat.block_read_bytes as i64),
                    block_write_bytes: Set(stat.block_write_bytes as i64),
                    timestamp: Set(timestamp),
                    ..Default::default()
                }
            })
            .collect();

        container_stats::Entity::insert_many(active_models)
            .exec(db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to batch insert stats: {}", e))?;

        Ok(())
    }

    // Helper: Batch insert container info
    async fn batch_insert_container_info(
        db: &DatabaseConnection,
        containers: &[ContainerInfo],
    ) -> anyhow::Result<()> {
        use sea_orm::ActiveValue::Set;

        let fixed_offset = FixedOffset::east_opt(0).unwrap();
        let collected_at = Utc::now().with_timezone(&fixed_offset);

        let active_models: Vec<container_info::ActiveModel> = containers
            .iter()
            .map(|container| {
                let created = container.created.map(|dt| dt.with_timezone(&fixed_offset));
                container_info::ActiveModel {
                    container_id: Set(container.id.clone()),
                    container_name: Set(container.name.clone()),
                    image: Set(container.image.clone()),
                    status: Set(container.status.clone()),
                    created: Set(created),
                    collected_at: Set(collected_at),
                    ..Default::default()
                }
            })
            .collect();

        container_info::Entity::insert_many(active_models)
            .exec(db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to batch insert container info: {}", e))?;

        Ok(())
    }

    // HTTP requests collection task - collects HTTP requests periodically
    async fn start_http_requests_collection(
        &self,
        buffer: Arc<Mutex<Vec<HttpRequest>>>,
        network_monitor: Arc<NetworkMonitorService>,
    ) {
        let docker_service = self.docker_service.clone();
        let interval = self.config.http_requests_collection_interval;

        log::info!("Resister the Http Request collections");

        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            match Self::collect_http_requests(&docker_service, &network_monitor).await {
                Ok(requests) => {
                    let requests_len = requests.len();
                    if requests_len > 0 {
                        let mut buf = buffer.lock().await;
                        buf.extend(requests);
                        log::info!("Collected {} HTTP requests, buffer size: {}", requests_len, buf.len());
                    } else {
                        log::debug!("No HTTP requests collected (network capture may not be working)");
                    }
                }
                Err(e) => {
                    log::warn!("Failed to collect HTTP requests: {}", e);
                }
            }
        }
    }

    // Batch insert task for HTTP requests
    async fn start_batch_insert_http_requests(&self, buffer: Arc<Mutex<Vec<HttpRequest>>>) {
        let db = self.db.clone();
        let batch_size = self.config.batch_size;
        let batch_timeout = self.config.batch_timeout;

        loop {
            tokio::time::sleep(batch_timeout).await;

            let mut buf = buffer.lock().await;
            if buf.is_empty() {
                continue;
            }

            let to_insert: Vec<HttpRequest> = if buf.len() > batch_size {
                buf.drain(..batch_size).collect()
            } else {
                buf.drain(..).collect()
            };

            drop(buf);

            if !to_insert.is_empty() {
                // Log summary of requests being inserted
                let mut container_counts: HashMap<String, usize> = HashMap::new();
                let mut method_counts: HashMap<String, usize> = HashMap::new();
                for req in &to_insert {
                    *container_counts.entry(req.container_name.clone()).or_insert(0) += 1;
                    *method_counts.entry(req.method.clone()).or_insert(0) += 1;
                }
                
                log::info!("💾 Attempting to insert {} HTTP request records into database", to_insert.len());
                log::info!("   📊 Summary: {} unique containers, {} unique methods", 
                    container_counts.len(), method_counts.len());
                log::debug!("   📋 Container breakdown: {:?}", container_counts);
                log::debug!("   📋 Method breakdown: {:?}", method_counts);
                
                // Log first few requests for traceability
                let sample_size = std::cmp::min(5, to_insert.len());
                for (idx, req) in to_insert.iter().take(sample_size).enumerate() {
                    log::info!("   [{}/{}] {} {} {} ({}ms) - container: {} ({})", 
                        idx + 1, sample_size,
                        req.method, req.endpoint, req.http_status, 
                        req.response_time_ms as i64,
                        &req.container_id[..12], req.container_name);
                }
                if to_insert.len() > sample_size {
                    log::info!("   ... and {} more requests", to_insert.len() - sample_size);
                }
                
                if let Err(e) = Self::batch_insert_http_requests(&db, &to_insert).await {
                    log::error!("❌ Failed to batch insert HTTP requests: {}", e);
                } else {
                    log::info!("✅ Successfully inserted {} HTTP request records into database", to_insert.len());
                }
            }
        }
    }

    // Helper: Collect HTTP requests for all containers
    async fn collect_http_requests(
        docker_service: &DockerService,
        network_monitor: &NetworkMonitorService,
    ) -> anyhow::Result<Vec<HttpRequest>> {
        let containers = docker_service.list_containers().await?;
        let mut all_requests = Vec::new();

        log::debug!("Collecting HTTP requests for {} containers", containers.len());

        for container in containers {
            // Only collect for running containers
            let is_running = container.status.to_lowercase().contains("up")
                || container.status.to_lowercase().contains("running");

            if !is_running {
                continue;
            }

            log::debug!("Checking container {} ({}) for HTTP requests", container.id, container.name);

            // Try to get requests from network monitor
            match network_monitor.get_container_requests(&container.id).await {
                Ok(requests) => {
                    if !requests.is_empty() {
                        log::info!("Found {} requests from network monitor for container {} ({})", 
                            requests.len(), container.id, container.name);
                        // Log sample of collected requests
                        let sample_size = std::cmp::min(3, requests.len());
                        for (idx, req) in requests.iter().take(sample_size).enumerate() {
                            log::debug!("   Collected [{}]: {} {} {} ({}ms)", 
                                idx + 1, req.method, req.endpoint, req.http_status, req.response_time_ms as i64);
                        }
                        if requests.len() > sample_size {
                            log::debug!("   ... and {} more requests from this container", requests.len() - sample_size);
                        }
                        all_requests.extend(requests);
                        // Clear requests after collection to avoid duplicates
                        network_monitor.clear_container_requests(&container.id).await;
                        continue;
                    } else {
                        log::debug!("No requests found in network monitor for container {} ({})", 
                            container.id, container.name);
                    }
                }
                Err(e) => {
                    log::warn!("Network monitor error for container {} ({}): {}", 
                        container.id, container.name, e);
                }
            }

            // Note: Network capture is the primary method
            // If network capture isn't working, check:
            // 1. Network capture feature is enabled: cargo build --features network-capture
            // 2. Npcap is installed (Windows) or libpcap-dev (Linux)
            // 3. Proper permissions (NET_RAW capability or root/admin)
            // 4. Network interface is being monitored (check worker logs for "Starting network-level HTTP request monitoring")
        }

        log::info!("Collected total {} HTTP requests from all containers", all_requests.len());
        Ok(all_requests)
    }


    // Helper: Batch insert images
    async fn batch_insert_images(
        db: &DatabaseConnection,
        images: &[ImageInfo],
    ) -> anyhow::Result<()> {
        use sea_orm::ActiveValue::Set;
        use serde_json::json;

        let fixed_offset = FixedOffset::east_opt(0).unwrap();
        let now = Utc::now().with_timezone(&fixed_offset);

        for image in images {
            // First, try to update existing image
            let existing = docker_images::Entity::find()
                .filter(docker_images::Column::ImageId.eq(&image.id))
                .one(db)
                .await?;

            if let Some(existing_model) = existing {
                // Update existing image
                let mut active_model: docker_images::ActiveModel = existing_model.into();
                active_model.repo_tags = Set(json!(image.repo_tags));
                active_model.size_bytes = Set(image.size as i64);
                active_model.architecture = Set(image.architecture.clone());
                active_model.os = Set(image.os.clone());
                active_model.created_at = Set(image.created.map(|dt| dt.with_timezone(&fixed_offset)));
                active_model.last_seen = Set(now);
                active_model.update(db).await?;

                // Also insert into image_versions for history
                let version_model = image_versions::ActiveModel {
                    image_id: Set(image.id.clone()),
                    repo_tags: Set(json!(image.repo_tags)),
                    size_bytes: Set(image.size as i64),
                    timestamp: Set(now),
                    ..Default::default()
                };
                image_versions::Entity::insert(version_model).exec(db).await?;
            } else {
                // Insert new image
                let image_model = docker_images::ActiveModel {
                    image_id: Set(image.id.clone()),
                    repo_tags: Set(json!(image.repo_tags)),
                    size_bytes: Set(image.size as i64),
                    architecture: Set(image.architecture.clone()),
                    os: Set(image.os.clone()),
                    created_at: Set(image.created.map(|dt| dt.with_timezone(&fixed_offset))),
                    first_seen: Set(now),
                    last_seen: Set(now),
                    ..Default::default()
                };
                docker_images::Entity::insert(image_model).exec(db).await?;

                // Insert into image_versions
                let version_model = image_versions::ActiveModel {
                    image_id: Set(image.id.clone()),
                    repo_tags: Set(json!(image.repo_tags)),
                    size_bytes: Set(image.size as i64),
                    timestamp: Set(now),
                    ..Default::default()
                };
                image_versions::Entity::insert(version_model).exec(db).await?;
            }
        }

        Ok(())
    }

    // Helper: Batch insert HTTP requests
    async fn batch_insert_http_requests(
        db: &DatabaseConnection,
        requests: &[HttpRequest],
    ) -> anyhow::Result<()> {
        use sea_orm::ActiveValue::Set;

        let fixed_offset = FixedOffset::east_opt(0).unwrap();

        log::debug!("🔍 Preparing {} HTTP requests for database insertion", requests.len());

        let active_models: Vec<http_requests::ActiveModel> = requests
            .iter()
            .map(|req| {
                let timestamp = req.timestamp.with_timezone(&fixed_offset);
                http_requests::ActiveModel {
                    container_id: Set(req.container_id.clone()),
                    container_name: Set(req.container_name.clone()),
                    endpoint: Set(req.endpoint.clone()),
                    method: Set(req.method.clone()),
                    http_status: Set(req.http_status as i16),
                    response_time_ms: Set(req.response_time_ms),
                    timestamp: Set(timestamp),
                    ..Default::default()
                }
            })
            .collect();

        log::debug!("📝 Executing database insert for {} HTTP request records", active_models.len());
        
        http_requests::Entity::insert_many(active_models)
            .exec(db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to batch insert HTTP requests: {}", e))?;

        log::debug!("✓ Database insert completed successfully for {} HTTP requests", requests.len());
        Ok(())
    }
}
