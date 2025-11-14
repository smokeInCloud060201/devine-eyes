use leptos::prelude::*;
use wasm_bindgen::JsCast;
use eyes_devine_shared::{ContainerInfo, ContainerLog};

#[component]
pub fn LogsView(
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

