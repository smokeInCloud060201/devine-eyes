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

#[derive(Clone)]
pub struct MetricSeries {
    pub name: String,
    pub color: String,
    pub metrics: Vec<f64>,
}

// Helper function to parse color string to RGBColor
fn parse_color(color_str: &str) -> RGBColor {
    let color_str = color_str.trim();
    
    // Handle hex colors like "#FFC107" or "FFC107"
    if color_str.starts_with('#') {
        let hex = &color_str[1..];
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return RGBColor(r, g, b);
            }
        }
    } else if color_str.len() == 6 {
        // Hex without #
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&color_str[0..2], 16),
            u8::from_str_radix(&color_str[2..4], 16),
            u8::from_str_radix(&color_str[4..6], 16),
        ) {
            return RGBColor(r, g, b);
        }
    }
    
    // Default to white if parsing fails
    RGBColor(255, 255, 255)
}

#[component]
pub fn MetricsChart(
    series: ReadSignal<Vec<MetricSeries>>,
    width: u32,
    height: u32,
) -> impl IntoView {
    let chart_ref = NodeRef::<html::Div>::new();

    Effect::new(move |_| {
        let metric_series = series.get();
        
        if let Some(element) = chart_ref.get() {
            let html_element: &HtmlElement = element.dyn_ref().unwrap();
            
            if metric_series.is_empty() {
                html_element.set_inner_html(&format!(
                    r#"<div style="width: {}px; height: {}px; background: #1e1e1e; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #999; font-size: 12px;">No data</div>"#,
                    width, height
                ));
                return;
            }

            // Find the maximum length of metrics arrays
            let max_length = metric_series.iter()
                .map(|s| s.metrics.len())
                .max()
                .unwrap_or(0);

            if max_length == 0 {
                html_element.set_inner_html(&format!(
                    r#"<div style="width: {}px; height: {}px; background: #1e1e1e; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #999; font-size: 12px;">No data</div>"#,
                    width, height
                ));
                return;
            }

            // Calculate the maximum value across all series for y-axis
            let max_value = metric_series.iter()
                .flat_map(|s| s.metrics.iter().copied())
                .fold(0.0f64, |acc, val| acc.max(val))
                .max(1.0f64) * 1.1; // Add 10% padding

            // X-axis range: 0 to max_length - 1
            let x_max = (max_length as f64 - 1.0).max(1.0);

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
                        0.0..x_max,
                        0.0..max_value,
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

                // Draw each metric series as a line
                for series_data in &metric_series {
                    if series_data.metrics.is_empty() {
                        continue;
                    }

                    let color = parse_color(&series_data.color);
                    
                    // Create points: (index, value)
                    let points: Vec<(f64, f64)> = series_data.metrics.iter()
                        .enumerate()
                        .map(|(i, &value)| (i as f64, value))
                        .collect();

                    // Draw the line
                    chart.draw_series(std::iter::once(PathElement::new(
                        points.clone(),
                        color,
                    ))).unwrap();

                    // Draw data points (circles) on the line
                    chart.draw_series(
                        points.iter().map(|&point| {
                            Circle::new(point, 3, color.filled())
                        })
                    ).unwrap();
                }
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
