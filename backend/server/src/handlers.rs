use eyes_devine_shared::{ContainerLog, LogFilter};
use eyes_devine_services::{CacheService, DockerService, CachedQueryService, ServiceMapService};
use actix_web::{web, HttpResponse, Responder, Error};
use actix_web::web::Bytes;
use chrono::{Utc, DateTime};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use futures::stream::{self, StreamExt, once};
use std::time::Duration;
use crate::query_validation::{HistoryQueryValidator, PaginationParams, PaginatedResponse};

pub struct AppState {
    pub docker_service: Arc<DockerService>, // Keep for logs (still need real-time)
    pub db: Option<DatabaseConnection>,
    pub query_service: Option<Arc<CachedQueryService>>,
    pub cache_service: Arc<CacheService>,
    pub query_validator: HistoryQueryValidator,
}

/// Get total stats aggregated from all containers (from database)
pub async fn get_total_stats(state: web::Data<AppState>) -> impl Responder {
    let query_service = match &state.query_service {
        Some(qs) => qs,
        None => {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Database not available"
            }));
        }
    };

    match query_service.get_total_stats().await {
        Ok(stats) => HttpResponse::Ok().json(serde_json::json!({
            "data": stats
        })),
        Err(e) => {
            log::error!("Failed to get total stats: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get total stats: {}", e)
            }))
        }
    }
}

/// SSE endpoint for comprehensive stats - streams data from database
pub async fn get_total_stats_sse(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let query_service = match &state.query_service {
        Some(qs) => Arc::clone(qs),
        None => {
            return Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Database not available"
            })));
        }
    };

    // Helper function to format stats as SSE data
    let format_stats = |stats: &eyes_devine_shared::TotalStats| -> Result<Bytes, Error> {
        match serde_json::to_string(stats) {
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

    // Get initial stats
    let initial_stats = match query_service.get_total_stats().await {
        Ok(stats) => stats,
        Err(e) => {
            log::error!("Failed to get total stats: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get total stats: {}", e)
            })));
        }
    };

    // Create shared cache for stats
    use std::sync::Arc;
    use tokio::sync::Mutex;
    let cached_stats = Arc::new(Mutex::new(initial_stats));
    let cached_stats_for_updater = Arc::clone(&cached_stats);
    let query_service_for_updater = Arc::clone(&query_service);

    // Spawn background task to update stats continuously
    tokio::spawn(async move {
        let mut update_interval = tokio::time::interval(Duration::from_millis(2000)); // Update every 2 seconds
        loop {
            update_interval.tick().await;
            match query_service_for_updater.get_total_stats().await {
                Ok(stats) => {
                    *cached_stats_for_updater.lock().await = stats;
                }
                Err(e) => {
                    log::error!("Background stats update failed: {}", e);
                }
            }
        }
    });

    // Send first message immediately
    let first_message = format_stats(&*cached_stats.lock().await)?;

    // Create stream that sends cached stats every 2 seconds
    let interval_stream = stream::unfold(cached_stats, move |cached_stats| async move {
        tokio::time::sleep(Duration::from_millis(2000)).await;
        let stats = cached_stats.lock().await.clone();
        let result = format_stats(&stats);
        Some((result, cached_stats))
    });

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

/// List all containers (from database)
pub async fn get_all_containers(state: web::Data<AppState>) -> impl Responder {
    let query_service = match &state.query_service {
        Some(qs) => qs,
        None => {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Database not available"
            }));
        }
    };

    match query_service.get_all_containers().await {
        Ok(containers) => HttpResponse::Ok().json(containers),
        Err(e) => {
            log::error!("Failed to list containers: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to list containers: {}", e)
            }))
        }
    }
}

/// Get latest stats for a specific container (from database)
pub async fn get_container_stats(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let container_id = path.into_inner();
    let query_service = match &state.query_service {
        Some(qs) => qs,
        None => {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Database not available"
            }));
        }
    };

    match query_service.get_latest_container_stats(&container_id).await {
        Ok(Some(stats)) => HttpResponse::Ok().json(stats),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("No stats found for container: {}", container_id)
        })),
        Err(e) => {
            log::error!("Failed to get container stats: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get container stats: {}", e)
            }))
        }
    }
}

