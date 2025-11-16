use crate::handlers;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Stats endpoints
        .route("/api/stats/total", web::get().to(handlers::get_total_stats))
        .route("/api/stats/total/sse", web::get().to(handlers::get_total_stats_sse))
        
        // Container endpoints
        .route("/api/containers", web::get().to(handlers::get_all_containers))
        .route(
            "/api/containers/{id}/stats",
            web::get().to(handlers::get_container_stats),
        )
        .route(
            "/api/containers/{id}/stats/history",
            web::get().to(handlers::get_container_stats_history),
        )
        .route(
            "/api/containers/stats",
            web::get().to(handlers::get_all_container_stats),
        )
        .route(
            "/api/containers/{id}/logs",
            web::get().to(handlers::get_container_logs),
        )
        
        // Image endpoints
        .route("/api/images", web::get().to(handlers::get_all_images))
        .route(
            "/api/images/{id}",
            web::get().to(handlers::get_image),
        )
        .route(
            "/api/images/{id}/history",
            web::get().to(handlers::get_image_history),
        )
        
        // Service map endpoint
        .route("/api/services/map", web::get().to(handlers::get_service_map));
}

