use std::env;
use std::time::Duration;

#[derive(Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    
    // Collection intervals
    pub stats_collection_interval: Duration,
    pub status_collection_interval: Duration,
    pub image_collection_interval: Duration,
    pub http_requests_collection_interval: Duration,
    
    // Batch settings
    pub batch_size: usize,
    pub batch_timeout: Duration,
}

impl Config {
    pub fn from_env() -> Self {
        let server_host = env::var("WORKER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        
        let server_port = env::var("WORKER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8081);
        
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL environment variable must be set");

        // Collection intervals (in seconds)
        let stats_interval_secs = env::var("STATS_COLLECTION_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);
        
        let status_interval_secs = env::var("STATUS_COLLECTION_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        
        let image_interval_secs = env::var("IMAGE_COLLECTION_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);
        
        let http_requests_interval_secs = env::var("HTTP_REQUESTS_COLLECTION_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        
        // Batch settings
        let batch_size = env::var("BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(500);
        
        let batch_timeout_secs = env::var("BATCH_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        Self {
            server_host,
            server_port,
            database_url,
            stats_collection_interval: Duration::from_secs(stats_interval_secs),
            status_collection_interval: Duration::from_secs(status_interval_secs),
            image_collection_interval: Duration::from_secs(image_interval_secs),
            http_requests_collection_interval: Duration::from_secs(http_requests_interval_secs),
            batch_size,
            batch_timeout: Duration::from_secs(batch_timeout_secs),
        }
    }
}

