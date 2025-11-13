use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;

use crate::pages::DockerMonitor;

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
