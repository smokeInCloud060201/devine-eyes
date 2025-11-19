use eyes_devine_shared::{ContainerLog, LogFilter, HttpRequest};
use eyes_devine_services::{CacheService, DockerService, CachedQueryService, ServiceMapService, NetworkMonitorService};
use actix_web::{web, HttpResponse, Responder, Error};
use actix_web::web::Bytes;
use chrono::{Utc, DateTime};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use futures::stream::{self, StreamExt, once};
use std::time::Duration;
use crate::query_validation::HistoryQueryValidator;

pub struct AppState {
    pub docker_service: Arc<DockerService>, // Keep for logs (still need real-time)
    pub db: Option<DatabaseConnection>,
    pub query_service: Option<Arc<CachedQueryService>>,
    pub cache_service: Arc<CacheService>,
    pub query_validator: HistoryQueryValidator,
    pub network_monitor: Option<Arc<NetworkMonitorService>>,
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

/// Get HTTP requests for a specific container/service
/// Uses network-level packet capture (if available) or falls back to log parsing
pub async fn get_container_http_requests(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let container_id = path.into_inner();
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(100);

    // Get container name
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

    let mut http_requests: Vec<HttpRequest> = Vec::new();

    // Try network-level capture first (if available)
    if let Some(network_monitor) = &state.network_monitor {
        match network_monitor.get_container_requests(&container_id).await {
            Ok(requests) => {
                if !requests.is_empty() {
                    log::debug!("Found {} requests from network monitoring", requests.len());
                    http_requests = requests;
                    // Limit results
                    if http_requests.len() > limit as usize {
                        http_requests.truncate(limit as usize);
                    }
                    return HttpResponse::Ok().json(http_requests);
                }
            }
            Err(e) => {
                log::debug!("Network monitoring not available: {}. Falling back to log parsing.", e);
            }
        }
    }

    // Fallback to log parsing
    let logs = match state
        .docker_service
        .get_container_logs(&container_id, None, None, Some(limit))
        .await
    {
        Ok(logs) => logs,
        Err(e) => {
            log::error!("Failed to get container logs: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get container logs: {}", e)
            }));
        }
    };

    // Parse logs for HTTP request patterns
    // Common patterns:
    // - "GET /api/users 200 45ms"
    // - "POST /api/orders 201 123ms"
    // - "127.0.0.1 - - [25/Dec/2024:10:00:00 +0000] \"GET /api/health HTTP/1.1\" 200 45"
    // - "GET /endpoint HTTP/1.1" 200 0.045s
    http_requests = logs
        .iter()
        .enumerate()
        .filter_map(|(idx, log_line)| {
            parse_http_request_from_log(log_line, &container_id, &container_name, idx)
        })
        .collect();

    // If no requests found in logs, check network activity
    if http_requests.is_empty() && !logs.is_empty() {
        if let Ok(stats) = state.docker_service.get_container_stats(&container_id).await {
            if stats.network_rx_bytes > 0 || stats.network_tx_bytes > 0 {
                log::debug!(
                    "No HTTP requests found in logs for container {}, but network activity detected. \
                    Network-level packet capture is being set up to automatically capture requests.",
                    container_id
                );
            }
        }
    }

    HttpResponse::Ok().json(http_requests)
}

