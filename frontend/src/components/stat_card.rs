use leptos::prelude::*;

#[component]
pub fn StatCard(title: String, value: String, unit: String) -> impl IntoView {
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

