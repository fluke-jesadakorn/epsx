use dioxus::prelude::*;

#[component]
pub fn Separator(orientation: Option<String>, class_name: Option<String>) -> Element {
    let o = orientation.unwrap_or_else(|| "horizontal".to_string());
    let mut cls = "separator".to_string();
    if o == "vertical" { cls.push_str(" separator-vertical"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! { div { class: "{cls}", role: "separator" } }
}
