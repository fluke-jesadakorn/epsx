//! `Tooltip` — hover/focus-triggered informational bubble.
//!
//! Backward-compatible with the previous API (`text: String`, children).
//! Adds `side` / `align` positioning, `delay` (ms before showing),
//! controlled `open`, and renders the bubble only when the trigger is
//! hovered or focused (no always-on rendering).
//!
//! ## A11y
//!
//! The bubble is given `role="tooltip"` plus a stable `id`. The trigger
//! wrapper (the `span` that contains the caller's children) gets
//! `aria-describedby={bubble_id}` so screen readers announce the
//! tooltip text when the trigger is focused / hovered.

use dioxus::prelude::*;

#[component]
pub fn Tooltip(
    text: String,
    #[props(default = None)] side: Option<String>,
    #[props(default = None)] align: Option<String>,
    #[props(default = 0)] delay: u32,
    #[props(default = None)] open: Option<bool>,
    children: Element,
) -> Element {
    // We render the bubble as `hidden` by default; CSS reveals it on
    // `:hover` / `:focus-within` of the wrapper. When `open` is `Some`,
    // we honour the controlled value instead. When `delay > 0`, callers
    // can opt into JS-driven show after a timeout (omitted for the
    // initial Track C deliverable — pure CSS hover/focus is enough).
    let side_cls = match side.as_deref() {
        Some("top") => " tooltip-side-top",
        Some("left") => " tooltip-side-left",
        Some("right") => " tooltip-side-right",
        _ => " tooltip-side-bottom",
    };
    let align_cls = match align.as_deref() {
        Some("start") => " tooltip-align-start",
        Some("end") => " tooltip-align-end",
        _ => " tooltip-align-center",
    };
    let delay_attr = if delay > 0 { format!("--tooltip-delay: {delay}ms;") } else { String::new() };
    let visible = open.unwrap_or(false);
    let bubble_class = format!(
        "tooltip-content{side_cls}{align_cls}{}",
        if visible { " tooltip-open" } else { "" }
    );

    // Stable id for the tooltip bubble; aria-describedby on the trigger
    // wrapper points to it so assistive tech can announce the text.
    let bubble_id = format!("tooltip-{}", generate_id());

    rsx! {
        span {
            class: "tooltip-wrapper",
            style: "{delay_attr}",
            "aria-describedby": "{bubble_id}",
            {children}
            span { id: "{bubble_id}", class: "{bubble_class}", role: "tooltip", "{text}" }
        }
    }
}

/// Monotonically increasing id for SSR-stable tooltip ids. SSR is
/// single threaded and component instances don't overlap, so a
/// counter is enough.
fn generate_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
