use leptos::prelude::*;
use leptos::task::spawn_local;
use eyes_devine_shared::{ContainerLog, ComprehensiveStats, ContainerInfo};
use crate::components::{ContainersView, LogsView, TotalStatsView, DataPoint};
use crate::services::{load_logs, connect_sse_stats};
use indexmap::IndexMap;

#[component]
pub fn DockerMonitor() -> impl IntoView {
    let (comprehensive_stats, set_comprehensive_stats) = signal(None::<ComprehensiveStats>);
    let (selected_container, set_selected_container) = signal(None::<String>);
    let (logs, set_logs) = signal(Vec::<ContainerLog>::new());
    let (log_limit, set_log_limit) = signal(100u64);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (sse_connected, set_sse_connected) = signal(false);

    // Connect to SSE endpoint /api/stats/total/sse for real-time container status updates
    // This SSE stream provides ComprehensiveStats which includes:
    // - Container list with status (running, stopped, etc.)
    // - Real-time stats for each container (CPU, memory, network)
    // - Total aggregated stats
    // Updates are received every 2 seconds from the server
    // Note: EventSource will be automatically closed by the browser when the page unloads.
    // For a single-page app, this is sufficient. The connection will remain open while
    // the component is mounted and will be cleaned up by the browser on page unload.
    let _event_source = connect_sse_stats(
        set_comprehensive_stats,
        set_error,
        set_sse_connected,
    );

    // Extract containers and stats from comprehensive stats using signals
    // Container status (running/stopped/etc.) is updated in real-time via SSE
    // We use signals instead of memos because the types don't implement PartialEq
    let (containers, set_containers) = signal(Vec::<ContainerInfo>::new());
    let (container_stats, set_container_stats) = signal(Vec::<eyes_devine_shared::ContainerStats>::new());
    
    // Historical data for charts - store up to 60 data points (2 minutes at 2s intervals)
    let (historical_data, set_historical_data) = signal(IndexMap::<String, Vec<DataPoint>>::new());
    const MAX_HISTORY: usize = 60;
    
    // Update containers and stats reactively when comprehensive_stats changes via SSE
    // This Effect runs whenever new data arrives from the SSE stream
    // Container status is extracted from cs.containers[].status
    // Use Effect::new() instead of deprecated create_effect()
    Effect::new(move |_| {
        let stats = comprehensive_stats.get();
        if let Some(cs) = stats.as_ref() {
            web_sys::console::log_1(&format!("DockerMonitor: ComprehensiveStats updated, {} containers", cs.containers.len()).into());
            // Extract container info including status from SSE data
            let new_containers: Vec<ContainerInfo> = cs.containers.iter().map(|cd| {
                ContainerInfo {
                    id: cd.container_id.clone(),
                    name: cd.container_name.clone(),
                    image: cd.image.clone(),
                    status: cd.status.clone(), // Container status updated via SSE
                    created: cd.created,
                }
            }).collect();
            set_containers.set(new_containers);
            
            let new_stats: Vec<eyes_devine_shared::ContainerStats> = cs.containers.iter().map(|cd| cd.stats.clone()).collect();
            let new_stats_clone = new_stats.clone();
            set_container_stats.set(new_stats);
            
            // Update historical data for charts
            let mut hist = historical_data.get();
            for stats in &new_stats_clone {
                let timestamp = stats.timestamp.timestamp() as f64;
                
                // Calculate network rate (KB/s) - for now use total bytes as approximation
                // In a real implementation, you'd calculate the rate between consecutive points
                let network_kb = (stats.network_rx_bytes + stats.network_tx_bytes) as f64 / 1024.0;
                
                let data_point = DataPoint {
                    timestamp,
                    cpu: stats.cpu_usage_percent,
                    memory: stats.memory_usage_percent,
                    network: network_kb,
                };
                
                let entry = hist.entry(stats.container_id.clone()).or_insert_with(Vec::new);
                entry.push(data_point);
                
                // Keep only the last MAX_HISTORY points
                if entry.len() > MAX_HISTORY {
                    entry.remove(0);
                }
            }
            set_historical_data.set(hist);
        } else {
            set_containers.set(Vec::new());
            set_container_stats.set(Vec::new());
        }
    });

    view! {
        <div class="container">
            <div class="header">
                <h1>"🐳 Docker Monitor - Eyes Devine"</h1>
                <Show when=move || sse_connected.get()>
                    <div style="color: #4CAF50; font-size: 0.9em; margin-top: 5px;">
                        "🟢 Connected (Real-time updates every 2s)"
                    </div>
                </Show>
                <Show when=move || !sse_connected.get()>
                    <div style="color: #f44336; font-size: 0.9em; margin-top: 5px;">
                        "🔴 Disconnected"
                    </div>
                </Show>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="error">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || loading.get()>
                <div class="loading">"Loading..."</div>
            </Show>

            <TotalStatsView stats=comprehensive_stats/>
            <ContainersView
                containers=containers
                container_stats=container_stats
                historical_data=historical_data
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

