use eyes_devine_shared::{ContainerLog, LogFilter, TotalStats};
use eyes_devine_services::{CacheService, DockerService};
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Duration;

pub struct AppState {
    pub docker_service: Arc<DockerService>,
    pub _db: Option<DatabaseConnection>,
    pub cache_service: Arc<CacheService>,
}

pub async fn get_total_stats(state: web::Data<AppState>) -> impl Responder {
    // Try to get from cache first
    if state.cache_service.is_enabled() {
        if let Ok(Some(cached_stats)) = state.cache_service.get::<TotalStats>("total_stats").await {
            return HttpResponse::Ok().json(cached_stats);
        }
    }

    match state.docker_service.get_total_stats().await {
        Ok(stats) => {
            // Cache the result for 5 seconds
            if state.cache_service.is_enabled() {
                let _ = state.cache_service.set("total_stats", &stats, Some(Duration::from_secs(5))).await;
            }
            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            log::error!("Failed to get total stats: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get total stats: {}", e)
            }))
        }
    }
}

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

