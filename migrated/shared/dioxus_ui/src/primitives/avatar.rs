use dioxus::prelude::*;

#[component]
pub fn Avatar(name: String, src: Option<String>, size: Option<String>) -> Element {
    let s = size.clone().unwrap_or_else(|| "md".to_string());
    let cls = format!("avatar avatar-{}", s);
    let initials: String = name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    rsx! {
        div { class: "{cls}",
            if let Some(url) = src {
                img { src: "{url}", alt: "{name}" }
            } else {
                span { class: "avatar-fallback", "{initials}" }
            }
        }
    }
}
