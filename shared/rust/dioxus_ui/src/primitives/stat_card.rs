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
) -> Element {
    let inner = rsx! {
        div { class: "card card-stats hover-scale",
            div { class: "flex items-start justify-between",
                div { class: "flex-1",
                    p { class: "stat-label text-sm text-muted-foreground", "{label}" }
                    p { class: "stat-value text-2xl font-semibold mt-1", "{value}" }
                    if let Some(c) = &change {
                        if let Some(k) = change_kind {
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
        }
    };
    if let Some(h) = href {
        rsx! { a { class: "block", href: "{h}", {inner} } }
    } else {
        inner
    }
}
