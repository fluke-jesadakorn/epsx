//! Admin sidebar — full 1:1 port of
//! `apps-old/admin-frontend/components/layout/sidebar.tsx`.
//!
//! The original `Sidebar` scaffold is preserved as a thin wrapper around
//! the new TS-parity [`AdminSidebar`] so the public API of
//! `crate::layout::Sidebar` stays importable for downstream callers.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

/// One entry in the admin sidebar navigation tree.
///
/// The struct is the union of the legacy `SidebarItem` shape (used by
/// `DashboardShell`) and the richer fields needed for TS parity
/// (`children`, `requires_auth`, `disabled`, `tab`, `chat_count`).
/// All fields are `Option`/`Vec` so callers can pass just the legacy
/// shape when they don't need the chrome features.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SidebarItem {
    /// Stable id (used as the React `key` and for expand/collapse state).
    pub id: String,
    /// Visible label.
    pub label: String,
    /// Target href. Used for plain (non-expandable) rows and as the
    /// parent href for child matching.
    pub href: String,
    /// Lucide icon name (kebab-case, looked up in
    /// `epsx_templates::lucide`). Empty string renders no icon.
    pub icon: String,
    /// Legacy badge text (rendered right-aligned). Prefer
    /// `chat_count` / `is_active` for the modern badge styling.
    pub badge: Option<String>,
    /// Legacy active-path prefixes. Kept for the existing
    /// `DashboardShell` use; `AdminSidebar` does its own matching.
    pub active_paths: Vec<String>,
    /// Nested children — when present, the row becomes an expand/collapse
    /// toggle instead of a link.
    pub children: Option<Vec<SidebarItem>>,
    /// When `true`, the row is rendered as a disabled "locked" stub if
    /// the user is unauthenticated.
    pub requires_auth: bool,
    /// Force the disabled visual even if authed.
    pub disabled: bool,
    /// Optional `?tab=` query-param fragment appended to the href.
    pub tab: Option<String>,
    /// Optional chat-count badge (purple gradient pill).
    pub chat_count: Option<u32>,
}

