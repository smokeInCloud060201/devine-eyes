use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::*;
use leptos_router::components::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use wasm_bindgen::JsCast;

// API Models (matching backend/shared/src/models.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub container_id: String,
    pub container_name: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_usage_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub block_read_bytes: u64,
    pub block_write_bytes: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalStats {
    pub total_containers: usize,
    pub total_cpu_usage_percent: f64,
    pub total_memory_usage_bytes: u64,
    pub total_memory_limit_bytes: u64,
    pub total_memory_usage_percent: f64,
    pub total_network_rx_bytes: u64,
    pub total_network_tx_bytes: u64,
    pub total_block_read_bytes: u64,
    pub total_block_write_bytes: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerLog {
    pub container_id: String,
    pub container_name: String,
    pub log_line: String,
    pub timestamp: DateTime<Utc>,
    pub stream: String,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/frontend.css"/>
        <Title text="Docker Monitor - Eyes Devine"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Router>
            <Routes fallback=move || view! { <div>"404 - Not Found"</div> }>
                <Route path=() view=DockerMonitor/>
            </Routes>
        </Router>
    }
}

#[component]
fn DockerMonitor() -> impl IntoView {
    let (total_stats, set_total_stats) = signal(None::<TotalStats>);
    let (containers, set_containers) = signal(Vec::<ContainerInfo>::new());
    let (container_stats, set_container_stats) = signal(Vec::<ContainerStats>::new());
    let (selected_container, set_selected_container) = signal(None::<String>);
    let (logs, set_logs) = signal(Vec::<ContainerLog>::new());
    let (log_limit, set_log_limit) = signal(100u64);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    // Load data on mount (client-side only)
    {
        spawn_local(async move {
            refresh_all(
                set_total_stats,
                set_containers,
                set_container_stats,
                set_loading,
                set_error,
            )
            .await;
        });
    }

    view! {
        <div class="container">
            <div class="header">
                <h1>"🐳 Docker Monitor - Eyes Devine"</h1>
            </div>

            <button
                class="refresh-btn"
                on:click=move |_| {
                    spawn_local(async move {
                        refresh_all(
                            set_total_stats,
                            set_containers,
                            set_container_stats,
                            set_loading,
                            set_error,
                        )
                        .await;
                    });
                }
            >
                "🔄 Refresh All"
            </button>

            <Show when=move || error.get().is_some()>
                <div class="error">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || loading.get()>
                <div class="loading">"Loading..."</div>
            </Show>

            <TotalStatsView stats=total_stats/>
            <ContainersView
                containers=containers
                container_stats=container_stats
                on_select=move |id: String| {
                    set_selected_container.set(Some(id.clone()));
                    spawn_local(async move {
                        load_logs(
                            Some(id),
                            log_limit.get(),
                            set_logs,
                            set_loading,
                            set_error,
                        )
                        .await;
                    });
                }
            />
            <LogsView
                containers=containers
                selected_container=selected_container
                logs=logs
                log_limit=log_limit
                on_container_change=move |id: String| {
                    set_selected_container.set(Some(id.clone()));
                    spawn_local(async move {
                        load_logs(
                            Some(id),
                            log_limit.get(),
                            set_logs,
                            set_loading,
                            set_error,
                        )
                        .await;
                    });
                }
                on_limit_change=move |limit: u64| {
                    set_log_limit.set(limit);
                    if let Some(container_id) = selected_container.get() {
                        spawn_local(async move {
                            load_logs(
                                Some(container_id),
                                limit,
                                set_logs,
                                set_loading,
                                set_error,
                            )
                            .await;
                        });
                    }
                }
                on_load=move || {
                    if let Some(container_id) = selected_container.get() {
                        spawn_local(async move {
                            load_logs(
                                Some(container_id),
                                log_limit.get(),
                                set_logs,
                                set_loading,
                                set_error,
                            )
                            .await;
                        });
                    }
                }
                on_clear=move || {
                    set_logs.set(Vec::new());
                }
            />
        </div>
    }
}

