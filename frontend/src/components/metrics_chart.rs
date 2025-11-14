use leptos::html;
use leptos::prelude::*;
use plotters::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

#[derive(Clone)]
pub struct DataPoint {
    pub timestamp: f64, // Unix timestamp
    pub cpu: f64,
    pub memory: f64,
    pub network: f64, // Combined RX + TX in KB/s
}

#[component]
pub fn MetricsChart(
    data_points: ReadSignal<Vec<DataPoint>>,
    width: u32,
    height: u32,
) -> impl IntoView {
    let chart_ref = NodeRef::<html::Div>::new();

    Effect::new(move |_| {
        let points = data_points.get();
        
        if let Some(element) = chart_ref.get() {
            let html_element: &HtmlElement = element.dyn_ref().unwrap();
            
            if points.is_empty() {
                html_element.set_inner_html(&format!(
                    r#"<div style="width: {}px; height: {}px; background: #1e1e1e; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #999; font-size: 12px;">No data</div>"#,
                    width, height
                ));
                return;
            }

            // Calculate ranges
            let (min_time, max_time) = if points.len() > 1 {
                (points[0].timestamp, points[points.len() - 1].timestamp)
            } else {
                let t = points[0].timestamp;
                (t - 120.0, t) // Default 2 minute window
            };

            let max_cpu = points.iter().map(|p| p.cpu).fold(0.0, f64::max).max(100.0);
            let max_memory = points.iter().map(|p| p.memory).fold(0.0, f64::max).max(100.0);
            let max_network = points.iter().map(|p| p.network).fold(0.0, f64::max) * 1.1;

            // Create SVG backend
            let mut buffer = String::new();
            {
                let root = SVGBackend::with_string(&mut buffer, (width, height))
                    .into_drawing_area();
                
                // Fill background
                root.fill(&RGBColor(30, 30, 30)).unwrap();
                
                // Create chart with proper ranges
                let mut chart = ChartBuilder::on(&root)
                    .margin(20)
                    .x_label_area_size(0)
                    .y_label_area_size(0)
                    .build_cartesian_2d(
                        min_time..max_time.max(min_time + 1.0),
                        0.0..max_cpu.max(1.0),
                    )
                    .unwrap();

                // Configure mesh (grid)
                chart.configure_mesh()
                    .disable_x_mesh()
                    .disable_y_mesh()
                    .x_label_style(("sans-serif", 10, &RGBColor(150, 150, 150)))
                    .y_label_style(("sans-serif", 10, &RGBColor(150, 150, 150)))
                    .axis_style(&RGBColor(100, 100, 100))
                    .draw()
                    .unwrap();

                // Draw CPU line (yellow)
                let cpu_points: Vec<(f64, f64)> = points.iter().map(|p| (p.timestamp, p.cpu)).collect();
                chart.draw_series(std::iter::once(PathElement::new(
                    cpu_points,
                    &RGBColor(255, 193, 7),
                ))).unwrap();

                // Draw Memory line (blue) - need to scale to same range
                let mem_scale = max_cpu / max_memory.max(1.0);
                let mem_points: Vec<(f64, f64)> = points.iter().map(|p| (p.timestamp, p.memory * mem_scale)).collect();
                chart.draw_series(std::iter::once(PathElement::new(
                    mem_points,
                    &RGBColor(33, 150, 243),
                ))).unwrap();

                // Draw Network line (green) - scale to same range
                let net_scale = max_cpu / max_network.max(1.0);
                let net_points: Vec<(f64, f64)> = points.iter().map(|p| (p.timestamp, p.network * net_scale)).collect();
                chart.draw_series(std::iter::once(PathElement::new(
                    net_points,
                    &RGBColor(76, 175, 80),
                ))).unwrap();
            }

            html_element.set_inner_html(&buffer);
        }
    });

    view! {
        <div
            node_ref=chart_ref
            style=format!("width: {}px; height: {}px; background: #1e1e1e; border-radius: 4px;", width, height)
        ></div>
    }
}
