//! Nav config — data-only port of `apps-old/frontend/components/nav/nav-config.ts`.
//!
//! Pure data, no Dioxus runtime needed. Consumed by `desktop-nav`, `mobile-nav`,
//! `footer`, etc.
//!
//! Mirrors the TS source 1:1:
//! - `NavItem` / `NavGroup` structs (icon is a lucide *name* string — same
//!   `epsx_templates::lucide` registry the rest of the UI uses).
//! - `NAV_GROUPS` constant (Market / Developer / Company).
//! - `FOOTER_LINKS` constant (Terms / Privacy / Contact).
//! - `is_group_active` / `is_item_active` helpers.
//!
//! The constants are `LazyLock` because the `NavGroup` items slice is
//! `Vec<NavItem>` (and `Vec` is not allowed in `const`). `LazyLock` is
//! stable in Rust 1.80+; the workspace is on 1.91.

use std::sync::LazyLock;

/// One entry in a nav group or footer.
#[derive(Clone, Debug, PartialEq)]
pub struct NavItem {
    pub label: String,
    pub href: String,
    /// Stable per-item key (used for list keys and to disambiguate items
    /// that share a label).
    pub key: String,
    /// Lucide icon name (matches `epsx_templates::lucide`).
    pub icon: Option<String>,
    /// Optional one-line description (shown in desktop dropdowns).
    pub desc: Option<String>,
}

/// A labeled group of nav items (e.g. "Market", "Developer").
#[derive(Clone, Debug, PartialEq)]
pub struct NavGroup {
    pub label: String,
    pub key: String,
    pub icon: Option<String>,
    pub items: Vec<NavItem>,
}

/// Top-level nav groups, in render order. Matches the original
/// `NAV_GROUPS` in `apps-old/frontend/components/nav/nav-config.ts`.
pub static NAV_GROUPS: LazyLock<Vec<NavGroup>> = LazyLock::new(|| {
    vec![
        NavGroup {
            label: "Market".to_string(),
            key: "market".to_string(),
            icon: Some("chart-column".to_string()),
            items: vec![
                NavItem {
                    label: "Rankings".to_string(),
                    href: "/analytics".to_string(),
                    key: "rankings".to_string(),
                    icon: Some("chart-line".to_string()),
                    desc: Some("Rankings availability".to_string()),
                },
                NavItem {
                    label: "Portfolio".to_string(),
                    href: "/portfolio".to_string(),
                    key: "portfolio".to_string(),
                    icon: Some("trending-up".to_string()),
                    desc: Some("Portfolio availability".to_string()),
                },
            ],
        },
        NavGroup {
            label: "Developer".to_string(),
            key: "developer".to_string(),
            icon: Some("code".to_string()),
            items: vec![
                NavItem {
                    label: "API Keys".to_string(),
                    href: "/developer".to_string(),
                    key: "api-keys".to_string(),
                    icon: Some("code".to_string()),
                    desc: Some("API access status".to_string()),
                },
                NavItem {
                    label: "Documentation".to_string(),
                    href: "/developer/docs".to_string(),
                    key: "docs".to_string(),
                    icon: Some("book".to_string()),
                    desc: Some("Pinned API reference".to_string()),
                },
            ],
        },
        NavGroup {
            label: "Company".to_string(),
            key: "company".to_string(),
            icon: Some("building".to_string()),
            items: vec![
                NavItem {
                    label: "About".to_string(),
                    href: "/about".to_string(),
                    key: "about".to_string(),
                    icon: Some("info".to_string()),
                    desc: None,
                },
                NavItem {
                    label: "News".to_string(),
                    href: "/news".to_string(),
                    key: "news".to_string(),
                    icon: Some("newspaper".to_string()),
                    desc: Some("News availability".to_string()),
                },
                NavItem {
                    label: "Contact".to_string(),
                    href: "/contact".to_string(),
                    key: "contact".to_string(),
                    icon: Some("mail".to_string()),
                    desc: None,
                },
                NavItem {
                    label: "Support".to_string(),
                    href: "/chat".to_string(),
                    key: "support".to_string(),
                    icon: Some("help-circle".to_string()),
                    desc: Some("Support status".to_string()),
                },
            ],
        },
    ]
});

