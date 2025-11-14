use leptos::prelude::*;
use eyes_devine_shared::{ContainerInfo, ContainerStats};
use crate::utils::format_bytes;
use crate::components::{MetricsChart, DataPoint};
use indexmap::IndexMap;
use std::sync::Arc;

#[component]
pub fn ContainerCard(
    container: ContainerInfo,
    stats: Option<ContainerStats>,
    container_id: String,
    historical_data: ReadSignal<IndexMap<String, Vec<DataPoint>>>,
    on_click: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    let container_name = stats
        .as_ref()
        .map(|s| s.container_name.clone())
        .unwrap_or_else(|| container.name.clone());

    let has_stats = stats.is_some();
    let (chart_data, set_chart_data) = signal(Vec::<DataPoint>::new());

    // Update chart data reactively
    Effect::new(move |_| {
        let data = historical_data
            .get()
            .get(&container_id)
            .cloned()
            .unwrap_or_default();
        set_chart_data.set(data);
    });

    // Extract formatted stats strings before closures to avoid FnOnce issues
    // Use Arc to share the tuple so it can be cloned in multiple closures (thread-safe)
    let stats_tuple = Arc::new(stats.as_ref().map(|s| (
        format!("{:.2}%", s.cpu_usage_percent),
        format_bytes(s.memory_usage_bytes),
        format_bytes(s.memory_limit_bytes),
        format!("{:.2}%", s.memory_usage_percent),
        format_bytes(s.network_rx_bytes),
        format_bytes(s.network_tx_bytes),
    )).unwrap_or_else(|| (String::new(), String::new(), String::new(), String::new(), String::new(), String::new())));
    let stats_tuple_1 = Arc::clone(&stats_tuple);
    let stats_tuple_2 = Arc::clone(&stats_tuple);

    view! {
        <div class="container-card" on:click=move |_| on_click()>
            <h3>{container_name}</h3>
            <div class="container-info">
                {"ID: "} {container.id.chars().take(12).collect::<String>()} {"..."}
            </div>
            <div class="container-info">{"Image: "} {container.image.clone()}</div>
            <div class="container-info">{"Status: "} {container.status.clone()}</div>

            // Stats available & chart data not empty
            <Show when=move || has_stats && !chart_data.get().is_empty()>
                {
                    let stats_tuple_clone = Arc::clone(&stats_tuple_1);
                    move || {
                        let (cpu, mem_used, mem_limit, mem_pct, net_rx, net_tx) = stats_tuple_clone.as_ref().clone();
                        view! {
                            <div style="margin-top: 15px; padding-top: 15px; border-top: 1px solid #ddd;">
                                <div style="margin-bottom: 10px;">
                                    <MetricsChart data_points=chart_data width=300 height=120/>
                                </div>
                                <div class="container-info" style="font-size: 0.85em; color: #666;">
                                    <span style="color: #FFC107;">"●"</span> " CPU  "
                                    <span style="color: #2196F3;">"●"</span> " Memory  "
                                    <span style="color: #4CAF50;">"●"</span> " Network"
                                </div>
                                <div class="container-info"><strong>"CPU: "</strong>{cpu}</div>
                                <div class="container-info">
                                    <strong>"Memory: "</strong>{mem_used} {" / "} {mem_limit} {" ("} {mem_pct} {")"}
                                </div>
                                <div class="container-info"><strong>"Network RX: "</strong>{net_rx}</div>
                                <div class="container-info"><strong>"Network TX: "</strong>{net_tx}</div>
                            </div>
                        }
                    }
                }
            </Show>

            // Stats available & chart data empty
            <Show when=move || has_stats && chart_data.get().is_empty()>
                {
                    let stats_tuple_clone = Arc::clone(&stats_tuple_2);
                    move || {
                        let (cpu, mem_used, mem_limit, mem_pct, net_rx, net_tx) = stats_tuple_clone.as_ref().clone();
                        view! {
                            <div style="margin-top: 15px; padding-top: 15px; border-top: 1px solid #ddd;">
                                <div class="container-info"><strong>"CPU: "</strong>{cpu}</div>
                                <div class="container-info"><strong>"Memory: "</strong>{mem_used} {" / "} {mem_limit} {" ("} {mem_pct} {")"}</div>
                                <div class="container-info"><strong>"Network RX: "</strong>{net_rx}</div>
                                <div class="container-info"><strong>"Network TX: "</strong>{net_tx}</div>
                            </div>
                        }
                    }
                }
            </Show>

            // No stats available
            <Show when=move || !has_stats>
                <div class="container-info" style="color: #999; margin-top: 10px;">
                    "Stats unavailable"
                </div>
            </Show>
        </div>
    }
}
