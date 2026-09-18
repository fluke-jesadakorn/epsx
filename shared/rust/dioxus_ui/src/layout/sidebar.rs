//! Shared admin navigation inventory and sidebar. The hydrated desktop and
//! mobile menus consume this same tree; aliases remain routable without
//! appearing as duplicate destinations.

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
    /// Stable id for keyed rows and expand/collapse state.
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

/// Admin sidebar with query-aware selection and automatic parent expansion.
/// Expand/collapse is UI state; authentication truth comes from the caller.
#[component]
pub fn AdminSidebar(
    /// Current URL path and query, used for active selection and expansion.
    current_path: String,
    /// Whether the viewer is authenticated. When `false`, items with
    /// `requires_auth = true` render as disabled stubs.
    is_authenticated: bool,
    /// Optional override for the sidebar items. When `None`, the
    /// shared `DEFAULT_NAV_ITEMS` are used.
    items: Option<Vec<SidebarItem>>,
    /// Initial expand/collapse set (controlled-mode seed).
    /// `Some(Set)` supplies the initial open set; navigation also opens
    /// the parent containing the active destination.
    default_expanded: Option<Vec<String>>,
    /// Optional `class_name` override for the outer container.
    class_name: Option<String>,
    /// Optional id for the outer container (handy for testing).
    id: Option<String>,
    /// SSR-verified session truth. When `None`, it is derived from
    /// `is_authenticated` (verified vs anonymous) for legacy callers.
    #[props(default = None)]
    session_state: Option<crate::layout::session_state::SessionState>,
) -> Element {
    let navigation = try_consume_context::<crate::fullstack::admin::AdminNavigation>();
    let items = items.unwrap_or_else(|| DEFAULT_NAV_ITEMS.clone());
    let active_id = active_nav_item(&items, &current_path).map(|item| item.id.clone());

    // Expand/collapse state — owned by the component (purely UI). Seed
    // route-matching parents synchronously so SSR emits the same initially
    // expanded tree as the production client. `use_effect` does not run
    // during SSR, so the active submenu must be open before hydration.
    let expanded_seed: std::collections::HashSet<String> = match default_expanded {
        Some(seed) => seed.into_iter().collect(),
        None => items
            .iter()
            .filter(|item| contains_active_child(item, active_id.as_deref()))
            .map(|item| item.id.clone())
            .collect(),
    };
    let mut expanded: Signal<std::collections::HashSet<String>> = use_signal(move || expanded_seed);

    // Follow route props after client-side navigation, including children whose
    // destinations live outside their parent's URL prefix (plans/subscriptions).
    use_effect(use_reactive!(|items, current_path| {
        let active_id = active_nav_item(&items, &current_path).map(|item| item.id.clone());
        let mut next = expanded.peek().clone();
        for item in &items {
            if contains_active_child(item, active_id.as_deref()) {
                next.insert(item.id.clone());
            }
        }
        if next != *expanded.peek() {
            expanded.set(next);
        }
    }));

    let session_state = session_state.unwrap_or({
        if is_authenticated {
            crate::layout::session_state::SessionState::Verified
        } else {
            crate::layout::session_state::SessionState::Anonymous
        }
    });

    let container_class = {
        let mut c = String::from("admin-sidebar w-56 sm:w-64 min-w-0 max-w-64 bg-card border-r border-border/40 h-full flex flex-col z-20");
        if let Some(extra) = class_name {
            c.push(' ');
            c.push_str(&extra);
        }
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
                crate::navigation::AppLink { class: "flex items-center gap-3 group", href: "/", onclick: move |event| crate::fullstack::admin::follow_admin_link(event,navigation,"/"),
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
                            key: "{item.id}",
                            item: item.clone(),
                            active_id: active_id.clone(),
                            current_path: current_path.clone(),
                            is_authenticated,
                            chat_count: item.chat_count.unwrap_or(0),
                            expanded: expanded,
                        }
                    }
                }
            }

            // ── Session status ────────────────────────────────────────
            div { class: "mt-auto p-4",
                UserPill { session_state }
            }
        }
    }
}