/// Bottom-of-page footer links.
pub static FOOTER_LINKS: LazyLock<Vec<NavItem>> = LazyLock::new(|| {
    vec![
        NavItem {
            label: "Terms of Service".to_string(),
            href: "/terms".to_string(),
            key: "terms".to_string(),
            icon: None,
            desc: None,
        },
        NavItem {
            label: "Privacy Policy".to_string(),
            href: "/privacy".to_string(),
            key: "privacy".to_string(),
            icon: None,
            desc: None,
        },
        NavItem {
            label: "Contact".to_string(),
            href: "/contact".to_string(),
            key: "contact".to_string(),
            icon: None,
            desc: None,
        },
    ]
});

/// Check if a path is active within a group (any of the group's items has
/// an `href` exactly equal to `pathname`).
///
/// This is the literal port of the TS `isGroupActive` helper.
pub fn is_group_active(group: &NavGroup, pathname: &str) -> bool {
    group.items.iter().any(|item| item.href == pathname)
}

/// Check if a specific item is the active route.
pub fn is_item_active(item: &NavItem, pathname: &str) -> bool {
    item.href == pathname
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_item_active_matches() {
        let item = NavItem {
            label: "X".into(),
            href: "/about".into(),
            key: "x".into(),
            icon: None,
            desc: None,
        };
        assert!(is_item_active(&item, "/about"));
        assert!(!is_item_active(&item, "/about/team"));
    }

    #[test]
    fn is_group_active_matches_any_item() {
        let about = NavItem {
            label: "About".into(),
            href: "/about".into(),
            key: "a".into(),
            icon: None,
            desc: None,
        };
        let news = NavItem {
            label: "News".into(),
            href: "/news".into(),
            key: "n".into(),
            icon: None,
            desc: None,
        };
        let group = NavGroup {
            label: "Co".into(),
            key: "co".into(),
            icon: None,
            items: vec![about, news],
        };
        assert!(is_group_active(&group, "/about"));
        assert!(is_group_active(&group, "/news"));
        assert!(!is_group_active(&group, "/contact"));
    }

    #[test]
    fn nav_groups_have_unique_keys() {
        let mut keys: Vec<&str> = NAV_GROUPS.iter().map(|g| g.key.as_str()).collect();
        keys.sort();
        let len = keys.len();
        keys.dedup();
        assert_eq!(len, keys.len(), "duplicate group keys in NAV_GROUPS");
    }

    #[test]
    fn nav_groups_preserve_routes_labels_and_order_with_truthful_descriptions() {
        let groups: Vec<(&str, Vec<(&str, &str)>)> = NAV_GROUPS
            .iter()
            .map(|group| {
                (
                    group.label.as_str(),
                    group
                        .items
                        .iter()
                        .map(|item| (item.label.as_str(), item.href.as_str()))
                        .collect(),
                )
            })
            .collect();
        assert_eq!(
            groups,
            vec![
                (
                    "Market",
                    vec![("Rankings", "/analytics"), ("Portfolio", "/portfolio")]
                ),
                (
                    "Developer",
                    vec![
                        ("API Keys", "/developer"),
                        ("Documentation", "/developer/docs"),
                    ]
                ),
                (
                    "Company",
                    vec![
                        ("About", "/about"),
                        ("News", "/news"),
                        ("Contact", "/contact"),
                        ("Support", "/chat"),
                    ]
                ),
            ]
        );

        let description = |key: &str| {
            NAV_GROUPS
                .iter()
                .flat_map(|group| group.items.iter())
                .find(|item| item.key == key)
                .and_then(|item| item.desc.as_deref())
        };
        assert_eq!(description("rankings"), Some("Rankings availability"));
        assert_eq!(description("portfolio"), Some("Portfolio availability"));
        assert_eq!(description("api-keys"), Some("API access status"));
        assert_eq!(description("docs"), Some("Pinned API reference"));
        assert_eq!(description("about"), None);
        assert_eq!(description("news"), Some("News availability"));
        assert_eq!(description("contact"), None);
        assert_eq!(description("support"), Some("Support status"));

        let descriptions: Vec<&str> = NAV_GROUPS
            .iter()
            .flat_map(|group| group.items.iter())
            .filter_map(|item| item.desc.as_deref())
            .collect();
        for unsupported in [
            "Watchlist & tracking",
            "Manage API access",
            "Live chat & help center",
            "EPS stock rankings",
            "API reference",
            "Latest updates",
        ] {
            assert!(
                !descriptions.contains(&unsupported),
                "navigation config retained unsupported description: {unsupported}"
            );
        }
    }
}