#[component]
fn TotalStatsView(stats: ReadSignal<Option<TotalStats>>) -> impl IntoView {
    view! {
        <div class="stats-grid">
            <Show when=move || stats.get().is_some()>
                {move || {
                    stats.get().map(|s| view! {
                        <StatCard title="Total Containers".to_string() value=s.total_containers.to_string() unit="".to_string()/>
                        <StatCard title="CPU Usage".to_string() value=format!("{:.2}%", s.total_cpu_usage_percent) unit="".to_string()/>
                        <StatCard
                            title="Memory Usage".to_string()
                            value=format_bytes(s.total_memory_usage_bytes)
                            unit=format!("/ {}", format_bytes(s.total_memory_limit_bytes))
                        />
                        <StatCard
                            title="Memory %".to_string()
                            value=format!("{:.2}%", s.total_memory_usage_percent)
                            unit="".to_string()
                        />
                        <StatCard title="Network RX".to_string() value=format_bytes(s.total_network_rx_bytes) unit="".to_string()/>
                        <StatCard title="Network TX".to_string() value=format_bytes(s.total_network_tx_bytes) unit="".to_string()/>
                        <StatCard title="Block Read".to_string() value=format_bytes(s.total_block_read_bytes) unit="".to_string()/>
                        <StatCard title="Block Write".to_string() value=format_bytes(s.total_block_write_bytes) unit="".to_string()/>
                    })
                }}
            </Show>
            <Show when=move || stats.get().is_none()>
                <div class="loading">"Loading total stats..."</div>
            </Show>
        </div>
    }
}

#[component]
fn StatCard(title: String, value: String, unit: String) -> impl IntoView {
    view! {
        <div class="stat-card">
            <h3>{title}</h3>
            <div class="stat-value">
                {value}
                <span class="stat-unit">{unit}</span>
            </div>
        </div>
    }
}

#[component]
fn ContainersView(
    containers: ReadSignal<Vec<ContainerInfo>>,
    container_stats: ReadSignal<Vec<ContainerStats>>,
    on_select: impl Fn(String) + Send + Sync + Clone + 'static,
) -> impl IntoView {
    view! {
        <div class="containers-section">
            <h2>"Containers"</h2>
            <div class="containers-grid">
                <For
                    each=move || {
                        containers
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, c)| (i, c))
                    }
                    key=|(i, _)| *i
                    children=move |(_i, container)| {
                        let stats = container_stats
                            .get()
                            .into_iter()
                            .find(|s| s.container_id == container.id);
                        let container_id = container.id.clone();
                        let on_select = on_select.clone();
                        view! {
                            <ContainerCard
                                container=container
                                stats=stats
                                on_click=move || {
                                    on_select(container_id.clone());
                                }
                            />
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn ContainerCard(
    container: ContainerInfo,
    stats: Option<ContainerStats>,
    on_click: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    let container_name = stats
        .as_ref()
        .map(|s| s.container_name.clone())
        .unwrap_or_else(|| container.name.clone());
    
    // Format all stats strings before the view to avoid closure move issues
    let (cpu, mem_used, mem_limit, mem_pct, net_rx, net_tx, block_r, block_w) = if let Some(s) = stats.as_ref() {
        (
            format!("{:.2}%", s.cpu_usage_percent),
            format_bytes(s.memory_usage_bytes),
            format_bytes(s.memory_limit_bytes),
            format!("{:.2}%", s.memory_usage_percent),
            format_bytes(s.network_rx_bytes),
            format_bytes(s.network_tx_bytes),
            format_bytes(s.block_read_bytes),
            format_bytes(s.block_write_bytes),
        )
    } else {
        (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        )
    };
    
    let has_stats = stats.is_some();
    
    view! {
        <div class="container-card" on:click=move |_| on_click()>
            <h3>{container_name}</h3>
            <div class="container-info">
                {"ID: "}
                {container.id.chars().take(12).collect::<String>()}
                {"..."}
            </div>
            <div class="container-info">{"Image: "} {container.image.clone()}</div>
            <div class="container-info">{"Status: "} {container.status.clone()}</div>
            <Show when=move || has_stats>
                <div style="margin-top: 15px; padding-top: 15px; border-top: 1px solid #ddd;">
                    <div class="container-info">
                        <strong>"CPU: "</strong>
                        {cpu.clone()}
                    </div>
                    <div class="container-info">
                        <strong>"Memory: "</strong>
                        {mem_used.clone()}
                        {" / "}
                        {mem_limit.clone()}
                        {" ("}
                        {mem_pct.clone()}
                        {")"}
                    </div>
                    <div class="container-info">
                        <strong>"Network RX: "</strong>
                        {net_rx.clone()}
                    </div>
                    <div class="container-info">
                        <strong>"Network TX: "</strong>
                        {net_tx.clone()}
                    </div>
                    <div class="container-info">
                        <strong>"Block Read: "</strong>
                        {block_r.clone()}
                    </div>
                    <div class="container-info">
                        <strong>"Block Write: "</strong>
                        {block_w.clone()}
                    </div>
                </div>
            </Show>
            <Show when=move || !has_stats>
                <div class="container-info" style="color: #999; margin-top: 10px;">
                    "Stats unavailable"
                </div>
            </Show>
        </div>
    }
}

#[component]
fn LogsView(
    containers: ReadSignal<Vec<ContainerInfo>>,
    selected_container: ReadSignal<Option<String>>,
    logs: ReadSignal<Vec<ContainerLog>>,
    log_limit: ReadSignal<u64>,
    on_container_change: impl Fn(String) + Send + Sync + 'static,
    on_limit_change: impl Fn(u64) + Send + Sync + 'static,
    on_load: impl Fn() + Send + Sync + 'static,
    on_clear: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="logs-section">
            <h2>"Container Logs"</h2>
            <div class="logs-controls">
                <select
                    on:change=move |ev| {
                        if let Some(select) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok()) {
                            on_container_change(select.value());
                        }
                    }
                >
                    <option value="" selected=move || selected_container.get().is_none()>
                        "Select a container..."
                    </option>
                    <For
                        each=move || containers.get()
                        key=|c| c.id.clone()
                        children=move |container| {
                            let container_id = container.id.clone();
                            let is_selected = move || selected_container.get().as_ref() == Some(&container_id);
                            view! {
                                <option value=container.id.clone() selected=is_selected>
                                    {format!("{} ({})", container.name, container.status)}
                                </option>
                            }
                        }
                    />
                </select>
                <input
                    type="number"
                    placeholder="Limit (default: 100)"
                    min="1"
                    max="1000"
                    value=move || log_limit.get().to_string()
                    on:input=move |ev| {
                        if let Some(input) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                            if let Ok(limit) = input.value().parse::<u64>() {
                                on_limit_change(limit);
                            }
                        }
                    }
                />
                <button on:click=move |_| on_load()>"Load Logs"</button>
                <button on:click=move |_| on_clear()>"Clear"</button>
            </div>
            <div class="logs-container">
                <Show when=move || !logs.get().is_empty()>
                    <For
                        each=move || logs.get()
                        key=|log| format!("{}-{}", log.container_id, log.timestamp)
                        children=move |log| {
                            let stream_class = if log.stream == "stderr" {
                                "stderr"
                            } else {
                                "stdout"
                            };
                            view! {
                                <div class=format!("log-line {}", stream_class)>
                                    {format!("[{}] {}", log.timestamp.format("%Y-%m-%d %H:%M:%S"), log.log_line)}
                                </div>
                            }
                        }
                    />
                </Show>
                <Show when=move || logs.get().is_empty() && selected_container.get().is_none()>
                    <div class="loading">"Select a container to view logs..."</div>
                </Show>
                <Show when=move || logs.get().is_empty() && selected_container.get().is_some()>
                    <div style="color: #999;">"No logs available for this container."</div>
                </Show>
            </div>
        </div>
    }
}

