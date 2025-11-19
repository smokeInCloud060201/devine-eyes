mod config;
mod handlers;
mod routes;
mod query_validation;

use actix_web::{web, App, HttpServer};
use config::Config;
use handlers::AppState;
use eyes_devine_services::{CacheService, DockerService, QueryService, CachedQueryService, create_connection, NetworkMonitorService};
use std::sync::Arc;
use actix_cors::Cors;
use crate::query_validation::HistoryQueryValidator;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    
    log::info!("Starting Docker Monitor Server on {}:{}", config.server_host, config.server_port);

    // Initialize Docker service
    let docker_service = Arc::new(
        DockerService::new()
            .await
            .expect("Failed to initialize Docker service"),
    );

    // Initialize Redis cache if URL is provided
    let cache_service = Arc::new(
        CacheService::new(config.redis_url.clone())
            .unwrap_or_else(|e| {
                log::warn!("Failed to initialize Redis cache: {}. Continuing without cache.", e);
                CacheService::new(None).expect("Failed to create cache service")
            })
    );

    if cache_service.is_enabled() {
        log::info!("Redis cache enabled");
    }

    // Initialize database if URL is provided
    let (db, query_service) = if !config.database_url.is_empty() {
        match create_connection(&config.database_url).await {
            Ok(conn) => {
                log::info!("Database connection established");
                log::info!("Note: Run migrations with 'cd migrations && cargo run -- up' if not already done");
                let base_qs = Arc::new(QueryService::new(conn.clone()));
                let cached_qs = Arc::new(CachedQueryService::new(
                    base_qs,
                    cache_service.clone(),
                    config.cache_ttl_containers,
                    config.cache_ttl_stats,
                    config.cache_ttl_images,
                    config.cache_ttl_history,
                ));
                (Some(conn), Some(cached_qs))
            }
            Err(e) => {
                log::warn!("Failed to connect to database: {}. Continuing without database.", e);
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    // Create query validator
    let query_validator = HistoryQueryValidator::new(
        config.max_query_range_days,
        config.max_results_per_query,
    );

    // Initialize network monitor for HTTP request capture
    let network_monitor = Arc::new(NetworkMonitorService::new(docker_service.clone()));
    let network_monitor_for_start = Arc::clone(&network_monitor);
    
    // Start network monitoring in background (non-blocking)
    tokio::spawn(async move {
        if let Err(e) = network_monitor_for_start.start_monitoring().await {
            log::warn!("Failed to start network monitoring: {}. Will use log parsing as fallback.", e);
        }
    });

    let app_state = web::Data::new(AppState {
        docker_service,
        db,
        query_service,
        cache_service,
        query_validator,
        network_monitor: Some(network_monitor),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_origin()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .configure(routes::configure)
    })
    .bind(format!("{}:{}", config.server_host, config.server_port))?
    .run()
    .await
}

