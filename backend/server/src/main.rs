mod config;
mod handlers;
mod routes;

use actix_web::{web, App, HttpServer};
use config::Config;
use handlers::AppState;
use eyes_devine_services::{CacheService, DockerService, create_connection};
use std::sync::Arc;
use actix_cors::Cors;

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
    let db = if !config.database_url.is_empty() {
        match create_connection(&config.database_url).await {
            Ok(conn) => {
                log::info!("Database connection established");
                log::info!("Note: Run migrations with 'cd migrations && cargo run -- up' if not already done");
                Some(conn)
            }
            Err(e) => {
                log::warn!("Failed to connect to database: {}. Continuing without database.", e);
                None
            }
        }
    } else {
        None
    };

    let app_state = web::Data::new(AppState {
        docker_service,
        _db: db,
        cache_service,
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

