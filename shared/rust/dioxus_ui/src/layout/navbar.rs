use crate::primitives::icon::Icon;

use dioxus::prelude::*;
use crate::i18n::t;
use crate::auth::User;

#[derive(Clone, Debug, PartialEq)]
pub struct NavItem { pub label: String, pub href: String, pub icon: Option<String>, pub external: bool }

#[derive(Clone, Debug, PartialEq)]
pub struct NavGroup { pub label: String, pub items: Vec<NavItem> }

pub fn main_nav_groups(is_authed: bool) -> Vec<NavGroup> {
    let mut groups = vec![
        NavGroup {
            label: "Platform".to_string(),
            items: vec![
                NavItem { label: t("nav.home").to_string(), href: "/".to_string(), icon: Some("home".to_string()), external: false },
                NavItem { label: t("nav.pricing").to_string(), href: "/pricing".to_string(), icon: Some("credit-card".to_string()), external: false },
                NavItem { label: t("nav.plans").to_string(), href: "/plans".to_string(), icon: Some("layout-dashboard".to_string()), external: false },
                NavItem { label: t("nav.analytics").to_string(), href: "/analytics".to_string(), icon: Some("chart-line".to_string()), external: false },
                NavItem { label: t("nav.portfolio").to_string(), href: "/portfolio".to_string(), icon: Some("briefcase".to_string()), external: false },
            ],
        },
        NavGroup {
            label: "Resources".to_string(),
            items: vec![
                NavItem { label: t("nav.news").to_string(), href: "/news".to_string(), icon: Some("newspaper".to_string()), external: false },
                NavItem { label: t("nav.manual").to_string(), href: "/manual".to_string(), icon: Some("book".to_string()), external: false },
                NavItem { label: t("nav.about").to_string(), href: "/about".to_string(), icon: Some("info".to_string()), external: false },
                NavItem { label: t("nav.contact").to_string(), href: "/contact".to_string(), icon: Some("mail".to_string()), external: false },
            ],
        },
    ];
    if is_authed {
        groups.push(NavGroup {
            label: "Account".to_string(),
            items: vec![
                NavItem { label: t("nav.dashboard").to_string(), href: "/dashboard".to_string(), icon: Some("layout-dashboard".to_string()), external: false },
                NavItem { label: t("nav.profile").to_string(), href: "/profile".to_string(), icon: Some("user".to_string()), external: false },
                NavItem { label: t("nav.notifications").to_string(), href: "/notifications".to_string(), icon: Some("bell".to_string()), external: false },
                NavItem { label: t("nav.chat").to_string(), href: "/chat".to_string(), icon: Some("message-circle".to_string()), external: false },
                NavItem { label: t("nav.developer").to_string(), href: "/developer".to_string(), icon: Some("code".to_string()), external: false },
            ],
        });
    }
    groups
}

#[component]
pub fn Navbar(user: Option<User>, current_path: Option<String>, on_theme_toggle: Option<EventHandler<MouseEvent>>) -> Element {
    let is_authed = user.is_some();
    let path = current_path.unwrap_or_else(|| "/".to_string());
    let groups = main_nav_groups(is_authed);
    let logo_svg = epsx_templates::epsx_icon_svg().to_string();
    let sun_svg = epsx_templates::lucide("sun", "18", "").to_string();
    let moon_svg = epsx_templates::lucide("moon", "18", "").to_string();
    let chev_svg = epsx_templates::lucide("chevron-down", "14", "").to_string();
    rsx! {
        nav { class: "navbar",
            div { class: "navbar-inner",
                a { class: "navbar-brand", href: "/",
                    span { class: "navbar-logo", dangerous_inner_html: "{logo_svg}" }
                    span { class: "navbar-title gradient-text", "EPSX" }
                }
                div { class: "navbar-menu",
                    for group in groups {
                        div { class: "nav-dropdown",
                            button { class: "nav-dropdown-trigger",
                                span { "{group.label}" }
                                span { class: "chev", dangerous_inner_html: "{chev_svg}" }
                            }
                            div { class: "nav-dropdown-menu",
                                for item in &group.items {
                                    a {
                                        class: if path == item.href { "nav-dropdown-item active" } else { "nav-dropdown-item" },
                                        href: "{item.href}",
                                        if let Some(i) = &item.icon {
                                            span { dangerous_inner_html: "{epsx_templates::lucide(i, \"16\", \"\")}" }
                                        }
                                        span { "{item.label}" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "navbar-actions",
                    button { class: "theme-toggle", onclick: move |e| if let Some(h) = &on_theme_toggle { h.call(e); },
                        span { class: "theme-icon-sun", dangerous_inner_html: "{sun_svg}" }
                        span { class: "theme-icon-moon", dangerous_inner_html: "{moon_svg}" }
                    }
                    if let Some(u) = &user {
                        a { class: "btn btn-primary", href: "/dashboard",
                            span { dangerous_inner_html: "{epsx_templates::lucide(\"wallet\", \"16\", \"\")}" }
                            span { "{u.short_address()}" }
                        }
                    } else {
                        a { class: "btn btn-gradient", href: "/auth",
                            span { "{t(\"nav.connect\")}" }
                        }
                    }
                }
            }
        }
    }
}