/// Full TS-parity admin sidebar.
///
/// Mirrors `apps-old/admin-frontend/components/layout/sidebar.tsx`:
/// - EPSX logo + "ADMIN" subtext at the top
/// - Vertical scrollable nav with nested children (expand/collapse)
/// - Disabled rows show a `Lock` icon when the item requires auth and the
///   caller is unauthed
/// - Per-row badge: chat count pill (purple gradient) or active dot
/// - "Full Access" connect-wallet CTA when unauthed
/// - User pill at the bottom (authed: "Admin user" + emerald dot,
///   guest: "?" + grey dot)
/// - Auto-expand any parent whose `href` is a prefix of the current
///   pathname (mirrors the `useEffect` in the TS source)
///
/// State is **owned internally** for expand/collapse (purely UI) but
/// callers control routing via `current_path` and auth via
/// `is_authenticated`. The `default_expanded` and
/// `expanded_set` props allow controlled override for tests / pages
/// that need to seed the open set.
#[component]
pub fn AdminSidebar(
    /// Current pathname, e.g. `"/wallet-management/access"`. Used for
    /// active-state matching and auto-expand.
    current_path: String,
    /// Whether the viewer is authenticated. When `false`, items with
    /// `requires_auth = true` render as disabled stubs.
    is_authenticated: bool,
    /// Optional override for the sidebar items. When `None`, the
    /// built-in TS-parity `DEFAULT_NAV_ITEMS` are used.
    items: Option<Vec<SidebarItem>>,
    /// Initial expand/collapse set (controlled-mode seed).
    /// `Some(Set)` fixes the open set; `None` lets the component
    /// auto-manage expand state from the pathname.
    default_expanded: Option<Vec<String>>,
    /// Optional `class_name` override for the outer container.
    class_name: Option<String>,
    /// Optional id for the outer container (handy for testing).
    id: Option<String>,
) -> Element {
    let items = items.unwrap_or_else(|| DEFAULT_NAV_ITEMS.clone());

    // Expand/collapse state — owned by the component (purely UI).
    let mut expanded: Signal<std::collections::HashSet<String>> = match default_expanded {
        Some(seed) => use_signal(|| seed.into_iter().collect()),
        None => use_signal(std::collections::HashSet::new),
    };

    // Auto-expand any parent whose href is a prefix of the current path
    // (mirrors the `useEffect(() => { setExpandedItems(...) }, [pathname])`
    // in the TS source).
    {
        let items_for_effect = items.clone();
        let path_for_effect = current_path.clone();
        use_effect(move || {
            let mut next = expanded.write();
            for it in items_for_effect.iter() {
                if it.children.is_some() && path_for_effect.starts_with(&it.href) {
                    next.insert(it.id.clone());
                }
            }
        });
    }

    let active_user_pill = is_authenticated;

    let container_class = {
        let mut c = String::from("admin-sidebar w-56 sm:w-64 min-w-0 max-w-64 bg-card border-r border-border/40 h-full flex flex-col z-20");
        if let Some(extra) = class_name { c.push(' '); c.push_str(&extra); }
        c
    };

    rsx! {
        aside {
            class: "{container_class}",
            id: id.clone(),
            "aria-label": "Sidebar",
            role: "navigation",

            // ── Brand block ────────────────────────────────────────────
            div { class: "px-6 pt-5 pb-4",
                a { class: "flex items-center gap-3 group", href: "/",
                    div { class: "relative",
                        div { class: "absolute inset-0 bg-gradient-to-br from-[#FF512F] to-[#DD2476] blur-xl opacity-20 group-hover:opacity-40 transition-opacity" }
                        div { class: "relative z-10 group-active:scale-95 transition-transform",
                            span { class: "epsx-icon epsx-icon-brand", dangerous_inner_html: "{epsx_templates::epsx_icon_svg()}" }
                        }
                    }
                    div { class: "flex flex-col justify-center",
                        span { class: "text-2xl font-black tracking-widest text-transparent bg-clip-text bg-gradient-to-r from-[#FF512F] to-[#DD2476] leading-none", "EPSX" }
                        span { class: "text-[10px] uppercase tracking-[0.3em] font-bold text-[#FF512F] mt-0.5 ml-0.5", "ADMIN" }
                    }
                }
            }

            // ── Nav list ───────────────────────────────────────────────
            nav { class: "flex-1 overflow-y-auto px-4 space-y-1",
                for item in items.iter() {
                    // Hide the "Connect Wallet" item once authenticated (matches TS filter).
                    if !(item.id == "auth" && is_authenticated) {
                        SidebarRow {
                            item: item.clone(),
                            current_path: current_path.clone(),
                            is_authenticated,
                            chat_count: item.chat_count.unwrap_or(0),
                            expanded: expanded.clone(),
                        }
                    }
                }
            }

            // ── Bottom block: connect CTA + user pill ──────────────────
            div { class: "mt-auto p-4",
                if !is_authenticated {
                    ConnectWalletCta { return_url: current_path.clone() }
                }
                UserPill { is_authenticated: active_user_pill }
            }
        }
    }
}

/// Connect-wallet CTA card shown in the sidebar when the viewer is
/// unauthenticated. Mirrors the gradient card in the TS source.
#[component]
fn ConnectWalletCta(return_url: String) -> Element {
    let href = format!("/auth?return_url={}", urlencode(&return_url));
    rsx! {
        div { class: "mb-4",
            div { class: "bg-gradient-to-br from-[#1fc7d4]/5 to-[#7645d9]/5 rounded-xl p-4 border border-border/40 relative overflow-hidden group",
                div { class: "absolute -right-4 -top-4 w-16 h-16 bg-[#1fc7d4]/10 rounded-full blur-2xl group-hover:bg-[#1fc7d4]/20 transition-colors" }
                div { class: "relative z-10 text-center",
                    p { class: "text-sm font-bold text-foreground mb-1", "Full Access" }
                    p { class: "text-[10px] text-muted-foreground mb-4 px-2", "Unlock all features by connecting your wallet." }
                    a { class: "admin-sidebar-cta block w-full bg-[#1fc7d4] text-white text-sm font-bold py-2.5 px-4 rounded-2xl shadow-lg shadow-cyan-500/20 hover:shadow-cyan-500/40 hover:scale-[1.02] active:scale-95 transition-all text-center",
                        href: "{href}",
                        "Connect Wallet"
                    }
                }
            }
        }
    }
}

