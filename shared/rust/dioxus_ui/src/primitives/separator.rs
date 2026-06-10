use dioxus::prelude::*;

#[component]
pub fn Separator(
    orientation: Option<String>,
    class_name: Option<String>,
    /// Optional centered label rendered between two separator lines (e.g.
    /// "or" dividers in auth flows). Only applies to horizontal orientation.
    /// Vertical orientation ignores the label.
    label: Option<String>,
) -> Element {
    let o = orientation.unwrap_or_else(|| "horizontal".to_string());
    let mut cls = "separator".to_string();
    if o == "vertical" { cls.push_str(" separator-vertical"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }

    if let Some(lbl) = &label {
        if o == "horizontal" {
            return rsx! {
                div { class: "flex items-center gap-3 my-4",
                    div { class: "flex-1 h-px bg-border" }
                    span { class: "text-xs text-muted-foreground uppercase tracking-wider", "{lbl}" }
                    div { class: "flex-1 h-px bg-border" }
                }
            };
        }
    }

    rsx! { div { class: "{cls}", role: "separator", "aria-orientation": "{o}" } }
}
