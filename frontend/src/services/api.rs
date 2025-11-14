use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use eyes_devine_shared::{ContainerInfo, ContainerLog, ContainerStats, TotalStats, ComprehensiveStats};

/// Get the API base URL from environment or use default
fn api_base() -> String {
    std::option_env!("BACKEND_API_URL")
        .unwrap_or("http://127.0.0.1:8080")
        .to_string()
}

/// Generic JSON fetch function
async fn fetch_json<T>(url: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let window = web_sys::window().ok_or("No window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|e| format!("Response error: {:?}", e))?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let json = JsFuture::from(
        resp.json()
            .map_err(|e| format!("JSON error: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("JSON parse error: {:?}", e))?;

    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize error: {:?}", e))
}

/// Response wrapper for API endpoints that return data in a data field
#[derive(serde::Deserialize)]
struct DataResponse<T> {
    data: T,
}

/// Fetch total stats from the API
/// 
/// **DEPRECATED**: This function uses HTTP GET. Use `connect_sse_stats()` instead
/// for real-time updates via Server-Sent Events (SSE).
#[deprecated(note = "Use connect_sse_stats() for SSE-based real-time updates")]
pub async fn fetch_total_stats() -> Result<TotalStats, String> {
    let url = format!("{}/api/stats/total", api_base());
    let response: DataResponse<ComprehensiveStats> = fetch_json(&url).await?;
    Ok(response.data.total_stats)
}

/// Fetch all containers from the API
pub async fn fetch_containers() -> Result<Vec<ContainerInfo>, String> {
    let url = format!("{}/api/containers", api_base());
    fetch_json::<Vec<ContainerInfo>>(&url).await
}

/// Fetch stats for a specific container
pub async fn fetch_container_stats(container_id: &str) -> Result<ContainerStats, String> {
    let url = format!("{}/api/containers/{}/stats", api_base(), container_id);
    fetch_json::<ContainerStats>(&url).await
}

/// Fetch logs for a specific container
pub async fn fetch_container_logs(container_id: &str, limit: u64) -> Result<Vec<ContainerLog>, String> {
    let url = format!("{}/api/containers/{}/logs?limit={}", api_base(), container_id, limit);
    fetch_json::<Vec<ContainerLog>>(&url).await
}

/// Refresh all data (total stats, containers, and container stats)
/// 
/// **DEPRECATED**: This function uses HTTP polling. Use SSE connection via `connect_sse_stats()` instead.
#[deprecated(note = "Use connect_sse_stats() for SSE-based real-time updates")]
#[allow(deprecated)]
pub async fn refresh_all(
    set_total_stats: WriteSignal<Option<TotalStats>>,
    set_containers: WriteSignal<Vec<ContainerInfo>>,
    set_container_stats: WriteSignal<Vec<ContainerStats>>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    set_loading.set(true);
    set_error.set(None);

    // Load total stats
    #[allow(deprecated)]
    if let Ok(stats) = fetch_total_stats().await {
        set_total_stats.set(Some(stats));
    }

    // Load containers
    if let Ok(containers) = fetch_containers().await {
        set_containers.set(containers.clone());

        // Load stats for each container
        let mut stats = Vec::new();
        for container in containers {
            if let Ok(stat) = fetch_container_stats(&container.id).await {
                stats.push(stat);
            }
        }
        set_container_stats.set(stats);
    }

    set_loading.set(false);
}

/// Load logs for a container
pub async fn load_logs(
    container_id: Option<String>,
    limit: u64,
    set_logs: WriteSignal<Vec<ContainerLog>>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    if let Some(id) = container_id {
        set_loading.set(true);
        set_error.set(None);

        if let Ok(logs) = fetch_container_logs(&id, limit).await {
            set_logs.set(logs);
        }

        set_loading.set(false);
    }
}

/// Connect to SSE endpoint for comprehensive stats at /api/stats/total/sse
/// 
/// This SSE connection provides real-time updates for:
/// - Container status (running, stopped, etc.) - updated every 2 seconds
/// - Container stats (CPU, memory, network) - updated every 2 seconds
/// - Total aggregated stats - updated every 2 seconds
/// 
/// The ComprehensiveStats structure includes a list of containers with their current status
/// and real-time performance metrics.
/// 
/// Returns the EventSource which should be stored and cleaned up on component unmount
pub fn connect_sse_stats(
    set_comprehensive_stats: WriteSignal<Option<ComprehensiveStats>>,
    set_error: WriteSignal<Option<String>>,
    set_connected: WriteSignal<bool>,
) -> web_sys::EventSource {
    let url = format!("{}/api/stats/total/sse", api_base());
    
    // Create EventSource for SSE
    let event_source = web_sys::EventSource::new(&url)
        .expect("Failed to create EventSource");
    
    // Clone signals for closures
    let set_comprehensive_stats_clone = set_comprehensive_stats.clone();
    let set_error_clone = set_error.clone();
    let set_connected_clone = set_connected.clone();
    
    // Handle messages
    // EventSource automatically parses "data: {...}\n\n" and gives us just the JSON in event.data()
    let onmessage_callback = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        if let Some(data) = event.data().as_string() {
            // Parse the JSON data
            match serde_json::from_str::<ComprehensiveStats>(&data) {
                Ok(stats) => {
                    web_sys::console::log_1(&format!("Received SSE data: {} containers", stats.containers.len()).into());
                    set_comprehensive_stats_clone.set(Some(stats));
                    set_error_clone.set(None);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to parse SSE data: {}", e).into());
                    set_error_clone.set(Some(format!("Parse error: {}", e)));
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    
    event_source.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget(); // Keep the closure alive
    
    // Handle connection open
    let onopen_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        set_connected_clone.set(true);
        set_error_clone.set(None);
        web_sys::console::log_1(&"SSE connection opened to /api/stats/total/sse".into());
    }) as Box<dyn FnMut(_)>);
    
    event_source.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
    
    // Handle errors
    let set_connected_error = set_connected.clone();
    let set_error_error = set_error.clone();
    let event_source_clone = event_source.clone();
    let onerror_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        set_connected_error.set(false);
        // Only set error if connection was previously open (to avoid spam on initial connection)
        if event_source_clone.ready_state() == web_sys::EventSource::CLOSED {
            set_error_error.set(Some("SSE connection closed. Attempting to reconnect...".to_string()));
            web_sys::console::warn_1(&"SSE connection closed".into());
        }
    }) as Box<dyn FnMut(_)>);
    
    event_source.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();
    
    event_source
}

