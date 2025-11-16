mod config;
mod worker_service;
mod entity;

use actix_web::{web, App, HttpServer};
use config::Config;
use eyes_devine_services::{DockerService, create_connection};
use std::sync::Arc;
use worker_service::WorkerService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env();
    
    log::info!("Starting Docker Monitor Worker");

    // Initialize Docker service
    let docker_service = Arc::new(
        DockerService::new()
            .await
            .expect("Failed to initialize Docker service"),
    );

    // Initialize database connection
    let db = create_connection(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    log::info!("Database connection established");

    // Create and start worker service
    let worker_service = WorkerService::new(docker_service, db, config.clone());
    
    // Start the worker in a background task
    tokio::spawn(async move {
        worker_service.start().await;
    });

    // Start a minimal HTTP server for health checks
    HttpServer::new(move || {
        App::new()
            .route("/health", web::get().to(|| async { "OK" }))
    })
    .bind(format!("{}:{}", config.server_host, config.server_port))?
    .run()
    .await
}

