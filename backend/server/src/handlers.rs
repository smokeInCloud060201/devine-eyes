use eyes_devine_shared::{ContainerLog, LogFilter};
use eyes_devine_services::{CacheService, DockerService};
use actix_web::{web, HttpResponse, Responder, Error};
use actix_web::web::Bytes;
use chrono::Utc;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use futures::stream::{self, StreamExt, once};
use std::time::Duration;
use tokio::time::interval;

pub struct AppState {
    pub docker_service: Arc<DockerService>,
    /// Database connection (optional, reserved for future use - not currently used for stats/containers)
    pub _db: Option<DatabaseConnection>,
    /// Cache service (optional, reserved for future use - stats/containers are fetched real-time)
    pub _cache_service: Arc<CacheService>,
}

/// Get total stats aggregated from all containers (real-time from Docker)
/// Returns comprehensive stats wrapped in a data field
pub async fn get_total_stats(state: web::Data<AppState>) -> impl Responder {
    match state.docker_service.get_comprehensive_stats().await {
        Ok(stats) => HttpResponse::Ok().json(serde_json::json!({
            "data": stats
        })),
        Err(e) => {
            log::error!("Failed to get comprehensive stats: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get comprehensive stats: {}", e)
            }))
        }
    }
}

/// SSE endpoint for comprehensive stats - streams data every 2 seconds
pub async fn get_total_stats_sse(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let docker_service = state.docker_service.clone();
    
    // Helper function to format stats as SSE data
    let format_stats = |stats: eyes_devine_shared::ComprehensiveStats| -> Result<Bytes, Error> {
        match serde_json::to_string(&stats) {
            Ok(json) => {
                let data = format!("data: {}\n\n", json);
                Ok(Bytes::from(data))
            }
            Err(e) => {
                log::error!("Failed to serialize stats: {}", e);
                let error_data = format!("data: {{\"error\":\"Failed to serialize stats: {}\"}}\n\n", e);
                Ok(Bytes::from(error_data))
            }
        }
    };
    
    // Send first message immediately
    let first_message = match docker_service.get_comprehensive_stats().await {
        Ok(stats) => format_stats(stats)?,
        Err(e) => {
            log::error!("Failed to get comprehensive stats: {}", e);
            Bytes::from(format!("data: {{\"error\":\"Failed to get stats: {}\"}}\n\n", e))
        }
    };
    
    // Create stream for subsequent messages (every 2 seconds)
    let interval_stream = stream::unfold((docker_service, interval(Duration::from_secs(2))), move |(docker_service, mut interval)| async move {
        interval.tick().await;
        
        let result = match docker_service.get_comprehensive_stats().await {
            Ok(stats) => format_stats(stats),
            Err(e) => {
                log::error!("Failed to get comprehensive stats: {}", e);
                Ok(Bytes::from(format!("data: {{\"error\":\"Failed to get stats: {}\"}}\n\n", e)))
            }
        };
        
        Some((result, (docker_service, interval)))
    });
    
    // Combine first message with interval stream
    let stream = once(async move { Ok::<Bytes, Error>(first_message) })
        .chain(interval_stream);

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .append_header(("X-Accel-Buffering", "no"))
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Access-Control-Allow-Headers", "Cache-Control"))
        .streaming(stream))
}

/// List all containers (real-time from Docker)
pub async fn get_all_containers(state: web::Data<AppState>) -> impl Responder {
    match state.docker_service.list_containers().await {
        Ok(containers) => HttpResponse::Ok().json(containers),
        Err(e) => {
            log::error!("Failed to list containers: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to list containers: {}", e)
            }))
        }
    }
}

/// Get stats for a specific container (real-time from Docker)
pub async fn get_container_stats(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let container_id = path.into_inner();
    match state.docker_service.get_container_stats(&container_id).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => {
            log::error!("Failed to get container stats: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get container stats: {}", e)
            }))
        }
    }
}

/// Get stats for all containers (real-time from Docker)
pub async fn get_all_container_stats(state: web::Data<AppState>) -> impl Responder {
    match state.docker_service.get_all_container_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => {
            log::error!("Failed to get all container stats: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get all container stats: {}", e)
            }))
        }
    }
}

/// Get logs for a specific container (real-time from Docker)
pub async fn get_container_logs(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<LogFilter>,
) -> impl Responder {
    let container_id = path.into_inner();
    
    let since = query.since.map(|dt| dt.timestamp());
    let until = query.until.map(|dt| dt.timestamp());
    let tail = query.limit;

    match state
        .docker_service
        .get_container_logs(&container_id, since, until, tail)
        .await
    {
        Ok(logs) => {
            let container_name = state
                .docker_service
                .list_containers()
                .await
                .ok()
                .and_then(|containers| {
                    containers
                        .iter()
                        .find(|c| c.id == container_id)
                        .map(|c| c.name.clone())
                })
                .unwrap_or_else(|| container_id.clone());

            let log_entries: Vec<ContainerLog> = logs
                .into_iter()
                .enumerate()
                .map(|(idx, line)| {
                    let stream = if line.contains("stderr") { "stderr" } else { "stdout" };
                    ContainerLog {
                        container_id: container_id.clone(),
                        container_name: container_name.clone(),
                        log_line: line,
                        timestamp: Utc::now() - chrono::Duration::seconds(idx as i64),
                        stream: stream.to_string(),
                    }
                })
                .collect();

            HttpResponse::Ok().json(log_entries)
        }
        Err(e) => {
            log::error!("Failed to get container logs: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get container logs: {}", e)
            }))
        }
    }
}

