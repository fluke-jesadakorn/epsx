use super::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardKind { Default, Glass, Insight, Flat, Primary, Stats }

impl CardKind {
    pub fn classes(&self) -> &'static str {
        match self {
            CardKind::Default => "card",
            CardKind::Glass => "card-glass",
            CardKind::Insight => "card-insight",
            CardKind::Flat => "",
            CardKind::Primary => "card-primary-solid",
            CardKind::Stats => "card-stats",
        }
    }
}

#[component]
pub fn Card(
    kind: Option<CardKind>,
    title: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    badge: Option<String>,
    hover: Option<bool>,
    class_name: Option<String>,
    footer: Option<String>,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let kind = kind.unwrap_or(CardKind::Glass);
    let hover = hover.unwrap_or(false);
    let mut cls = kind.classes().to_string();
    if hover { cls.push_str(" hover-scale"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }

    rsx! {
        div {
            class: "{cls}",
            onclick: move |e| if let Some(h) = &onclick { h.call(e); },
            if title.is_some() || icon.is_some() || badge.is_some() {
                div { class: "card-header",
                    if let Some(i) = &icon {
                        span { class: "card-icon", Icon { name: i.clone(), size: Some(20), class_name: Some("text-primary".to_string()) } }
                    }
                    div { class: "flex-1",
                        if let Some(t) = &title {
                            h3 { class: "card-title", "{t}" }
                        }
                        if let Some(d) = &description {
                            p { class: "card-description text-muted-foreground text-sm", "{d}" }
                        }
                    }
                    if let Some(b) = &badge {
                        span { class: "card-badge", "{b}" }
                    }
                }
            }
            div { class: "card-body", {children} }
            if let Some(f) = &footer {
                div { class: "card-footer", "{f}" }
            }
        }
    }
}

#[component]
pub fn CardHeader(children: Element) -> Element { rsx! { div { class: "card-header", {children} } } }
#[component]
pub fn CardBody(children: Element) -> Element { rsx! { div { class: "card-body", {children} } } }
#[component]
pub fn CardFooter(children: Element) -> Element { rsx! { div { class: "card-footer", {children} } } }
#[component]
pub fn CardTitle(children: Element) -> Element { rsx! { h3 { class: "card-title", {children} } } }
#[component]
pub fn CardDescription(children: Element) -> Element { rsx! { p { class: "card-description", {children} } } }

/// Horizontal divider rendered inside a card body. Useful for separating
/// sections of content without an extra margin. Mirrors the visual
/// "CardDivider" pattern in the shadcn admin shell.
#[component]
pub fn CardDivider(class_name: Option<String>) -> Element {
    let mut cls = "border-t border-border my-4".to_string();
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! { div { class: "{cls}", role: "separator", "aria-orientation": "horizontal" } }
}

/// Card rendered as an anchor — wraps the whole card in `<a href>` so the
/// entire surface is clickable. `kind` and visual props are inherited from
/// `Card` semantics. Set `target` to open in a new tab (`_blank` will
/// automatically set `rel="noopener noreferrer"`).
#[component]
pub fn CardLink(
    href: String,
    kind: Option<CardKind>,
    title: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    badge: Option<String>,
    hover: Option<bool>,
    class_name: Option<String>,
    target: Option<String>,
    rel: Option<String>,
    children: Element,
) -> Element {
    let kind = kind.unwrap_or(CardKind::Glass);
    let hover = hover.unwrap_or(false);
    let mut cls = kind.classes().to_string();
    if hover { cls.push_str(" hover-scale"); }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    let rel = rel.unwrap_or_else(|| {
        if target.as_deref() == Some("_blank") { "noopener noreferrer".to_string() } else { String::new() }
    });
    let target_attr = target.unwrap_or_default();

    rsx! {
        a {
            class: "block {cls}",
            href: "{href}",
            target: "{target_attr}",
            rel: "{rel}",
            if title.is_some() || icon.is_some() || badge.is_some() {
                div { class: "card-header",
                    if let Some(i) = &icon {
                        span { class: "card-icon", Icon { name: i.clone(), size: Some(20), class_name: Some("text-primary".to_string()) } }
                    }
                    div { class: "flex-1",
                        if let Some(t) = &title {
                            h3 { class: "card-title", "{t}" }
                        }
                        if let Some(d) = &description {
                            p { class: "card-description text-muted-foreground text-sm", "{d}" }
                        }
                    }
                    if let Some(b) = &badge {
                        span { class: "card-badge", "{b}" }
                    }
                }
            }
            div { class: "card-body", {children} }
        }
    }
}