// API client functions
async fn refresh_all(
    set_total_stats: WriteSignal<Option<TotalStats>>,
    set_containers: WriteSignal<Vec<ContainerInfo>>,
    set_container_stats: WriteSignal<Vec<ContainerStats>>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    set_loading.set(true);
    set_error.set(None);

    // Get backend API URL from environment or use default
    let api_base = std::option_env!("BACKEND_API_URL")
        .unwrap_or("http://127.0.0.1:8080");

    // Load total stats
    if let Ok(stats) = fetch_json::<TotalStats>(&format!("{}/api/stats/total", api_base)).await {
        set_total_stats.set(Some(stats));
    }

    // Load containers
    if let Ok(containers) = fetch_json::<Vec<ContainerInfo>>(&format!("{}/api/containers", api_base)).await {
        set_containers.set(containers.clone());

        // Load stats for each container
        let mut stats = Vec::new();
        for container in containers {
            if let Ok(stat) = fetch_json::<ContainerStats>(&format!("{}/api/containers/{}/stats", api_base, container.id)).await {
                stats.push(stat);
            }
        }
        set_container_stats.set(stats);
    }

    set_loading.set(false);
}

async fn load_logs(
    container_id: Option<String>,
    limit: u64,
    set_logs: WriteSignal<Vec<ContainerLog>>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    if let Some(id) = container_id {
        set_loading.set(true);
        set_error.set(None);

        let api_base = std::option_env!("BACKEND_API_URL")
            .unwrap_or("http://127.0.0.1:8080");

        let url = format!("{}/api/containers/{}/logs?limit={}", api_base, id, limit);
        if let Ok(logs) = fetch_json::<Vec<ContainerLog>>(&url).await {
            set_logs.set(logs);
        }

        set_loading.set(false);
    }
}

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

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let k: f64 = 1024.0;
    let sizes = ["B", "KB", "MB", "GB", "TB"];
    let i = (bytes as f64).log2() / k.log2();
    let i = i.floor() as usize;
    let i = i.min(sizes.len() - 1);
    let size = sizes[i];
    format!("{:.2} {}", bytes as f64 / k.powi(i as i32), size)
}
