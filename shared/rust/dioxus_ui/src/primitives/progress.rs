use dioxus::prelude::*;

#[component]
pub fn Progress(
    value: f32,
    max: Option<f32>,
    class_name: Option<String>,
    /// When true, render an animated indeterminate bar via the
    /// `progress-indeterminate` CSS class (the class itself lives in the
    /// design system; visually it pulses / slides from left to right).
    indeterminate: Option<bool>,
    /// Size variant: "sm" (2px tall), "md" (4px), "lg" (8px). Default md.
    size: Option<String>,
    /// Optional label shown above the bar.
    label: Option<String>,
) -> Element {
    let max = max.unwrap_or(100.0);
    let indeterminate = indeterminate.unwrap_or(false);
    let size = size.unwrap_or_else(|| "md".to_string());
    let pct = (value / max * 100.0).clamp(0.0, 100.0);
    let mut cls = "progress".to_string();
    if indeterminate {
        cls.push_str(" progress-indeterminate");
    }
    let height_cls = match size.as_str() {
        "sm" => "h-1",
        "lg" => "h-2",
        _ => "h-2.5",
    };
    cls.push(' ');
    cls.push_str(height_cls);
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }

    rsx! {
        div { class: "w-full",
            if let Some(lbl) = &label {
                div { class: "flex justify-between text-xs text-muted-foreground mb-1",
                    span { "{lbl}" }
                    if !indeterminate {
                        span { "{pct:.0}%" }
                    }
                }
            }
            div {
                class: "{cls}",
                role: "progressbar",
                "aria-valuenow": if indeterminate { "0" } else { "{value}" },
                "aria-valuemin": "0",
                "aria-valuemax": "{max}",
                if indeterminate {
                    div { class: "progress-bar progress-bar-indeterminate" }
                } else {
                    div { class: "progress-bar", style: "width:{pct}%" }
                }
            }
        }
    }
}
