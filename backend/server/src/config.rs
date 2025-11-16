use std::env;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub server_host: String,
    pub server_port: u16,
    
    // Cache TTLs
    pub cache_ttl_containers: Duration,
    pub cache_ttl_stats: Duration,
    pub cache_ttl_images: Duration,
    pub cache_ttl_history: Duration,
    
    // Query limits
    pub max_query_range_days: u32,
    pub max_results_per_query: usize,
}

impl Config {
    pub fn from_env() -> Self {
        // Cache TTLs (in seconds)
        let cache_ttl_containers_secs = env::var("CACHE_TTL_CONTAINERS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        
        let cache_ttl_stats_secs = env::var("CACHE_TTL_STATS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);
        
        let cache_ttl_images_secs = env::var("CACHE_TTL_IMAGES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300); // 5 minutes
        
        let cache_ttl_history_secs = env::var("CACHE_TTL_HISTORY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        // Query limits
        let max_query_range_days = env::var("MAX_QUERY_RANGE_DAYS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        
        let max_results_per_query = env::var("MAX_RESULTS_PER_QUERY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000);

        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/docker_monitor".to_string()),
            redis_url: env::var("REDIS_URL").ok(),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            cache_ttl_containers: Duration::from_secs(cache_ttl_containers_secs),
            cache_ttl_stats: Duration::from_secs(cache_ttl_stats_secs),
            cache_ttl_images: Duration::from_secs(cache_ttl_images_secs),
            cache_ttl_history: Duration::from_secs(cache_ttl_history_secs),
            max_query_range_days,
            max_results_per_query,
        }
    }
}