/// User pill at the bottom of the sidebar.
#[component]
fn UserPill(is_authenticated: bool) -> Element {
    rsx! {
        div { class: "bg-muted/30 rounded-xl p-3 border border-border/40",
            div { class: "flex items-center gap-3",
                div { class: if is_authenticated {
                    "w-10 h-10 rounded-2xl flex items-center justify-center text-white text-sm font-bold shadow-lg transition-all bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] shadow-cyan-500/10"
                } else {
                    "w-10 h-10 rounded-2xl flex items-center justify-center text-white text-sm font-bold shadow-lg transition-all bg-muted/50 text-muted-foreground shadow-none"
                },
                    if is_authenticated { "AU" } else { "?" }
                }
                div { class: "flex-1 min-w-0",
                    p { class: "text-xs font-bold text-foreground truncate",
                        if is_authenticated { "Admin user" } else { "Guest" }
                    }
                    div { class: "flex items-center gap-1.5",
                        div { class: if is_authenticated { "w-1.5 h-1.5 rounded-full bg-emerald-500" } else { "w-1.5 h-1.5 rounded-full bg-slate-500" } }
                        p { class: "text-[10px] font-bold text-muted-foreground tracking-wide uppercase",
                            if is_authenticated { "Connected" } else { "Offline" }
                        }
                    }
                }
            }
        }
    }
}

/// Internal: one row in the sidebar (parent or leaf).
#[component]
fn SidebarRow(
    item: SidebarItem,
    current_path: String,
    is_authenticated: bool,
    chat_count: u32,
    expanded: Signal<std::collections::HashSet<String>>,
) -> Element {
    let is_active = current_path == item.href
        || (item.href != "/" && current_path.starts_with(&format!("{}/", item.href)));
    let is_expanded = expanded.read().contains(&item.id);
    let has_children = item.children.is_some();
    let is_disabled = item.disabled || (item.requires_auth && !is_authenticated);
    let has_active_child = match &item.children {
        Some(children) => children.iter().any(|c| is_child_active(c, &current_path, None)),
        None => false,
    };
    let is_highlighted = is_active || has_active_child;

    // Disabled visual — locked stub.
    if is_disabled {
        return rsx! {
            div { class: "mb-1",
                div { class: "flex items-center gap-3 px-4 py-2.5 rounded-2xl cursor-not-allowed opacity-40 text-muted-foreground grayscale",
                    if !item.icon.is_empty() { Icon { name: item.icon.clone(), size: Some(20) } }
                    span { class: "text-sm font-semibold truncate", "{item.label}" }
                    Icon { name: "lock".to_string(), size: Some(14), class_name: Some("ml-auto flex-shrink-0".to_string()) }
                }
            }
        };
    }

    // Expand/collapse parent.
    if has_children {
        let id_for_toggle = item.id.clone();
        let child_id = format!("sidebar-children-{}", id_for_toggle);
        return rsx! {
            div { class: "mb-1",
                button {
                    class: if is_highlighted { "w-full flex items-center gap-3 px-4 py-2.5 rounded-2xl transition-all duration-200 active:scale-[0.98] admin-nav-row admin-nav-row-active bg-gradient-to-r from-[#1fc7d4]/10 to-[#7645d9]/10 text-[#1fc7d4] border border-[#1fc7d4]/20 shadow-sm" } else { "w-full flex items-center gap-3 px-4 py-2.5 rounded-2xl transition-all duration-200 active:scale-[0.98] admin-nav-row text-muted-foreground hover:bg-muted/30 hover:text-foreground" },
                    r#type: "button",
                    "aria-expanded": if is_expanded { "true" } else { "false" },
                    "aria-controls": "{child_id}",
                    onclick: move |_| {
                        let mut set = expanded.write();
                        if set.contains(&id_for_toggle) { set.remove(&id_for_toggle); } else { set.insert(id_for_toggle.clone()); }
                    },
                    if !item.icon.is_empty() {
                        Icon {
                            name: item.icon.clone(),
                            size: Some(20),
                            class_name: Some("flex-shrink-0".to_string()),
                        }
                    }
                    span { class: "text-sm font-semibold truncate", "{item.label}" }
                    Icon {
                        name: "chevron-right".to_string(),
                        size: Some(14),
                        class_name: Some(format!("flex-shrink-0 ml-auto transition-transform duration-200{}", if is_expanded { " rotate-90" } else { "" })),
                    }
                }
                if is_expanded {
                    NavChildren {
                        item: item.clone(),
                        current_path: current_path.clone(),
                        child_id: child_id.clone(),
                    }
                }
            }
        };
    }

    // Plain leaf row.
    let href = if item.id == "auth" {
        format!("/auth?return_url={}", urlencode(&current_path))
    } else {
        match &item.tab {
            Some(t) if !t.is_empty() => format!("{}?tab={}", item.href, t),
            _ => item.href.clone(),
        }
    };
    rsx! {
        div { class: "mb-1",
            a {
                class: if is_active { "admin-nav-row admin-nav-row-active flex items-center gap-3 px-4 py-2.5 rounded-2xl transition-all duration-200 group-active:scale-[0.98] bg-gradient-to-r from-[#1fc7d4]/10 to-[#7645d9]/10 text-[#1fc7d4] border border-[#1fc7d4]/20 shadow-sm" } else { "admin-nav-row flex items-center gap-3 px-4 py-2.5 rounded-2xl transition-all duration-200 group-active:scale-[0.98] text-muted-foreground hover:bg-muted/30 hover:text-foreground" },
                href: "{href}",
                "aria-current": if is_active { "page" } else { "false" },
                if !item.icon.is_empty() {
                    Icon {
                        name: item.icon.clone(),
                        size: Some(20),
                        class_name: Some("flex-shrink-0".to_string()),
                    }
                }
                span { class: "text-sm font-semibold truncate", "{item.label}" }
                NavItemBadge { item_id: item.id.clone(), chat_count, is_active }
                if let Some(b) = &item.badge {
                    span { class: "ml-auto text-[10px] font-bold text-muted-foreground", "{b}" }
                }
            }
        }
    }
}

