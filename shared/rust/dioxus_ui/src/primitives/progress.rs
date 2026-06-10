use dioxus::prelude::*;

#[component]
pub fn Progress(value: f32, max: Option<f32>, class_name: Option<String>) -> Element {
    let max = max.unwrap_or(100.0);
    let pct = (value / max * 100.0).clamp(0.0, 100.0);
    let mut cls = "progress".to_string();
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! {
        div { class: "{cls}", role: "progressbar", "aria-valuenow": "{value}", "aria-valuemax": "{max}",
            div { class: "progress-bar", style: "width:{pct}%" }
        }
    }
}
