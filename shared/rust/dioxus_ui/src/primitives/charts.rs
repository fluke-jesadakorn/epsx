//! SVG-based charts. No JS chart lib; pure rsx.
//!
//! - `ChartLine` — line chart with one or more series
//! - `ChartBar`  — bar chart (vertical or horizontal)
//! - `ChartDonut`— pie/donut with optional legend
//!
//! The chart is server-renderable: takes a `Vec<Series>` and emits SVG.

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Series {
    pub name: String,
    pub color: String,
    pub points: Vec<DataPoint>,
}

#[component]
pub fn ChartLine(
    series: Vec<Series>,
    #[props(default = 480)] width: i32,
    #[props(default = 220)] height: i32,
    #[props(default = 32)] padding: i32,
    #[props(default = None)] y_label: Option<String>,
    #[props(default = true)] show_grid: bool,
    #[props(default = true)] show_dots: bool,
) -> Element {
    if series.is_empty() || series[0].points.is_empty() {
        return rsx! { div { class: "chart-empty", "No data" } };
    }
    let all_y: Vec<f64> = series.iter().flat_map(|s| s.points.iter().map(|p| p.y)).collect();
    let all_x: Vec<f64> = series.iter().flat_map(|s| s.points.iter().map(|p| p.x)).collect();
    let (min_x, max_x) = (*all_x.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(), *all_x.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
    let (min_y, max_y) = (*all_y.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(), *all_y.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
    let y_range = (max_y - min_y).max(0.0001);
    let x_range = (max_x - min_x).max(0.0001);
    let w = (width - padding * 2) as f64;
    let h = (height - padding * 2) as f64;

    let project = |p: &DataPoint| -> (f64, f64) {
        let x = padding as f64 + (p.x - min_x) / x_range * w;
        let y = padding as f64 + h - (p.y - min_y) / y_range * h;
        (x, y)
    };

    let paths: Vec<String> = series.iter().map(|s| {
        let mut d = String::new();
        for (i, p) in s.points.iter().enumerate() {
            let (x, y) = project(p);
            if i == 0 { d.push_str(&format!("M{:.1},{:.1}", x, y)); }
            else { d.push_str(&format!(" L{:.1},{:.1}", x, y)); }
        }
        d
    }).collect();

    let grid_lines: Vec<f64> = (0..=4).map(|i| padding as f64 + i as f64 * h / 4.0).collect();

    rsx! {
        div { class: "chart chart-line",
            svg { width: "{width}", height: "{height}", view_box: "0 0 {width} {height}", xmlns: "http://www.w3.org/2000/svg",
                if show_grid {
                    for y in &grid_lines {
                        line { x1: "{padding}", y1: "{y}", x2: "{width}", y2: "{y}", stroke: "rgba(255,255,255,0.06)", "stroke-width": "1" }
                    }
                }
                for (i, s) in series.iter().enumerate() {
                    path { d: "{paths[i]}", fill: "none", stroke: "{s.color}", "stroke-width": "2", "stroke-linejoin": "round", "stroke-linecap": "round" }
                }
                if show_dots {
                    for s in &series {
                        for p in &s.points {
                            {
                                let (x, y) = project(p);
                                rsx! { circle { cx: "{x}", cy: "{y}", r: "3", fill: "{s.color}" } }
                            }
                        }
                    }
                }
            }
            if !series.is_empty() {
                div { class: "chart-legend",
                    for s in &series {
                        span { class: "chart-legend-item",
                            span { class: "chart-legend-swatch", style: "background:{s.color}" }
                            "{s.name}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ChartBar(
    data: Vec<(String, f64)>,
    #[props(default = 480)] width: i32,
    #[props(default = 220)] height: i32,
    #[props(default = 32)] padding: i32,
    #[props(default = "#22d3ee".to_string())] color: String,
    #[props(default = false)] horizontal: bool,
    #[props(default = None)] y_label: Option<String>,
) -> Element {
    if data.is_empty() {
        return rsx! { div { class: "chart-empty", "No data" } };
    }
    let max_v = data.iter().map(|x| x.1).fold(f64::MIN, f64::max).max(0.0001);
    let n = data.len();
    let w = (width as f64) - (padding as f64) * 2.0;
    let h = (height as f64) - (padding as f64) * 2.0;
    let bar_w = if horizontal { h / n as f64 * 0.7 } else { w / n as f64 * 0.7 };
    let gap = if horizontal { h / n as f64 * 0.3 } else { w / n as f64 * 0.3 };

    rsx! {
        div { class: "chart chart-bar",
            svg { width: "{width}", height: "{height}", view_box: "0 0 {width} {height}", xmlns: "http://www.w3.org/2000/svg",
                for (i, (lbl, v)) in data.iter().enumerate() {
                    {
                        let pct = v / max_v;
                        if horizontal {
                            let bar_len = pct * w;
                            let y = padding as f64 + i as f64 * (bar_w + gap);
                            rsx! {
                                rect { x: "{padding}", y: "{y}", width: "{bar_len}", height: "{bar_w}", fill: "{color}", rx: "2" }
                                text { x: "{padding}", y: "{y + bar_w/2.0}", fill: "currentColor", "font-size": "11", "dominant-baseline": "middle", "{lbl}: {v}" }
                            }
                        } else {
                            let bar_h = pct * h;
                            let x = padding as f64 + i as f64 * (bar_w + gap);
                            let y = padding as f64 + (h - bar_h);
                            rsx! {
                                rect { x: "{x}", y: "{y}", width: "{bar_w}", height: "{bar_h}", fill: "{color}", rx: "2" }
                                text { x: "{x + bar_w/2.0}", y: "{padding as f64 + h + 12.0}", fill: "currentColor", "font-size": "10", "text-anchor": "middle", "{lbl}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ChartDonut(
    data: Vec<(String, f64, String)>,
    #[props(default = 200)] size: i32,
    #[props(default = 32)] thickness: i32,
) -> Element {
    let total: f64 = data.iter().map(|x| x.1).sum();
    if total <= 0.0 {
        return rsx! { div { class: "chart-empty", "No data" } };
    }
    let cx = size as f64 / 2.0;
    let cy = size as f64 / 2.0;
    let r = size as f64 / 2.0 - 4.0;
    let inner = r - thickness as f64;

    let mut acc = -std::f64::consts::FRAC_PI_2;
    let mut slices: Vec<(f64, f64, String, f64, String)> = Vec::new();
    for (lbl, v, color) in &data {
        let frac = v / total;
        let start = acc;
        let end = acc + frac * std::f64::consts::TAU;
        slices.push((start, end, color.clone(), frac, lbl.clone()));
        acc = end;
    }

    rsx! {
        div { class: "chart chart-donut flex items-center gap-6",
            svg { width: "{size}", height: "{size}", view_box: "0 0 {size} {size}", xmlns: "http://www.w3.org/2000/svg",
                for (s, e, color, _, _) in &slices {
                    {
                        let x1 = cx + r * s.cos();
                        let y1 = cy + r * s.sin();
                        let x2 = cx + r * e.cos();
                        let y2 = cy + r * e.sin();
                        let xi2 = cx + inner * e.cos();
                        let yi2 = cy + inner * e.sin();
                        let xi1 = cx + inner * s.cos();
                        let yi1 = cy + inner * s.sin();
                        let large = if e - s > std::f64::consts::PI { 1 } else { 0 };
                        let d = format!("M{x1},{y1} A{r},{r} 0 {large} 1 {x2},{y2} L{xi2},{yi2} A{inner},{inner} 0 {large} 0 {xi1},{yi1} Z");
                        rsx! { path { d: "{d}", fill: "{color}" } }
                    }
                }
                text { x: "{cx}", y: "{cy}", "text-anchor": "middle", "dominant-baseline": "middle", fill: "currentColor", "font-size": "18", "font-weight": "600", "{total:.0}" }
            }
            div { class: "chart-legend",
                for (_, _, color, frac, lbl) in &slices {
                    span { class: "chart-legend-item",
                        span { class: "chart-legend-swatch", style: "background:{color}" }
                        "{lbl} ({(frac * 100.0):.1}%)"
                    }
                }
            }
        }
    }
}