/// Sub-list of children, rendered indented under a parent row.
#[component]
fn NavChildren(item: SidebarItem, current_path: String, child_id: String) -> Element {
    let children = match item.children.clone() {
        Some(c) => c,
        None => return rsx! { Fragment {} },
    };
    rsx! {
        div { class: "ml-6 mt-1 space-y-0.5 border-l border-border/40 pl-2", id: child_id, role: "list",
            for child in children.iter() {
                {
                    let child_active = is_child_active(child, &current_path, None);
                    let child_href = match &child.tab {
                        Some(t) if !t.is_empty() => format!("{}?tab={}", child.href, t),
                        _ => child.href.clone(),
                    };
                    rsx! {
                        a { class: if child_active { "flex items-center gap-3 px-3 py-2 rounded-xl transition-all text-[#1fc7d4] bg-[#1fc7d4]/5 font-bold" } else { "flex items-center gap-3 px-3 py-2 rounded-xl transition-all text-muted-foreground hover:text-foreground hover:bg-muted/30" },
                            href: "{child_href}",
                            "aria-current": if child_active { "page" } else { "false" },
                            if !child.icon.is_empty() {
                                Icon {
                                    name: child.icon.clone(),
                                    size: Some(16),
                                    class_name: Some("flex-shrink-0".to_string()),
                                }
                            }
                            span { class: "text-xs font-medium truncate", "{child.label}" }
                            if child_active {
                                div { class: "w-1 h-1 rounded-full bg-[#1fc7d4] ml-auto" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Right-aligned badge: chat count pill (purple gradient) or active dot.
#[component]
fn NavItemBadge(item_id: String, chat_count: u32, is_active: bool) -> Element {
    if item_id == "chat" && chat_count > 0 {
        let text = if chat_count > 99 { "99+".to_string() } else { chat_count.to_string() };
        return rsx! {
            span { class: "ml-auto min-w-[20px] h-5 px-1.5 rounded-full bg-gradient-to-r from-violet-500 to-purple-500 text-white text-[10px] font-bold flex items-center justify-center shadow-sm shadow-violet-500/30",
                "{text}"
            }
        };
    }
    if is_active {
        return rsx! {
            div { class: "w-1.5 h-1.5 rounded-full bg-[#1fc7d4] ml-auto animate-pulse" }
        };
    }
    rsx! { Fragment {} }
}

/// TS-parity default nav items — the full admin sidebar tree.
///
/// Mirrors `navigationItems` in `sidebar.tsx` line-for-line (id, label,
/// href, icon, requires_auth, children, tab). Consumer code can pass a
/// custom set via `AdminSidebar`'s `items` prop when the page needs a
/// different set.
pub fn default_nav_items() -> Vec<SidebarItem> { DEFAULT_NAV_ITEMS.clone() }

/// One static copy of the default nav tree; exposed as a `const` style
/// `static` via a function so callers can clone it cheaply.
pub static DEFAULT_NAV_ITEMS: std::sync::LazyLock<Vec<SidebarItem>> = std::sync::LazyLock::new(|| vec![
    SidebarItem { id: "dashboard".into(), label: "Dashboard".into(), href: "/".into(), icon: "home".into(), ..Default::default() },
    SidebarItem { id: "auth".into(), label: "Connect Wallet".into(), href: "/auth".into(), icon: "link".into(), ..Default::default() },
    SidebarItem {
        id: "wallet-management".into(), label: "Wallet Mgmt".into(), href: "/wallet-management".into(), icon: "wallet".into(), requires_auth: true,
        children: Some(vec![
            SidebarItem { id: "wm-wallets".into(), label: "Wallets".into(), href: "/wallet-management/wallets".into(), icon: "wallet".into(), ..Default::default() },
            SidebarItem { id: "wm-access".into(), label: "Access".into(), href: "/wallet-management/access".into(), icon: "shield".into(), ..Default::default() },
            SidebarItem { id: "wm-credits".into(), label: "Credits".into(), href: "/wallet-management/credits".into(), icon: "coins".into(), ..Default::default() },
        ]),
        ..Default::default()
    },
    SidebarItem {
        id: "payments".into(), label: "Payments".into(), href: "/payments".into(), icon: "credit-card".into(), requires_auth: true,
        children: Some(vec![
            SidebarItem { id: "pay-payments".into(), label: "Payments".into(), href: "/payments".into(), icon: "credit-card".into(), tab: Some("payments".into()), ..Default::default() },
            SidebarItem { id: "pay-access".into(), label: "User Access".into(), href: "/payments".into(), icon: "users".into(), tab: Some("user-access".into()), ..Default::default() },
            SidebarItem { id: "pay-links".into(), label: "Links".into(), href: "/payments".into(), icon: "link-2".into(), tab: Some("payment-links".into()), ..Default::default() },
        ]),
        ..Default::default()
    },
    SidebarItem { id: "chat".into(), label: "Chat Support".into(), href: "/chat".into(), icon: "message-circle".into(), requires_auth: true, ..Default::default() },
    SidebarItem { id: "news".into(), label: "News".into(), href: "/news".into(), icon: "newspaper".into(), requires_auth: true, ..Default::default() },
    SidebarItem { id: "media".into(), label: "Media".into(), href: "/media".into(), icon: "image".into(), requires_auth: true, ..Default::default() },
    SidebarItem { id: "analytics".into(), label: "Analytics".into(), href: "/analytics".into(), icon: "bar-chart-3".into(), requires_auth: true, ..Default::default() },
    SidebarItem { id: "audit-log".into(), label: "Audit Log".into(), href: "/audit-log".into(), icon: "file-text".into(), requires_auth: true, ..Default::default() },
    SidebarItem {
        id: "developer".into(), label: "Developer".into(), href: "/developer-portal".into(), icon: "code".into(), requires_auth: true,
        children: Some(vec![
            SidebarItem { id: "dev-overview".into(), label: "Overview".into(), href: "/developer-portal".into(), icon: "layout-dashboard".into(), tab: Some("overview".into()), ..Default::default() },
            SidebarItem { id: "dev-keys".into(), label: "API Keys".into(), href: "/developer-portal".into(), icon: "key".into(), tab: Some("keys".into()), ..Default::default() },
            SidebarItem { id: "dev-docs".into(), label: "Docs".into(), href: "/developer-portal".into(), icon: "book-open".into(), tab: Some("docs".into()), ..Default::default() },
            SidebarItem { id: "dev-usage".into(), label: "Usage".into(), href: "/developer-portal".into(), icon: "trending-up".into(), tab: Some("usage".into()), ..Default::default() },
        ]),
        ..Default::default()
    },
    SidebarItem {
        id: "notifications".into(), label: "Notifications".into(), href: "/notifications".into(), icon: "bell".into(), requires_auth: true,
        children: Some(vec![
            SidebarItem { id: "notif-manage".into(), label: "Overview".into(), href: "/notifications/manage".into(), icon: "bell".into(), ..Default::default() },
            SidebarItem { id: "notif-create".into(), label: "Send Signal".into(), href: "/notifications/create".into(), icon: "send".into(), ..Default::default() },
        ]),
        ..Default::default()
    },
    SidebarItem {
        id: "settings".into(), label: "Settings".into(), href: "/settings".into(), icon: "settings".into(),
        children: Some(vec![
            SidebarItem { id: "set-general".into(), label: "Nodes".into(), href: "/settings".into(), icon: "globe".into(), tab: Some("general".into()), ..Default::default() },
            SidebarItem { id: "set-notifications".into(), label: "Signals".into(), href: "/settings".into(), icon: "bell".into(), tab: Some("notifications".into()), ..Default::default() },
            SidebarItem { id: "set-security".into(), label: "Vault".into(), href: "/settings".into(), icon: "lock".into(), tab: Some("security".into()), ..Default::default() },
            SidebarItem { id: "set-appearance".into(), label: "Optics".into(), href: "/settings".into(), icon: "palette".into(), tab: Some("appearance".into()), ..Default::default() },
        ]),
        ..Default::default()
    },
]);

/// TS-parity helper: is this child the active route? Mirrors
/// `isChildActive` in `sidebar.tsx` — a child matches if its `tab` is
/// the current `?tab=` value (or no tab is set) AND its href matches the
/// pathname (with optional `/`-prefix match for index pages).
pub fn is_child_active(item: &SidebarItem, current_path: &str, current_tab: Option<&str>) -> bool {
    if let Some(tab) = &item.tab {
        if !tab.is_empty() {
            return current_path == item.href && current_tab == Some(tab.as_str());
        }
    }
    current_path == item.href || current_path.starts_with(&format!("{}/", item.href))
}

/// Minimal percent-encoder for query values (only encodes the chars that
/// matter for `return_url=` round-tripping through the Dioxus SSR
/// template). Kept local so we don't pull in the `percent-encoding` crate.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────
// Legacy scaffold re-export
// ─────────────────────────────────────────────────────────────────────────

/// Legacy thin wrapper kept for backward compatibility with
/// `DashboardShell`. New code should use [`AdminSidebar`].
///
/// The `header` param is ignored (the TS source's brand block is now
/// hard-coded to "EPSX / ADMIN"; pass a different nav set via
/// [`AdminSidebar`] for custom branding).
#[component]
pub fn Sidebar(
    items: Vec<SidebarItem>,
    current_path: String,
    /// Legacy header label — ignored; kept for API stability.
    #[allow(unused_variables)] header: Option<String>,
    is_authenticated: Option<bool>,
) -> Element {
    rsx! {
        AdminSidebar {
            current_path,
            is_authenticated: is_authenticated.unwrap_or(false),
            items: Some(items),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_child_active_matches_tabbed_route() {
        let item = SidebarItem {
            id: "set-general".into(), label: "Nodes".into(), href: "/settings".into(), icon: "globe".into(),
            tab: Some("general".into()),
            ..Default::default()
        };
        assert!(is_child_active(&item, "/settings", Some("general")));
        assert!(!is_child_active(&item, "/settings", Some("appearance")));
        assert!(!is_child_active(&item, "/settings/other", Some("general")));
    }

    #[test]
    fn is_child_active_matches_index_prefix() {
        let item = SidebarItem {
            id: "wm-wallets".into(), label: "Wallets".into(), href: "/wallet-management/wallets".into(), icon: "wallet".into(),
            ..Default::default()
        };
        assert!(is_child_active(&item, "/wallet-management/wallets", None));
        assert!(is_child_active(&item, "/wallet-management/wallets/123", None));
        assert!(!is_child_active(&item, "/wallet-management/access", None));
    }

    #[test]
    fn urlencode_keeps_unreserved_chars() {
        assert_eq!(urlencode("/wallet-management/access"), "%2Fwallet-management%2Faccess");
        assert_eq!(urlencode("abc-123_X.~"), "abc-123_X.~");
    }
}
