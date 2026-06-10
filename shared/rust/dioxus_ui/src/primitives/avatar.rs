use dioxus::prelude::*;

#[component]
pub fn Avatar(
    name: String,
    src: Option<String>,
    size: Option<String>,
    /// Optional status indicator. One of: "online" | "offline" | "away" | "busy".
    /// Renders a small colored dot at the bottom-right of the avatar.
    /// The dot has a `role="img"` and `aria-label` reflecting the status.
    status: Option<String>,
) -> Element {
    let s = size.clone().unwrap_or_else(|| "md".to_string());
    let cls = format!("avatar avatar-{}", s);
    let initials: String = name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();

    let (status_dot, status_aria) = match status.as_deref() {
        Some("online") => (Some("bg-emerald-400"), "online"),
        Some("offline") => (Some("bg-gray-400"), "offline"),
        Some("away") => (Some("bg-amber-400"), "away"),
        Some("busy") => (Some("bg-red-400"), "busy"),
        _ => (None, ""),
    };

    rsx! {
        div { class: "{cls} relative",
            if let Some(url) = src {
                img { src: "{url}", alt: "{name}" }
            } else {
                span { class: "avatar-fallback", "{initials}" }
            }
            if let Some(dot_cls) = status_dot {
                span {
                    class: "absolute bottom-0 right-0 w-2.5 h-2.5 rounded-full ring-2 ring-background {dot_cls}",
                    "aria-label": "{status_aria}",
                    role: "img",
                }
            }
        }
    }
}