/// Parse HTTP request information from a log line
/// Supports multiple common log formats
fn parse_http_request_from_log(
    log_line: &str,
    container_id: &str,
    container_name: &str,
    idx: usize,
) -> Option<HttpRequest> {
    use regex::Regex;
    use std::sync::OnceLock;

    // Pattern 1: "GET /api/users 200 45ms" or "POST /api/orders 201 123.5ms"
    static PATTERN1: OnceLock<Regex> = OnceLock::new();
    let pattern1 = PATTERN1.get_or_init(|| {
        Regex::new(r"(?i)(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\s+([^\s]+)\s+(\d{3})\s+([\d.]+)ms?").unwrap()
    });
    
    // Pattern 2: Apache/Nginx style: "127.0.0.1 - - [timestamp] \"GET /path HTTP/1.1\" 200 45"
    static PATTERN2: OnceLock<Regex> = OnceLock::new();
    let pattern2 = PATTERN2.get_or_init(|| {
        Regex::new(r#""(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\s+([^\s"]+)\s+HTTP/[^"]*"\s+(\d{3})\s+([\d.]+)"#).unwrap()
    });
    
    // Pattern 3: "GET /endpoint HTTP/1.1" 200 0.045s
    static PATTERN3: OnceLock<Regex> = OnceLock::new();
    let pattern3 = PATTERN3.get_or_init(|| {
        Regex::new(r#"(?i)(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\s+([^\s]+)\s+HTTP/[^\s]+\s+(\d{3})\s+([\d.]+)s"#).unwrap()
    });
    
    // Pattern 4: JSON log format: {"method":"GET","path":"/api/users","status":200,"duration":45.2}
    static PATTERN4: OnceLock<Regex> = OnceLock::new();
    let pattern4 = PATTERN4.get_or_init(|| {
        Regex::new(r#""method"\s*:\s*"([^"]+)"[^}]*"path"\s*:\s*"([^"]+)"[^}]*"status"\s*:\s*(\d{3})[^}]*"duration"\s*:\s*([\d.]+)"#).unwrap()
    });

    // Try Pattern 1
    if let Some(caps) = pattern1.captures(log_line) {
        if let (Some(method), Some(endpoint), Some(status), Some(time)) = (
            caps.get(1),
            caps.get(2),
            caps.get(3),
            caps.get(4),
        ) {
            if let (Ok(status_code), Ok(response_time)) = (
                status.as_str().parse::<u16>(),
                time.as_str().parse::<f64>(),
            ) {
                return Some(HttpRequest {
                    container_id: container_id.to_string(),
                    container_name: container_name.to_string(),
                    endpoint: endpoint.as_str().to_string(),
                    method: method.as_str().to_uppercase(),
                    http_status: status_code,
                    response_time_ms: response_time,
                    timestamp: Utc::now() - chrono::Duration::seconds(idx as i64),
                });
            }
        }
    }

    // Try Pattern 2
    if let Some(caps) = pattern2.captures(log_line) {
        if let (Some(method), Some(endpoint), Some(status), Some(time)) = (
            caps.get(1),
            caps.get(2),
            caps.get(3),
            caps.get(4),
        ) {
            if let (Ok(status_code), Ok(response_time)) = (
                status.as_str().parse::<u16>(),
                time.as_str().parse::<f64>(),
            ) {
                return Some(HttpRequest {
                    container_id: container_id.to_string(),
                    container_name: container_name.to_string(),
                    endpoint: endpoint.as_str().to_string(),
                    method: method.as_str().to_uppercase(),
                    http_status: status_code,
                    response_time_ms: response_time,
                    timestamp: Utc::now() - chrono::Duration::seconds(idx as i64),
                });
            }
        }
    }

    // Try Pattern 3
    if let Some(caps) = pattern3.captures(log_line) {
        if let (Some(method), Some(endpoint), Some(status), Some(time)) = (
            caps.get(1),
            caps.get(2),
            caps.get(3),
            caps.get(4),
        ) {
            if let (Ok(status_code), Ok(response_time_sec)) = (
                status.as_str().parse::<u16>(),
                time.as_str().parse::<f64>(),
            ) {
                return Some(HttpRequest {
                    container_id: container_id.to_string(),
                    container_name: container_name.to_string(),
                    endpoint: endpoint.as_str().to_string(),
                    method: method.as_str().to_uppercase(),
                    http_status: status_code,
                    response_time_ms: response_time_sec * 1000.0, // Convert seconds to ms
                    timestamp: Utc::now() - chrono::Duration::seconds(idx as i64),
                });
            }
        }
    }

    // Try Pattern 4 (JSON)
    if let Some(caps) = pattern4.captures(log_line) {
        if let (Some(method), Some(endpoint), Some(status), Some(time)) = (
            caps.get(1),
            caps.get(2),
            caps.get(3),
            caps.get(4),
        ) {
            if let (Ok(status_code), Ok(response_time)) = (
                status.as_str().parse::<u16>(),
                time.as_str().parse::<f64>(),
            ) {
                return Some(HttpRequest {
                    container_id: container_id.to_string(),
                    container_name: container_name.to_string(),
                    endpoint: endpoint.as_str().to_string(),
                    method: method.as_str().to_uppercase(),
                    http_status: status_code,
                    response_time_ms: response_time,
                    timestamp: Utc::now() - chrono::Duration::seconds(idx as i64),
                });
            }
        }
    }

    None
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