/// Get historical stats for a container (from database)
pub async fn get_container_stats_history(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<HistoryQuery>,
) -> impl Responder {
    let container_id = path.into_inner();
    let query_service = match &state.query_service {
        Some(qs) => qs,
        None => {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Database not available"
            }));
        }
    };

    // Validate query parameters
    let (from, to, limit) = match state.query_validator.validate(query.from, query.to, query.limit) {
        Ok(params) => params,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid query parameters: {}", e)
            }));
        }
    };

    match query_service
        .get_container_stats_history(
            &container_id,
            from,
            to,
            limit,
        )
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => {
            log::error!("Failed to get container stats history: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get container stats history: {}", e)
            }))
        }
    }
}

/// Get latest stats for all containers (from database)
pub async fn get_all_container_stats(state: web::Data<AppState>) -> impl Responder {
    let query_service = match &state.query_service {
        Some(qs) => qs,
        None => {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Database not available"
            }));
        }
    };

    match query_service.get_latest_all_container_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => {
            log::error!("Failed to get all container stats: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get all container stats: {}", e)
            }))
        }
    }
}

/// Get logs for a specific container (still from Docker - logs not stored in DB yet)
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

/// Get all images (from database)
pub async fn get_all_images(state: web::Data<AppState>) -> impl Responder {
    let query_service = match &state.query_service {
        Some(qs) => qs,
        None => {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Database not available"
            }));
        }
    };

    match query_service.get_all_images().await {
        Ok(images) => HttpResponse::Ok().json(images),
        Err(e) => {
            log::error!("Failed to get images: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get images: {}", e)
            }))
        }
    }
}

/// Get image by ID (from database)
pub async fn get_image(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let image_id = path.into_inner();
    let query_service = match &state.query_service {
        Some(qs) => qs,
        None => {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Database not available"
            }));
        }
    };

    match query_service.get_image(&image_id).await {
        Ok(Some(image)) => HttpResponse::Ok().json(image),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Image not found: {}", image_id)
        })),
        Err(e) => {
            log::error!("Failed to get image: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get image: {}", e)
            }))
        }
    }
}

/// Get image version history (from database)
pub async fn get_image_history(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<HistoryQuery>,
) -> impl Responder {
    let image_id = path.into_inner();
    let query_service = match &state.query_service {
        Some(qs) => qs,
        None => {
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Database not available"
            }));
        }
    };

    // Validate query parameters
    let (from, to, limit) = match state.query_validator.validate(query.from, query.to, query.limit) {
        Ok(params) => params,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid query parameters: {}", e)
            }));
        }
    };

    match query_service
        .get_image_history(&image_id, from, to, limit)
        .await
    {
        Ok(history) => HttpResponse::Ok().json(history),
        Err(e) => {
            log::error!("Failed to get image history: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get image history: {}", e)
            }))
        }
    }
}

/// Get service communication map
/// Query parameter: `service_id` (optional) - filter to show only connections for a specific service
pub async fn get_service_map(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let service_map_service = ServiceMapService::new(Arc::clone(&state.docker_service));

    // Get optional service_id from query parameters
    let service_id = query.get("service_id").map(|s| s.as_str());

    match service_map_service.generate_service_map_for_service(service_id).await {
        Ok(service_map) => HttpResponse::Ok().json(service_map),
        Err(e) => {
            log::error!("Failed to generate service map: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to generate service map: {}", e)
            }))
        }
    }
}

/// Query parameters for history endpoints
#[derive(serde::Deserialize)]
pub struct HistoryQuery {
    #[serde(default)]
    pub from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub to: Option<DateTime<Utc>>,
    #[serde(default)]
    pub limit: Option<u64>,
}
