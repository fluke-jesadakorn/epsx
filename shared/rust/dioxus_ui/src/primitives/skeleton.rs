use dioxus::prelude::*;

#[component]
pub fn Skeleton(
    width: Option<String>,
    height: Option<String>,
    rounded: Option<bool>,
    class_name: Option<String>,
) -> Element {
    let mut style = String::new();
    if let Some(w) = width { style.push_str(&format!("width:{};", w)); }
    if let Some(h) = height { style.push_str(&format!("height:{};", h)); }
    let mut cls = "skeleton".to_string();
    if rounded.unwrap_or(false) { cls.push_str(" rounded-full"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! { div { class: "{cls}", style: "{style}" } }
}

#[component]
pub fn SkeletonGroup(count: Option<usize>, gap: Option<String>) -> Element {
    let n = count.unwrap_or(3);
    let g = gap.unwrap_or_else(|| "0.5rem".to_string());
    rsx! {
        div { class: "flex flex-col", style: "gap:{g}",
            for _ in 0..n {
                Skeleton { height: Some("1.25rem".to_string()) }
            }
        }
    }
}

/// Circular skeleton placeholder. Useful for avatar / icon loading states.
/// `size` is a CSS length (e.g. "2.5rem", "40px"). Defaults to 2.5rem.
#[component]
pub fn SkeletonCircle(size: Option<String>, class_name: Option<String>) -> Element {
    let s = size.unwrap_or_else(|| "2.5rem".to_string());
    let mut cls = "skeleton rounded-full".to_string();
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! { div { class: "{cls}", style: "width:{s};height:{s};" } }
}

/// Rectangular skeleton placeholder with explicit width/height (CSS
/// lengths). Useful for image or hero block loading states where a
/// plain `Skeleton` would leave the dimensions to the parent.
#[component]
pub fn SkeletonBlock(
    width: String,
    height: String,
    rounded: Option<bool>,
    class_name: Option<String>,
) -> Element {
    let mut cls = "skeleton".to_string();
    if rounded.unwrap_or(false) { cls.push_str(" rounded-md"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! { div { class: "{cls}", style: "width:{width};height:{height};" } }
}
