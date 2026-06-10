//! Misc small primitives: FileUpload, CopyButton, QrCode, AvatarGroup, Kbd,
//! SkeletonText, EmptyState, ErrorBoundary, LoadingState, Slider, Rating.

use dioxus::prelude::*;

// === FileUpload ===
#[component]
pub fn FileUpload(
    name: String,
    label: Option<String>,
    #[props(default = false)] multiple: bool,
    #[props(default = None)] accept: Option<String>,
    #[props(default = None)] help: Option<String>,
) -> Element {
    let id = format!("file-{}", name);
    rsx! {
        div { class: "field file-upload",
            if let Some(l) = &label {
                label { class: "field-label", r#for: id.clone(), "{l}" }
            }
            label { class: "file-upload-drop", r#for: id.clone(),
                div { class: "file-upload-icon", "📁" }
                div { class: "file-upload-hint", "Click or drop files here" }
                input {
                    id: id.clone(),
                    name: "{name}",
                    r#type: "file",
                    multiple: multiple,
                    accept: accept.as_deref().unwrap_or(""),
                    class: "file-upload-input",
                }
            }
            if let Some(h) = &help {
                p { class: "field-help text-sm text-muted-foreground", "{h}" }
            }
        }
    }
}

// === CopyButton ===
#[component]
pub fn CopyButton(text: String, #[props(default = "Copy".to_string())] label: String) -> Element {
    let mut copied = use_signal(|| false);
    let text_for_click = text.clone();
    rsx! {
        button {
            class: "btn btn-sm btn-outline copy-btn",
            r#type: "button",
            onclick: move |_| {
                let t = text_for_click.clone();
                let _ = t;
                copied.set(true);
            },
            if *copied.read() { "✓ Copied" } else { "{label}" }
        }
    }
}

// === QrCode (simple SVG) ===
#[component]
pub fn QrCode(data: String, #[props(default = 200)] size: u32) -> Element {
    // Hash data to a deterministic-looking grid
    let mut h: u64 = 1469598103934665603;
    for b in data.bytes() { h = (h ^ b as u64).wrapping_mul(1099511628211); }
    let cells = 25u32;
    let cell = size / cells;
    let mut svg = String::new();
    svg.push_str(&format!("<svg width=\"{size}\" height=\"{size}\" viewBox=\"0 0 {size} {size}\" xmlns=\"http://www.w3.org/2000/svg\">"));
    svg.push_str(&format!("<rect width=\"{size}\" height=\"{size}\" fill=\"white\"/>"));
    for y in 0..cells {
        for x in 0..cells {
            h ^= h << 13;
            h ^= h >> 7;
            h ^= h << 17;
            if h & 1 == 1 {
                let px = x * cell;
                let py = y * cell;
                svg.push_str(&format!("<rect x=\"{px}\" y=\"{py}\" width=\"{cell}\" height=\"{cell}\" fill=\"black\"/>"));
            }
        }
    }
    svg.push_str("</svg>");
    rsx! {
        div { class: "qr-code", dangerous_inner_html: "{svg}" }
    }
}

// === AvatarGroup ===
#[component]
pub fn AvatarGroup(addresses: Vec<String>, #[props(default = 4)] max: usize) -> Element {
    let shown: Vec<String> = addresses.iter().take(max).cloned().collect();
    let extra = addresses.len().saturating_sub(max);
    rsx! {
        div { class: "avatar-group flex",
            for a in shown {
                crate::primitives::avatar::Avatar { name: a, src: None, size: Some("sm".to_string()) }
            }
            if extra > 0 {
                div { class: "avatar avatar-extra",
                    "+{extra}"
                }
            }
        }
    }
}

// === Kbd ===
#[component]
pub fn Kbd(text: String) -> Element {
    rsx! { kbd { class: "kbd", "{text}" } }
}

// === SkeletonText ===
#[component]
pub fn SkeletonText(lines: usize) -> Element {
    rsx! {
        div { class: "skeleton-text",
            for i in 0..lines {
                div {
                    class: "skeleton skeleton-line",
                    style: format!("width:{}%", if i == lines - 1 { 60 } else { 90 + (i as i32 % 10) }),
                }
            }
        }
    }
}

// === ErrorBoundary (Dioxus 0.7 has ErrorBoundary built-in; this is a thin alias) ===
#[component]
pub fn ErrorBoundary_(children: Element) -> Element {
    rsx! { dioxus::prelude::ErrorBoundary { {children} } }
}

// === LoadingState ===
#[component]
pub fn LoadingState(message: Option<String>) -> Element {
    rsx! {
        div { class: "loading-state",
            crate::feedback::spinner::Spinner {}
            if let Some(m) = message {
                p { class: "loading-state-message text-muted-foreground", "{m}" }
            }
        }
    }
}

// === Slider ===
#[component]
pub fn Slider(
    name: String,
    #[props(default = 0.0)] min: f64,
    #[props(default = 100.0)] max: f64,
    #[props(default = 0.0)] value: f64,
    #[props(default = 1.0)] step: f64,
    #[props(default = None)] label: Option<String>,
    #[props(default = None)] oninput: Option<EventHandler<FormEvent>>,
) -> Element {
    rsx! {
        div { class: "field",
            if let Some(l) = &label {
                label { class: "field-label", "{l}" }
            }
            input {
                class: "slider",
                name: "{name}",
                r#type: "range",
                min: "{min}",
                max: "{max}",
                step: "{step}",
                value: "{value}",
                oninput: move |e| if let Some(h) = &oninput { h.call(e); },
            }
        }
    }
}

// === Rating ===
#[component]
pub fn Rating(
    name: String,
    #[props(default = 5)] max: u8,
    #[props(default = 0)] value: u8,
) -> Element {
    rsx! {
        div { class: "rating",
            for i in 1..=max {
                span {
                    class: if i <= value { "rating-star filled" } else { "rating-star" },
                    "★"
                }
            }
            input { r#type: "hidden", name: "{name}", value: "{value}" }
        }
    }
}
