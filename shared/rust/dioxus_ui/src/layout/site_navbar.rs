//! Shared Home navigation. Apps supply destinations and their own session actions.
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct SiteNavItem {
    pub href: String,
    pub label: String,
    pub disabled: bool,
    pub icon: String,
}
impl SiteNavItem {
    pub fn new(href: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            label: label.into(),
            disabled: false,
            icon: String::new(),
        }
    }
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct SiteNavGroup {
    pub label: String,
    pub items: Vec<SiteNavItem>,
}
impl SiteNavGroup {
    pub fn new(label: impl Into<String>, items: Vec<SiteNavItem>) -> Self {
        Self {
            label: label.into(),
            items,
        }
    }
}

fn active(path: &str, href: &str) -> bool {
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let (href, expected) = href.split_once('?').unwrap_or((href, ""));
    let route_matches = path == href
        || (!matches!(href, "/" | "/account" | "/developer")
            && path.starts_with(&format!("{href}/")));
    route_matches
        && url::form_urlencoded::parse(expected.as_bytes())
            .all(|pair| url::form_urlencoded::parse(query.as_bytes()).any(|value| value == pair))
}

#[component]
pub fn SiteNavbar(
    groups: Vec<SiteNavGroup>,
    path: String,
    #[props(default = "/".into())] brand_href: String,
    #[props(default = "EPSX".into())] brand_label: String,
    #[props(default = "/public/logos/epsx-icon.svg".into())] logo_src: String,
    #[props(default)] on_navigate: Option<EventHandler<(MouseEvent, String)>>,
    actions: Element,
) -> Element {
    let mut menu = use_signal(|| None::<String>);
    let mut mobile = use_signal(|| false);
    let mut close = move |event: MouseEvent, href: String| {
        menu.set(None);
        mobile.set(false);
        if let Some(handler) = on_navigate {
            handler.call((event, href));
        }
    };
    rsx! {
        if try_consume_context::<crate::app::FrontendTailwindStyles>().is_none() {
            document::Style { {include_str!("site_navbar.css")} }
        }
        div { class: "epsx-site-nav", "data-site-navbar": "true",
            onkeydown: move |event| { if event.key() == Key::Escape { menu.set(None); mobile.set(false); } },
            if menu.read().is_some() {
                button { class: "site-nav-dismiss", aria_label: "Close navigation", tabindex: -1, onclick: move |_| menu.set(None) }
            }
            header { class: "site-nav-header",
                crate::navigation::AppLink { class: "site-nav-brand", href: brand_href.clone(), onclick: move |event| close(event, brand_href.clone()),
                    img { src: logo_src, alt: "", width: 30, height: 30 }
                    span { "{brand_label}" }
                }
                nav { class: "site-nav-desktop", aria_label: "Primary",
                    for group in groups.clone() {
                        { let label = group.label.clone(); let open = menu.read().as_ref() == Some(&label);
                            rsx! { details { class: "site-nav-group", "name": "epsx-primary-menu", open,
                                summary { aria_expanded: open, onclick: move |event| { event.prevent_default(); menu.set(if open { None } else { Some(label.clone()) }); },
                                    "{group.label}" crate::primitives::Icon { name: "chevron-down", size: 14 }
                                }
                                nav { aria_label: group.label,
                                    for item in group.items {
                                        if item.disabled { span { class: "site-nav-link", aria_disabled: "true", if !item.icon.is_empty() { crate::primitives::Icon { name: item.icon.clone(), size: 20 } } "{item.label}" } }
                                        else { crate::navigation::AppLink { class: "site-nav-link", href: item.href.clone(), aria_current: active(&path, &item.href).then_some("page"), onclick: move |event| close(event, item.href.clone()), if !item.icon.is_empty() { crate::primitives::Icon { name: item.icon.clone(), size: 20 } } "{item.label}" } }
                                    }
                                }
                            } }
                        }
                    }
                }
                div { class: "site-nav-actions", {actions} }
            }
            details { class: "site-nav-mobile", open: mobile(),
                summary { aria_expanded: mobile(), onclick: move |event| { event.prevent_default(); mobile.toggle(); }, "Menu" }
                nav { aria_label: "Mobile navigation",
                    for group in groups {
                        p { class: "site-nav-caption", "{group.label}" }
                        for item in group.items {
                            if item.disabled { span { class: "site-nav-link", aria_disabled: "true", if !item.icon.is_empty() { crate::primitives::Icon { name: item.icon.clone(), size: 20 } } "{item.label}" } }
                            else { crate::navigation::AppLink { class: "site-nav-link", href: item.href.clone(), aria_current: active(&path, &item.href).then_some("page"), onclick: move |event| close(event, item.href.clone()), if !item.icon.is_empty() { crate::primitives::Icon { name: item.icon.clone(), size: 20 } } "{item.label}" } }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn destinations_keep_query_and_disabled_state() {
        let html = dioxus_ssr::render_element(rsx! { SiteNavbar {
            groups: vec![SiteNavGroup::new("Payments", vec![SiteNavItem::new("/payments?tab=links", "Links"), SiteNavItem { href: "/locked".into(), label: "Locked".into(), disabled: true, icon: String::new() }])],
            path: "/payments?tab=links".to_string(), actions: rsx! { button { "Account" } },
        } });
        assert_eq!(html.matches("data-site-navbar=\"true\"").count(), 1);
        assert!(html.contains("href=\"/payments?tab=links\""));
        assert!(html.contains("aria-current=\"page\""));
        assert!(!html.contains("href=\"/locked\""));
        assert!(html.contains("Mobile navigation"));
    }
}