/// Session status supplied by the server, without duplicating account actions.
#[component]
fn UserPill(session_state: crate::layout::session_state::SessionState) -> Element {
    let (avatar_class, initials, dot_class, label) = match session_state {
        crate::layout::session_state::SessionState::Verified => (
            "w-10 h-10 rounded-2xl flex items-center justify-center text-white text-sm font-bold shadow-lg transition-all bg-gradient-to-br from-[#1fc7d4] to-[#7645d9] shadow-cyan-500/10",
            "AU",
            "w-1.5 h-1.5 rounded-full bg-emerald-500",
            "Authenticated",
        ),
        crate::layout::session_state::SessionState::Fixture => (
            "w-10 h-10 rounded-2xl flex items-center justify-center text-white text-sm font-bold shadow-lg transition-all bg-gradient-to-br from-amber-400 to-orange-500 shadow-amber-500/20",
            "FX",
            "w-1.5 h-1.5 rounded-full bg-amber-400",
            "Fixture mode",
        ),
        crate::layout::session_state::SessionState::Anonymous => (
            "w-10 h-10 rounded-2xl flex items-center justify-center text-white text-sm font-bold shadow-lg transition-all bg-muted/50 text-muted-foreground shadow-none",
            "?",
            "w-1.5 h-1.5 rounded-full bg-slate-500",
            "Offline",
        )
    };
    rsx! {
        div { class: "bg-muted/30 rounded-xl p-3 border border-border/40",
            "data-session-state": session_state.as_str(),
            div { class: "flex items-center gap-3",
                div { class: avatar_class, "{initials}" }
                div { class: "flex-1 min-w-0",
                    p { class: "text-xs font-bold text-foreground truncate",
                        if session_state == crate::layout::session_state::SessionState::Anonymous {
                            "Guest"
                        } else {
                            "Admin user"
                        }
                    }
                    div { class: "flex items-center gap-1.5",
                        div { class: dot_class }
                        p { class: "text-[10px] font-bold text-muted-foreground tracking-wide uppercase", "{label}" }
                    }
                }
                // Production keeps account actions in the header wallet
                // dropdown; the sidebar pill is status-only.
            }
        }
    }
}

