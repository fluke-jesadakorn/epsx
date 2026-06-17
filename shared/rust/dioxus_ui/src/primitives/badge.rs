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
    /// When true, render a small colored dot before the label.
    /// The dot color follows the badge kind (success/green, danger/red,
    /// warn/yellow, info/blue, etc).
    dot: Option<bool>,
    /// When true, add the `truncate` utility so long labels ellipsize.
    /// Useful in tight table cells or card headers.
    truncate: Option<bool>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let kind = kind.unwrap_or(BadgeKind::Default);
    let pill = pill.unwrap_or(false);
    let dot = dot.unwrap_or(false);
    let truncate = truncate.unwrap_or(false);
    let mut cls = kind.classes().to_string();
    if pill { cls.push_str(" rounded-full"); }
    if truncate { cls.push_str(" truncate max-w-[12rem]"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }

    let dot_cls = match kind {
        BadgeKind::Success => "bg-emerald-400",
        BadgeKind::Danger | BadgeKind::Destructive => "bg-red-400",
        BadgeKind::Warning => "bg-amber-400",
        BadgeKind::Info => "bg-blue-400",
        BadgeKind::Brand | BadgeKind::Primary => "bg-purple-400",
        BadgeKind::Cool => "bg-cyan-400",
        BadgeKind::Warm => "bg-orange-400",
        _ => "bg-gray-400",
    };

    rsx! {
        span { class: "{cls}",
            if dot {
                span {
                    class: "inline-block w-1.5 h-1.5 rounded-full mr-1 {dot_cls}",
                    "aria-hidden": "true",
                }
            }
            if let Some(i) = &icon {
                span { class: "badge-icon", Icon { name: i.clone(), size: Some(12) } }
            }
            {children}
        }
    }
}
