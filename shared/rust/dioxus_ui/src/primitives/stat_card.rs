use super::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn StatCard(
    label: String,
    value: String,
    change: Option<String>,
    change_kind: Option<crate::BadgeKind>,
    icon: Option<String>,
    href: Option<String>,
    /// Optional explicit trend direction. When set, takes precedence over
    /// `change_kind` for coloring the change text:
    /// - "up"   → green
    /// - "down" → red
    /// - "flat" → gray/muted
    /// When `None`, the parent's `change_kind` badge (or muted text) is
    /// used unchanged.
    trend_direction: Option<String>,
    /// Optional inline chart element (e.g. a small `ChartLine` or
    /// `ChartArea`) rendered in the lower-right of the card.
    sparkline: Option<Element>,
) -> Element {
    // Derive a Tailwind text class from `trend_direction` for the change.
    let trend_text_cls = match trend_direction.as_deref() {
        Some("up") => "text-emerald-400",
        Some("down") => "text-red-400",
        Some("flat") => "text-muted-foreground",
        _ => "",
    };

    let inner = rsx! {
        div { class: "card card-stats hover-scale",
            div { class: "flex items-start justify-between",
                div { class: "flex-1",
                    p { class: "stat-label text-sm text-muted-foreground", "{label}" }
                    p { class: "stat-value text-2xl font-semibold mt-1", "{value}" }
                    if let Some(c) = &change {
                        if !trend_text_cls.is_empty() {
                            p { class: "stat-change text-sm mt-1 {trend_text_cls}", "{c}" }
                        } else if let Some(k) = change_kind {
                            crate::Badge { kind: k, "{c}" }
                        } else {
                            span { class: "text-sm text-muted-foreground", "{c}" }
                        }
                    }
                }
                if let Some(i) = &icon {
                    div { class: "stat-icon", Icon { name: i.clone(), size: Some(24), class_name: Some("text-primary".to_string()) } }
                }
            }
            if let Some(s) = &sparkline {
                div { class: "stat-sparkline mt-2", {s} }
            }
        }
    };
    if let Some(h) = href {
        rsx! { a { class: "block", href: "{h}", {inner} } }
    } else {
        inner
    }
}
