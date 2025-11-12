use crate::handlers;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/api/stats/total", web::get().to(handlers::get_total_stats))
        .route("/api/containers", web::get().to(handlers::get_all_containers))
        .route(
            "/api/containers/{id}/stats",
            web::get().to(handlers::get_container_stats),
        )
        .route(
            "/api/containers/stats",
            web::get().to(handlers::get_all_container_stats),
        )
        .route(
            "/api/containers/{id}/logs",
            web::get().to(handlers::get_container_logs),
        );
}

