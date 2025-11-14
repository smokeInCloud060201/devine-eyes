use leptos::prelude::*;
use eyes_devine_shared::{ContainerInfo, ContainerStats};
use crate::components::{ContainerCard, DataPoint};
use indexmap::IndexMap;

#[component]
pub fn ContainersView(
    containers: ReadSignal<Vec<ContainerInfo>>,
    container_stats: ReadSignal<Vec<ContainerStats>>,
    historical_data: ReadSignal<IndexMap<String, Vec<DataPoint>>>,
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
                        let historical_data_clone = historical_data;
                        view! {
                            <ContainerCard
                                container=container
                                stats=stats
                                container_id=container_id.clone()
                                historical_data=historical_data_clone
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

