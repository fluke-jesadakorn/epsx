use dioxus::prelude::*;

#[component]
pub fn Icon(name: String, size: Option<u32>, class_name: Option<String>) -> Element {
    let s = size.unwrap_or(20);
    let cls = class_name.unwrap_or_default();
    let sz_str = s.to_string();
    let svg = epsx_templates::lucide(&name, &sz_str, &cls);
    rsx! { span { class: "epsx-icon", dangerous_inner_html: "{svg}" } }
}
