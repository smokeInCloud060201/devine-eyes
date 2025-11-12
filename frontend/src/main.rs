#[cfg(feature = "ssr")]
use axum::Router;
#[cfg(feature = "ssr")]
use leptos::*;
#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;
#[cfg(feature = "ssr")]
use leptos_axum::{generate_route_list, LeptosRoutes};
#[cfg(feature = "ssr")]
use tower_http::services::ServeDir;
#[cfg(feature = "ssr")]
use tracing_subscriber;
use eyes_devine_frontend::App;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Detect environment (dev or prod)
    let env = if cfg!(debug_assertions) {
        leptos::config::Env::DEV
    } else {
        leptos::config::Env::PROD
    };

    // Set up Leptos configuration
    let leptos_options = LeptosOptions::builder()
        .output_name("eyes-devine-frontend")
        .site_root("target/site")
        .site_pkg_dir("pkg")
        .env(env)
        .build();
    
    // Get port from environment or default to 3000
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    // Generate route list for SSR
    let routes = generate_route_list(App);

    // Build application with routes
    let app = Router::new()
        .leptos_routes_with_handler(routes, leptos_axum::render_app_to_stream(leptos_options.clone(), App))
        .fallback_service(ServeDir::new(&leptos_options.site_root));

    tracing::info!("Frontend server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

