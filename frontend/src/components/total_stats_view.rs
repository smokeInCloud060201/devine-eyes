use leptos::prelude::*;
use eyes_devine_shared::ComprehensiveStats;
use crate::components::StatCard;
use crate::utils::format_bytes;

#[component]
pub fn TotalStatsView(stats: ReadSignal<Option<ComprehensiveStats>>) -> impl IntoView {
    // Debug: log when stats change
    Effect::new(move |_| {
        let current_stats = stats.get();
        if current_stats.is_some() {
            web_sys::console::log_1(&format!("TotalStatsView: Stats updated, {} containers", current_stats.as_ref().unwrap().containers.len()).into());
        } else {
            web_sys::console::log_1(&"TotalStatsView: Stats is None".into());
        }
    });

    view! {
        <div class="stats-grid">
            <Show
                when=move || stats.get().is_some()
                fallback=move || view! {
                    <div class="loading">"Loading total stats..."</div>
                }
            >
                {move || {
                    let cs = stats.get().unwrap();
                    let s = &cs.total_stats;
                    view! {
                        <StatCard title="Total Containers".to_string() value=cs.total_containers.to_string() unit="".to_string()/>
                        <StatCard title="Containers Up".to_string() value=cs.containers_up.to_string() unit="".to_string()/>
                        <StatCard title="Containers Down".to_string() value=cs.containers_down.to_string() unit="".to_string()/>
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
                    }
                }}
            </Show>
        </div>
    }
}

