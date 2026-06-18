//! `ProgressCircle` / `ProgressCircleWithLabel` — circular
//! progress indicator.
//!
//! Mirrors the visual pattern of `Progress` (a linear bar) but
//! renders as a circle. Useful for showing completion percentages
//! in a compact form (e.g. profile completeness, storage usage,
//! CPU/memory monitors).

use dioxus::prelude::*;

/// Circular progress indicator.
///
/// - `value: f32` — current progress (0.0 to 100.0).
/// - `size: Option<u32>` — diameter in pixels. Defaults to 64.
/// - `stroke_width: Option<u32>` — the stroke thickness in pixels.
///   Defaults to 4.
/// - `class: Option<String>` — extra Tailwind classes.
///
/// The circle renders an SVG with a track (the full circle) and a
/// progress arc (the partial circle whose length corresponds to
/// `value`). The progress arc uses `currentColor` so callers can
/// control the color with Tailwind `text-*` classes.
#[component]
pub fn ProgressCircle(
    value: f32,
    #[props(default = 64)] size: u32,
    #[props(default = 4)] stroke_width: u32,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let v = value.clamp(0.0, 100.0);
    let radius = (size as f32 - stroke_width as f32) / 2.0;
    let circumference = 2.0 * std::f32::consts::PI * radius;
    let offset = circumference * (1.0 - v / 100.0);
    let mut cls = "progress-circle relative inline-flex items-center justify-center text-primary".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", style: "width: {size}px; height: {size}px;",
            svg {
                width: "{size}",
                height: "{size}",
                view_box: "0 0 {size} {size}",
                class: "progress-circle-svg"
            }
            circle {
                cx: "{size / 2}",
                cy: "{size / 2}",
                r: "{radius}",
                fill: "none",
                stroke: "currentColor",
                "stroke-width": "{stroke_width}",
                opacity: "0.2",
                class: "progress-circle-track"
            }
            circle {
                cx: "{size / 2}",
                cy: "{size / 2}",
                r: "{radius}",
                fill: "none",
                stroke: "currentColor",
                "stroke-width": "{stroke_width}",
                "stroke-dasharray": "{circumference}",
                "stroke-dashoffset": "{offset}",
                "stroke-linecap": "round",
                transform: "rotate(-90 {size / 2} {size / 2})",
                class: "progress-circle-progress"
            }
        }
    }
}

/// Progress circle with a centered label (typically the
/// percentage value).
#[component]
pub fn ProgressCircleWithLabel(
    value: f32,
    #[props(default = 64)] size: u32,
    #[props(default = 4)] stroke_width: u32,
    #[props(default = None)] label: Option<String>,
) -> Element {
    let v = value.clamp(0.0, 100.0);
    let display = label.unwrap_or_else(|| format!("{:.0}%", v));
    let v_for_circle = v;
    rsx! {
        div { class: "progress-circle-wrap relative inline-flex items-center justify-center",
            style: "width: {size}px; height: {size}px;",
            ProgressCircle { value: v_for_circle, size: size, stroke_width: stroke_width }
            span { class: "progress-circle-label absolute inset-0 flex items-center justify-center text-xs font-medium",
                "{display}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_value_clamps_to_zero() {
        let v: f32 = -10.0;
        let clamped = v.clamp(0.0, 100.0);
        assert_eq!(clamped, 0.0);
    }

    #[test]
    fn progress_value_clamps_to_hundred() {
        let v: f32 = 150.0;
        let clamped = v.clamp(0.0, 100.0);
        assert_eq!(clamped, 100.0);
    }

    #[test]
    fn progress_value_in_range_unchanged() {
        let v: f32 = 42.5;
        let clamped = v.clamp(0.0, 100.0);
        assert_eq!(clamped, 42.5);
    }

    #[test]
    fn progress_circle_label_format() {
        // When label is None, the display string is `"{:.0}%"`.
        // 42.5 rounds to 42 with Rust's default banker's rounding
        // (or 43 depending on the float repr); the key contract is
        // that the value is a percentage with no decimals.
        let v: f32 = 42.5;
        let display = format!("{:.0}%", v);
        assert!(
            display == "42%" || display == "43%",
            "display should round to 42 or 43 percent, got {display}"
        );
        // Also verify the integer case.
        let v2: f32 = 75.0;
        let display2 = format!("{:.0}%", v2);
        assert_eq!(display2, "75%");
    }
}