/// Internal: one row in the sidebar (parent or leaf).
#[component]
fn SidebarRow(
    item: SidebarItem,
    active_id: Option<String>,
    current_path: String,
    is_authenticated: bool,
    chat_count: u32,
    expanded: Signal<std::collections::HashSet<String>>,
) -> Element {
    let navigation = try_consume_context::<crate::fullstack::admin::AdminNavigation>();
    let is_active = active_id.as_deref() == Some(item.id.as_str());
    let is_expanded = expanded.read().contains(&item.id);
    let has_children = item.children.is_some();
    let is_disabled = item.disabled || (item.requires_auth && !is_authenticated);
    let is_highlighted = is_active || contains_active_child(&item, active_id.as_deref());

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
                    "data-epsx-action": "toggle-nav",
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
                        class_name: Some(format!("admin-nav-chevron flex-shrink-0 ml-auto transition-transform duration-200{}", if is_expanded { " rotate-90" } else { "" })),
                    }
                }
                NavChildren {
                    item: item.clone(),
                    active_id: active_id.clone(),
                    is_authenticated,
                    child_id: child_id.clone(),
                    expanded: is_expanded,
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
            crate::navigation::AppLink {
                class: if is_active { "admin-nav-row admin-nav-row-active flex items-center gap-3 px-4 py-2.5 rounded-2xl transition-all duration-200 group-active:scale-[0.98] bg-gradient-to-r from-[#1fc7d4]/10 to-[#7645d9]/10 text-[#1fc7d4] border border-[#1fc7d4]/20 shadow-sm" } else { "admin-nav-row flex items-center gap-3 px-4 py-2.5 rounded-2xl transition-all duration-200 group-active:scale-[0.98] text-muted-foreground hover:bg-muted/30 hover:text-foreground" },
                href: "{href}",
                onclick: move |event| {
                    crate::fullstack::admin::follow_admin_link(event, navigation, &href);
                },
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
fn NavChildren(
    item: SidebarItem,
    active_id: Option<String>,
    is_authenticated: bool,
    child_id: String,
    expanded: bool,
) -> Element {
    let navigation = try_consume_context::<crate::fullstack::admin::AdminNavigation>();
    let children = match item.children.clone() {
        Some(c) => c,
        None => return rsx! { Fragment {} },
    };
    rsx! {
        div {
            class: "admin-nav-children ml-6 mt-1 space-y-0.5 border-l border-border/40 pl-2",
            id: child_id,
            role: "list",
            hidden: !expanded,
            "aria-hidden": if expanded { "false" } else { "true" },
            for child in children.iter() {
                {
                    let child_active = active_id.as_deref() == Some(child.id.as_str());
                    let child_disabled = child.disabled || (child.requires_auth && !is_authenticated);
                    let child_href = match &child.tab {
                        Some(t) if !t.is_empty() => format!("{}?tab={}", child.href, t),
                        _ => child.href.clone(),
                    };
                    rsx! {
                        if child_disabled {
                            span { key: "{child.id}", class: "flex items-center gap-3 px-3 py-2 text-xs text-muted-foreground opacity-40", aria_disabled: "true", "{child.label}" }
                        } else {
                        crate::navigation::AppLink { key: "{child.id}", class: if child_active { "flex items-center gap-3 px-3 py-2 rounded-xl transition-all text-[#1fc7d4] bg-[#1fc7d4]/5 font-bold" } else { "flex items-center gap-3 px-3 py-2 rounded-xl transition-all text-muted-foreground hover:text-foreground hover:bg-muted/30" },
                            href: "{child_href}",
                            onclick: move |event| { crate::fullstack::admin::follow_admin_link(event, navigation, &child_href); },
                            "aria-current": if child_active { "page" } else { "false" },
                            if !child.icon.is_empty() {
                                Icon {
                                    name: child.icon.clone(),
                                    size: Some(16),
                                    class_name: Some("flex-shrink-0".to_string()),
                                }
                            }
                            span { class: "min-w-0 text-xs font-medium text-left", "{child.label}" }
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
}

/// Right-aligned badge: chat count pill (purple gradient) or active dot.
#[component]
fn NavItemBadge(item_id: String, chat_count: u32, is_active: bool) -> Element {
    if item_id == "chat" && chat_count > 0 {
        let text = if chat_count > 99 {
            "99+".to_string()
        } else {
            chat_count.to_string()
        };
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

/// One navigation inventory for the sidebar, desktop dropdowns and mobile menu.
pub fn default_nav_items() -> Vec<SidebarItem> {
    DEFAULT_NAV_ITEMS.clone()
}

fn nav_link(id: &str, label: &str, href: &str, icon: &str) -> SidebarItem {
    SidebarItem {
        id: id.into(),
        label: label.into(),
        href: href.into(),
        icon: icon.into(),
        requires_auth: true,
        ..Default::default()
    }
}

fn nav_tab(id: &str, label: &str, href: &str, icon: &str, tab: &str) -> SidebarItem {
    SidebarItem {
        tab: Some(tab.into()),
        ..nav_link(id, label, href, icon)
    }
}

fn nav_group(
    id: &str,
    label: &str,
    href: &str,
    icon: &str,
    children: Vec<SidebarItem>,
) -> SidebarItem {
    SidebarItem {
        children: Some(children),
        ..nav_link(id, label, href, icon)
    }
}

pub static DEFAULT_NAV_ITEMS: std::sync::LazyLock<Vec<SidebarItem>> =
    std::sync::LazyLock::new(|| {
        vec![
            SidebarItem {
                requires_auth: false,
                ..nav_link("dashboard", "Dashboard", "/", "home")
            },
            SidebarItem {
                requires_auth: false,
                ..nav_link("auth", "Connect Wallet", "/auth", "link")
            },
            nav_link("analytics", "Analytics", "/analytics", "bar-chart-3"),
            nav_group(
                "wallet-management",
                "Wallets & access",
                "/wallet-management",
                "wallet",
                vec![
                    nav_link(
                        "wm-wallets",
                        "Wallets & permissions",
                        "/wallet-management/wallets",
                        "wallet",
                    ),
                    nav_tab(
                        "wm-subscriptions",
                        "Subscriptions",
                        "/payments",
                        "users",
                        "user-access",
                    ),
                    nav_link("wm-plans", "Plan catalog", "/plans", "layers"),
                    nav_link(
                        "wm-credits",
                        "Credits",
                        "/wallet-management/credits",
                        "coins",
                    ),
                ],
            ),
            nav_group(
                "payments",
                "Payments",
                "/payments",
                "credit-card",
                vec![
                    nav_link(
                        "pay-purchases",
                        "Plan purchases",
                        "/payments/epsx",
                        "credit-card",
                    ),
                    nav_tab(
                        "pay-payments",
                        "Payment intents",
                        "/payments",
                        "credit-card",
                        "payments",
                    ),
                    nav_tab(
                        "pay-links",
                        "Payment links",
                        "/payments",
                        "link-2",
                        "payment-links",
                    ),
                    nav_link("pay-escrows", "Escrows", "/pay/escrows", "shield"),
                    nav_link(
                        "pay-merchant-escrows",
                        "Merchant escrows",
                        "/pay/merchant-escrows",
                        "building",
                    ),
                ],
            ),
            nav_group(
                "content",
                "Content",
                "/news",
                "newspaper",
                vec![
                    nav_link("news", "News", "/news", "newspaper"),
                    nav_link("news-create", "Create news", "/news/create", "plus"),
                    nav_link("media", "Media library", "/media", "image"),
                ],
            ),
            nav_link("chat", "Chat support", "/chat", "message-circle"),
            nav_group(
                "notifications",
                "Notifications",
                "/notifications",
                "bell",
                vec![
                    nav_link(
                        "notif-manage",
                        "Manage notifications",
                        "/notifications/manage",
                        "bell",
                    ),
                    nav_link(
                        "notif-create",
                        "Send notification",
                        "/notifications/create",
                        "send",
                    ),
                ],
            ),
            nav_group(
                "developer",
                "Developer",
                "/developer-portal",
                "code",
                vec![
                    nav_tab(
                        "dev-overview",
                        "Overview",
                        "/developer-portal",
                        "layout-dashboard",
                        "overview",
                    ),
                    nav_tab("dev-keys", "API keys", "/developer-portal", "key", "keys"),
                    nav_link(
                        "dev-create",
                        "Create API key",
                        "/developer-portal/api-keys/create",
                        "plus",
                    ),
                    nav_tab(
                        "dev-usage",
                        "Usage",
                        "/developer-portal",
                        "trending-up",
                        "usage",
                    ),
                    nav_tab(
                        "dev-docs",
                        "Documentation",
                        "/developer-portal",
                        "book-open",
                        "docs",
                    ),
                ],
            ),
            nav_group(
                "settings",
                "Settings",
                "/settings",
                "settings",
                vec![
                    nav_tab("set-general", "General", "/settings", "globe", "general"),
                    nav_tab(
                        "set-notifications",
                        "Notification preferences",
                        "/settings",
                        "bell",
                        "notifications",
                    ),
                    nav_tab("set-security", "Security", "/settings", "lock", "security"),
                    nav_tab(
                        "set-appearance",
                        "Appearance",
                        "/settings",
                        "palette",
                        "appearance",
                    ),
                    nav_link("audit-log", "Audit log", "/audit-log", "file-text"),
                ],
            ),
        ]
    });

impl SidebarItem {
    pub fn destination(&self) -> String {
        match self.tab.as_deref().filter(|tab| !tab.is_empty()) {
            Some(tab) => format!("{}?tab={tab}", self.href),
            None => self.href.clone(),
        }
    }
}

fn contains_active_child(item: &SidebarItem, active_id: Option<&str>) -> bool {
    item.children.as_ref().is_some_and(|children| {
        children.iter().any(|child| {
            active_id == Some(child.id.as_str()) || contains_active_child(child, active_id)
        })
    })
}

/// Normalize existing route aliases for display only; routing and authorization
/// remain owned by the Router and backend.
fn canonical_path(path: &str) -> String {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let path = path
        .strip_prefix("/admin/")
        .map(|tail| format!("/{tail}"))
        .unwrap_or_else(|| path.to_owned());
    match path.as_str() {
        "/admin" | "/index" | "/dashboard" => "/".into(),
        "/notifications" => "/notifications/manage".into(),
        "/wallet-management/access" => "/wallet-management/wallets".into(),
        path if path == "/wallet-management/access/plans"
            || path.starts_with("/wallet-management/access/plans/") =>
        {
            path.replacen("/wallet-management/access/plans", "/plans", 1)
        }
        path if path.starts_with("/wallet-management/0x") => {
            path.replacen("/wallet-management/", "/wallet-management/wallets/", 1)
        }
        _ => path,
    }
}

/// Match a destination, including tab defaults and existing route aliases.
pub fn is_child_active(item: &SidebarItem, current_path: &str, current_tab: Option<&str>) -> bool {
    let path = canonical_path(current_path);
    let query_tab = current_path.split_once('?').and_then(|(_, query)| {
        url::form_urlencoded::parse(query.split('#').next().unwrap_or(query).as_bytes())
            .find(|(key, _)| key == "tab")
            .map(|(_, value)| value.into_owned())
    });
    let tab = current_tab
        .or(query_tab.as_deref())
        .or(match path.as_str() {
            "/payments" => Some("payments"),
            "/settings" => Some("general"),
            "/developer-portal" => Some("overview"),
            _ => None,
        });
    if let Some(expected) = item.tab.as_deref().filter(|tab| !tab.is_empty()) {
        return path == item.href && tab == Some(expected);
    }
    path == item.href || (item.href != "/" && path.starts_with(&format!("{}/", item.href)))
}

/// Select only the most specific leaf, so create/detail pages do not highlight
/// multiple entries. Parent expansion follows that leaf, never a URL prefix.
pub fn active_nav_item<'a>(
    items: &'a [SidebarItem],
    current_path: &str,
) -> Option<&'a SidebarItem> {
    items
        .iter()
        .filter_map(|item| match &item.children {
            Some(children) => active_nav_item(children, current_path),
            None => is_child_active(item, current_path, None).then_some(item),
        })
        .max_by_key(|item| item.href.len())
}

/// Minimal percent-encoder for query values (only encodes the chars that
/// matter for `return_url=` round-tripping through the Dioxus SSR
/// template). Kept local so we don't pull in the `percent-encoding` crate.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
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
    #[allow(unused_variables)]
    header: Option<String>,
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

    fn rendered_sidebar(path: &str) -> String {
        dioxus_ssr::render_element(rsx! {
            AdminSidebar {
                current_path: path.to_string(),
                is_authenticated: true,
                items: None,
                default_expanded: None,
                class_name: None,
                id: None,
            }
        })
    }

    #[test]
    fn is_child_active_matches_tabbed_route() {
        let item = SidebarItem {
            id: "set-general".into(),
            label: "General".into(),
            href: "/settings".into(),
            icon: "globe".into(),
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
            id: "wm-wallets".into(),
            label: "Wallets".into(),
            href: "/wallet-management/wallets".into(),
            icon: "wallet".into(),
            ..Default::default()
        };
        assert!(is_child_active(&item, "/wallet-management/wallets", None));
        assert!(is_child_active(
            &item,
            "/wallet-management/wallets/123",
            None
        ));
        assert!(is_child_active(&item, "/wallet-management/access", None));
        assert!(!is_child_active(
            &item,
            "/wallet-management/wallets-other",
            None
        ));
    }

    #[test]
    fn urlencode_keeps_unreserved_chars() {
        assert_eq!(
            urlencode("/wallet-management/access"),
            "%2Fwallet-management%2Faccess"
        );
        assert_eq!(urlencode("abc-123_X.~"), "abc-123_X.~");
    }

    #[test]
    fn ssr_emits_collapsed_children_for_progressive_enhancement() {
        let html = rendered_sidebar("/audit-log");
        assert!(html.contains("data-epsx-action=\"toggle-nav\""));
        assert!(html.contains("aria-controls=\"sidebar-children-wallet-management\""));
        let child_start = html
            .find("id=\"sidebar-children-wallet-management\"")
            .expect("wallet child list must be rendered");
        let child_tag_end = html[child_start..]
            .find('>')
            .map(|offset| child_start + offset)
            .expect("wallet child list must have an opening tag");
        let child_tag = &html[child_start..child_tag_end];
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(child_tag.contains(" hidden"));
        assert!(html.contains("admin-nav-chevron"));
        assert!(!html.contains("admin-sidebar-logout"));
    }

    #[test]
    fn ssr_expands_parent_for_the_active_route() {
        let html = rendered_sidebar("/wallet-management/access");
        assert!(html.contains("aria-expanded=\"true\""));
        let child_start = html
            .find("id=\"sidebar-children-wallet-management\"")
            .expect("wallet child list must be rendered");
        let child_html = &html[child_start..];
        assert!(child_html.starts_with(
            "id=\"sidebar-children-wallet-management\" role=\"list\" aria-hidden=\"false\""
        ));
    }
    #[test]
    fn navigation_inventory_has_unique_routable_destinations() {
        let mut ids = std::collections::HashSet::new();
        let mut destinations = std::collections::HashSet::new();
        for parent in DEFAULT_NAV_ITEMS.iter() {
            assert!(ids.insert(parent.id.clone()));
            for item in parent
                .children
                .as_deref()
                .unwrap_or(std::slice::from_ref(parent))
            {
                if item.id != parent.id {
                    assert!(ids.insert(item.id.clone()));
                }
                let href = item.destination();
                assert!(
                    destinations.insert(href.clone()),
                    "Duplicate destination: {href}"
                );
                assert!(
                    href.parse::<crate::routes::AdminRoute>().is_ok(),
                    "Missing route: {href}"
                );
            }
        }
    }

    #[test]
    fn active_destination_follows_tabs_defaults_details_and_aliases() {
        for (path, id) in [
            ("/payments", "pay-payments"),
            ("/payments?tab=user-access&page=2", "wm-subscriptions"),
            ("/payments?tab=payment-links", "pay-links"),
            ("/payments/epsx/123", "pay-purchases"),
            ("/pay/escrows/123", "pay-escrows"),
            ("/pay/merchant-escrows/123", "pay-merchant-escrows"),
            ("/settings", "set-general"),
            ("/admin/settings?tab=security#panel", "set-security"),
            ("/settings?tab=%61ppearance", "set-appearance"),
            ("/developer-portal", "dev-overview"),
            ("/developer-portal?tab=keys", "dev-keys"),
            ("/developer-portal/api-keys/create", "dev-create"),
            ("/admin/wallet-management/access/plans/123", "wm-plans"),
            ("/plans/123", "wm-plans"),
            ("/wallet-management/access", "wm-wallets"),
            ("/wallet-management/0x123", "wm-wallets"),
            ("/wallet-management/wallets/0x123/disable", "wm-wallets"),
            ("/news/create", "news-create"),
            ("/news/123/edit", "news"),
            ("/media", "media"),
            ("/notifications", "notif-manage"),
            ("/admin/dashboard", "dashboard"),
        ] {
            assert_eq!(
                active_nav_item(&DEFAULT_NAV_ITEMS, path).map(|item| item.id.as_str()),
                Some(id),
                "{path}"
            );
            let html = rendered_sidebar(path);
            assert_eq!(html.matches("aria-current=\"page\"").count(), 1, "{path}");
        }
        for path in [
            "/payments-other",
            "/settings?tab=invalid",
            "/administer",
            "/unknown",
        ] {
            assert!(
                active_nav_item(&DEFAULT_NAV_ITEMS, path).is_none(),
                "{path}"
            );
        }
    }

    #[test]
    fn regrouped_destinations_expand_their_actual_parent() {
        for (path, parent) in [
            ("/plans", "wallet-management"),
            ("/payments?tab=user-access", "wallet-management"),
            ("/pay/escrows", "payments"),
            ("/pay/merchant-escrows", "payments"),
            ("/media", "content"),
            ("/audit-log", "settings"),
        ] {
            let html = rendered_sidebar(path);
            let marker = format!("id=\"sidebar-children-{parent}\"");
            let start = html.find(&marker).unwrap();
            let opening_tag = html[start..].split('>').next().unwrap();
            assert!(
                opening_tag.contains("aria-hidden=\"false\""),
                "{path}: {opening_tag}"
            );
            assert!(!opening_tag.contains(" hidden"), "{path}: {opening_tag}");
            assert_eq!(html.matches("aria-expanded=\"true\"").count(), 1, "{path}");
        }
    }
}
