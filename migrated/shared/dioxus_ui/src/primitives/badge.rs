use super::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeKind { Default, Primary, Success, Warning, Danger, Info, Brand, Cool, Warm, Purple, Outline, Secondary, Destructive }

impl BadgeKind {
    pub fn classes(&self) -> &'static str {
        match self {
            BadgeKind::Default => "badge",
            BadgeKind::Primary => "badge badge-primary",
            BadgeKind::Success => "badge badge-success",
            BadgeKind::Warning => "badge badge-warn",
            BadgeKind::Danger => "badge badge-danger",
            BadgeKind::Info => "badge badge-info",
            BadgeKind::Brand => "badge badge-brand",
            BadgeKind::Cool => "badge badge-cool",
            BadgeKind::Warm => "badge badge-warm",
            BadgeKind::Purple => "badge badge-purple",
            BadgeKind::Outline => "badge badge-outline",
            BadgeKind::Secondary => "badge badge-outline",
            BadgeKind::Destructive => "badge badge-danger",
        }
    }
}

#[component]
pub fn Badge(
    kind: Option<BadgeKind>,
    icon: Option<String>,
    pill: Option<bool>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let kind = kind.unwrap_or(BadgeKind::Default);
    let pill = pill.unwrap_or(false);
    let mut cls = kind.classes().to_string();
    if pill { cls.push_str(" rounded-full"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! {
        span { class: "{cls}",
            if let Some(i) = &icon {
                span { class: "badge-icon", Icon { name: i.clone(), size: Some(12) } }
            }
            {children}
        }
    }
}
