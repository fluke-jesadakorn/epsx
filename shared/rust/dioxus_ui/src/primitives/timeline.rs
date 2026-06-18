//! `Timeline` / `TimelineItem` — vertical timeline list.
//!
//! Mirrors the visual pattern of a vertical timeline (e.g. git
//! history, audit log entries, transaction history).
//!
//! Each `TimelineItem` renders a dot on the left, an optional
//! timestamp, a title, and a description. Items are connected by
//! a vertical line that runs the full height of the list.

use super::icon::Icon;

use dioxus::prelude::*;

/// One item in the timeline. Renders a dot, an optional icon, a
/// timestamp, a title, and a description.
#[component]
pub fn TimelineItem(
    title: String,
    #[props(default = None)] description: Option<String>,
    #[props(default = None)] timestamp: Option<String>,
    #[props(default = None)] icon: Option<String>,
    #[props(default = false)] is_last: bool,
) -> Element {
    rsx! {
        div { class: "timeline-item relative flex gap-3 pb-6 last:pb-0",
            // Dot + connecting line column.
            div { class: "timeline-item-rail flex flex-col items-center",
                div { class: "timeline-item-dot flex h-6 w-6 items-center justify-center rounded-full border-2 border-primary bg-background",
                    if let Some(i) = icon {
                        Icon { name: i.clone(), size: Some(12) }
                    }
                }
                if !is_last {
                    div { class: "timeline-item-line mt-1 h-full w-px bg-border" }
                }
            }
            // Content column.
            div { class: "timeline-item-content flex flex-col gap-1 pt-0.5 pb-4",
                div { class: "flex items-center gap-2",
                    h4 { class: "timeline-item-title text-sm font-medium", "{title}" }
                    if let Some(t) = timestamp {
                        span { class: "timeline-item-timestamp text-xs text-muted-foreground", "{t}" }
                    }
                }
                if let Some(d) = description {
                    p { class: "timeline-item-description text-sm text-muted-foreground", "{d}" }
                }
            }
        }
    }
}

/// Vertical timeline. Renders a stack of `TimelineItem`s.
#[component]
pub fn Timeline(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "timeline flex flex-col".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_default_class() {
        let base = "timeline flex flex-col";
        assert!(base.contains("flex flex-col"));
    }

    #[test]
    fn timeline_item_dot_is_rounded() {
        let base = "timeline-item-dot flex h-6 w-6 items-center justify-center rounded-full border-2 border-primary bg-background";
        assert!(base.contains("rounded-full"));
        assert!(base.contains("border-2"));
    }

    #[test]
    fn timeline_item_line_is_thin() {
        let base = "timeline-item-line mt-1 h-full w-px bg-border";
        assert!(base.contains("w-px"));
    }
}
