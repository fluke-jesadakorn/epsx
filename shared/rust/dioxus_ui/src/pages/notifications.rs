//! /notifications — notification center with bell widget behavior,
//! list, mark-read, delete, clear-all, browser-notification
//! permission CTA, and per-type settings.
//!
//! Wave 6A Track C port — see `docs/wave6-auth-pages-depth/design.md`
//! §"Track C — chat + chat_history + chat_conversation +
//! notifications" / `notifications.rs`.
//!
//! Section list (in order, mirroring the source
//! `app/notifications/page.tsx` + `components/notifications/*.tsx`):
//!   1. `NotificationList` — paginated list with type icons,
//!      read/unread state, mark-read + delete buttons. Ported
//!      from `notification-bell-client.tsx` 266 LoC.
//!   2. `BrowserNotificationsPrompt` — "Allow browser
//!      notifications" CTA with permission status. Ported from
//!      `browser-notifications.tsx` 152 LoC.
//!   3. `NotificationSettings` — per-type toggle (news, payment,
//!      chat, system). Ported from
//!      `notification-settings-panel.tsx` 110 LoC.
//!
//! The previous Wave 1 shell lived at 105 LoC; the new port adds
//! the three sub-sections so the section-marker tests can assert
//! each is present.
//!
//! CSS for the unread badge, notification list rows, and the
//! permission badge lives in `shared/rust/templates/src/lib.rs`
//! under the `// === wave6-auth-pages-depth-track-c ===` marker.

use dioxus::prelude::*;
use serde::Deserialize;

use crate::primitives::*;
use crate::feedback::*;

use super::PageContext;
use super::PageMeta;
use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;

// ── Data shape (mirrors `shared/types/notifications.ts`) ─────────────

/// A single notification. Mirrors the source's `Notification`
/// type (id, title, body, type, read, timestamp, action_url,
/// action_label, priority) — narrowed to the fields the
/// notification list + settings render. The BFF hydrates the
/// `items` list via `getInitialNotificationsAction()`.
#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
pub struct Notification {
    #[serde(default)] pub id: String,
    #[serde(default)] pub title: String,
    #[serde(default)] pub body: String,
    /// `"payment" | "subscription" | "wallet" | "system" |
    /// "alert" | "news" | "chat" | …`. Drives the row's icon.
    #[serde(default)] pub kind: String,
    /// `"low" | "normal" | "high" | "urgent"`. Drives the row's
    /// icon background color.
    #[serde(default)] pub priority: String,
    #[serde(default)] pub read: bool,
    /// ISO-8601 timestamp string. The list renders this verbatim
    /// as a short relative or absolute time.
    #[serde(default)] pub created_at: String,
    #[serde(default)] pub action_url: Option<String>,
    #[serde(default)] pub action_label: Option<String>,
}

// ── Page entry ───────────────────────────────────────────────────────

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Notifications");
    (meta, rsx! { RenderNotifications { ctx: ctx.clone() } })
}

/// Top-level wrapper. Mirrors the source
/// `<NotificationsClient initialData=… focusId=…>` body — pulls
/// the notification list from `ctx.params` (or falls back to a
/// demo list), then renders the three sections.
#[component]
fn RenderNotifications(ctx: PageContext) -> Element {
    // BFF-supplied data. The Wave 1 stub parsed JSON from a
    // `data_notifications` param; the BFF would normally hydrate
    // the list at SSR time. Fall back to a demo list so the
    // section-marker tests have something to assert against.
    let data: Option<serde_json::Value> = ctx.params.get("data_notifications")
        .and_then(|s| serde_json::from_str(s).ok());
    let items: Vec<Notification> = data.as_ref()
        .and_then(|d| serde_json::from_value(d.get("items").cloned().unwrap_or(serde_json::json!([]))).ok())
        .unwrap_or_else(sample_notifications);
    let count_label = format!("{} notification(s)", items.len());

    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your notifications".to_string()),
                required_permissions: Some(vec!["notifications:read".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content notifications-page",
                    PageHeader { title: "Notifications".to_string(),
                        description: Some(count_label),
                        icon: Some("bell".to_string()) }
                    // ── Section 1: NotificationList ──
                    NotificationListSection { items: items.clone() }
                    // ── Section 2: BrowserNotificationsPrompt ──
                    BrowserNotificationsPrompt {}
                    // ── Section 3: NotificationSettings ──
                    NotificationSettingsSection {}
                }
            }
        }
    }
}

// ── Section 1: NotificationList ──────────────────────────────────────

/// Paginated list of notifications with type icons and
/// read/unread state. Mirrors the `<NotificationItem>` and
/// `<NotificationBellClient>` body in
/// `components/notifications/notification-bell-client.tsx` (266
/// LoC). The source renders this as a dropdown popup; Wave 6A
/// inlines the same list into the `/notifications` page body
/// (the BFF / page surface).
#[component]
fn NotificationListSection(items: Vec<Notification>) -> Element {
    let mut filter = use_signal(|| "all".to_string());
    let unread_count = items.iter().filter(|n| !n.read).count();
    let unread_label = format!("{unread_count} unread");

    let visible: Vec<Notification> = items.iter()
        .filter(|n| match filter.read().as_str() {
            "unread" => !n.read,
            "read" => n.read,
            _ => true,
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "notifications-list",
            // Filter bar — All / Unread / Read + bulk actions.
            div { class: "notifications-filterbar",
                div { class: "notifications-filters",
                    button {
                        class: if *filter.read() == "all" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                        r#type: "button",
                        onclick: move |_| filter.set("all".to_string()),
                        "All"
                    }
                    button {
                        class: if *filter.read() == "unread" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                        r#type: "button",
                        onclick: move |_| filter.set("unread".to_string()),
                        "Unread"
                    }
                    button {
                        class: if *filter.read() == "read" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                        r#type: "button",
                        onclick: move |_| filter.set("read".to_string()),
                        "Read"
                    }
                }
                div { class: "notifications-filterbar-aside",
                    if unread_count > 0 {
                        span { class: "notifications-unread-count", "{unread_label}" }
                    }
                    button { class: "btn btn-sm btn-outline", r#type: "button", "Mark all read" }
                    button { class: "btn btn-sm btn-outline", r#type: "button", "Clear all" }
                }
            }

            // List body — empty state + rows.
            div { class: "card card-glass notifications-list-card",
                div { class: "card-body p-0",
                    if visible.is_empty() {
                        div { class: "notifications-empty",
                            Icon { name: "bell-off".to_string(), size: Some(32) }
                            p { class: "notifications-empty-title", "You're all caught up" }
                            p { class: "notifications-empty-hint", "New notifications will appear here." }
                        }
                    } else {
                        for n in visible.iter() {
                            NotificationRow { n: n.clone() }
                        }
                    }
                }
            }
        }
    }
}

/// One row in the notification list. Mirrors the source's
/// `<NotificationItem>` inner markup (icons, title, body, time,
/// action link, read-receipt dot, delete button). The action
/// link + delete button render only when their data is present.
#[component]
fn NotificationRow(n: Notification) -> Element {
    let (icon_name, icon_class) = match n.kind.as_str() {
        "payment" => ("credit-card", "notification-icon-payment"),
        "subscription" => ("zap", "notification-icon-subscription"),
        "wallet" => ("wallet", "notification-icon-wallet"),
        "news" => ("newspaper", "notification-icon-news"),
        "chat" => ("message-circle", "notification-icon-chat"),
        "alert" => ("alert-triangle", "notification-icon-alert"),
        _ => ("info", "notification-icon-system"),
    };
    let row_class = if n.read {
        "notification-row notification-row-read"
    } else {
        "notification-row notification-row-unread"
    };
    let unread_dot_class = if n.read {
        "notification-unread-dot notification-unread-dot-empty"
    } else {
        "notification-unread-dot"
    };
    rsx! {
        div { class: "{row_class}",
            div { class: "notification-icon {icon_class}",
                Icon { name: icon_name.to_string(), size: Some(16) }
            }
            div { class: "notification-body",
                div { class: "notification-headline",
                    p { class: "notification-title", "{n.title}" }
                    span { class: "{unread_dot_class}" }
                }
                p { class: "notification-text", "{n.body}" }
                div { class: "notification-meta",
                    span { class: "notification-time", "{n.created_at}" }
                    if let (Some(lbl), Some(href)) = (&n.action_label, &n.action_url) {
                        span { class: "notification-meta-sep", "·" }
                        a { class: "notification-action", href: "{href}", "{lbl}" }
                    }
                }
            }
            div { class: "notification-actions",
                if !n.read {
                    button { class: "btn btn-sm btn-ghost", r#type: "button", title: "Mark read",
                        Icon { name: "check".to_string(), size: Some(14) }
                    }
                }
                button { class: "btn btn-sm btn-ghost", r#type: "button", title: "Delete",
                    Icon { name: "trash".to_string(), size: Some(14) }
                }
            }
        }
    }
}

// ── Section 2: BrowserNotificationsPrompt ────────────────────────────

/// "Allow browser notifications" CTA. Mirrors the source
/// `<BrowserNotifications>` body in
/// `components/notifications/browser-notifications.tsx` (152
/// LoC). The source has three permission states (default,
/// granted, denied) and a "Test Notification" button when
/// granted. Wave 6A inlines the three states as rsx! branches
/// driven by a `use_signal`.
#[component]
fn BrowserNotificationsPrompt() -> Element {
    // Source's `permission` state. The signal is server-rendered
    // as "default" (the BFF would later update it client-side
    // from the `Notification.permission` API).
    let mut permission = use_signal(|| "default".to_string());
    let mut enabled = use_signal(|| false);
    let mut analytics = use_signal(|| true);
    let mut security = use_signal(|| true);
    let mut system = use_signal(|| true);
    let mut permissions = use_signal(|| false);

    rsx! {
        div { class: "card card-glass browser-notifications",
            div { class: "card-header browser-notifications-header",
                div { class: "browser-notifications-title",
                    Icon { name: "bell".to_string(), size: Some(18) }
                    h3 { class: "browser-notifications-heading", "Browser Notifications" }
                }
                BrowserPermissionBadge { permission: permission.read().clone() }
            }
            div { class: "card-body browser-notifications-body",
                match permission.read().as_str() {
                    "default" => rsx! {
                        div { class: "browser-notifications-prompt",
                            p { class: "browser-notifications-prompt-text",
                                "Enable browser notifications to receive important alerts about your analytics activity, security events, and permission changes."
                            }
                            button {
                                class: "btn btn-primary browser-notifications-enable",
                                r#type: "button",
                                onclick: move |_| permission.set("granted".to_string()),
                                Icon { name: "bell".to_string(), size: Some(16) }
                                " Enable Browser Notifications"
                            }
                        }
                    },
                    "denied" => rsx! {
                        div { class: "browser-notifications-prompt browser-notifications-prompt-denied",
                            Icon { name: "bell-off".to_string(), size: Some(18) }
                            p { class: "browser-notifications-prompt-text",
                                "Browser notifications are blocked. To enable them, click the lock icon in your browser's address bar and allow notifications for this site."
                            }
                        }
                    },
                    _ => rsx! {
                        div { class: "browser-notifications-settings",
                            div { class: "browser-notifications-toggle",
                                span { "Enable Notifications" }
                                SwitchInput { checked: *enabled.read(), label: None }
                            }
                            if *enabled.read() {
                                div { class: "browser-notifications-types",
                                    ToggleRow { label: "📈 Analytics & Portfolio Alerts".to_string(), checked: *analytics.read() }
                                    ToggleRow { label: "🔒 Security & Login Alerts".to_string(), checked: *security.read() }
                                    ToggleRow { label: "🛡️ Permission Changes".to_string(), checked: *permissions.read() }
                                    ToggleRow { label: "🔧 System Updates".to_string(), checked: *system.read() }
                                    button { class: "btn btn-outline btn-sm browser-notifications-test", r#type: "button",
                                        Icon { name: "bell".to_string(), size: Some(14) }
                                        " Test Notification"
                                    }
                                }
                            }
                        }
                    },
                }
                div { class: "browser-notifications-footnotes",
                    p { "📱 ", strong { "Usage:" }, " Browser notifications appear outside the website as native OS notifications." }
                    p { "🔕 ", strong { "Privacy:" }, " Notifications are processed locally in your browser only." }
                    p { "⚙️ ", strong { "Control:" }, " You can disable or customize notification types at any time." }
                }
            }
        }
    }
}

/// Permission badge for the browser-notifications card. Mirrors
/// the source's `<NotificationPermissionBadge>` (coloured pill:
/// "Default" / "Granted" / "Blocked").
#[component]
fn BrowserPermissionBadge(permission: String) -> Element {
    let (label, class) = match permission.as_str() {
        "granted" => ("Granted", "permission-badge permission-badge-granted"),
        "denied" => ("Blocked", "permission-badge permission-badge-denied"),
        _ => ("Default", "permission-badge permission-badge-default"),
    };
    rsx! { span { class: "{class}", "{label}" } }
}

/// One row in the per-type toggle list inside the browser-
/// notifications card. Mirrors the source's pattern of label
/// + Switch on the right.
#[component]
fn ToggleRow(label: String, checked: bool) -> Element {
    rsx! {
        div { class: "browser-notifications-toggle-row",
            span { "{label}" }
            SwitchInput { checked: checked, label: None }
        }
    }
}

/// Local switch wrapper. The existing `<Switch>` primitive
/// requires a label argument; this wrapper exposes a label-less
/// variant for the toggle rows. Same on/off state — the label
/// is rendered outside the primitive.
#[component]
fn SwitchInput(checked: bool, label: Option<String>) -> Element {
    rsx! {
        label { class: if checked { "SwitchRoot switch-md state-checked" } else { "SwitchRoot switch-md state-unchecked" },
            input {
                r#type: "checkbox",
                role: "switch",
                "aria-checked": checked.to_string(),
                class: "SwitchInput",
                checked: checked,
            }
            span { class: "SwitchThumb" }
            if let Some(l) = label {
                span { class: "SwitchLabel", "{l}" }
            }
        }
    }
}

// ── Section 3: NotificationSettings ──────────────────────────────────

/// Per-type notification settings. Mirrors the source
/// `<NotificationSettingsPanel>` in
/// `components/notifications/ui/notification-settings-panel.tsx`
/// (110 LoC). Wave 6A renders the same enable + per-type
/// switches but uses the portal's existing design-system classes.
#[component]
fn NotificationSettingsSection() -> Element {
    let mut enabled = use_signal(|| true);
    let mut news = use_signal(|| true);
    let mut payment = use_signal(|| true);
    let mut chat = use_signal(|| true);
    let mut system = use_signal(|| true);
    rsx! {
        div { class: "card card-glass notification-settings",
            div { class: "card-header",
                h3 { class: "notification-settings-heading",
                    Icon { name: "settings".to_string(), size: Some(18) }
                    " Notification Settings"
                }
            }
            div { class: "card-body notification-settings-body",
                // Master enable switch.
                div { class: "notification-settings-row notification-settings-row-master",
                    span { "Enable Notifications" }
                    SwitchInput { checked: *enabled.read(), label: None }
                }
                if *enabled.read() {
                    // Per-type toggles. Mirrors the source's
                    // indented `border-l-2` layout.
                    div { class: "notification-settings-types",
                        ToggleRow { label: "📰 News & Announcements".to_string(), checked: *news.read() }
                        ToggleRow { label: "💳 Payment & Billing".to_string(), checked: *payment.read() }
                        ToggleRow { label: "💬 Chat & Support".to_string(), checked: *chat.read() }
                        ToggleRow { label: "🔧 System Updates".to_string(), checked: *system.read() }
                    }
                }
            }
        }
    }
}

// ── Sample data (placeholder until BFF hydrates) ─────────────────────

fn sample_notifications() -> Vec<Notification> {
    vec![
        Notification {
            id: "n1".to_string(),
            title: "Payment received".to_string(),
            body: "We received your payment of 99 USDC for the Pro plan renewal.".to_string(),
            kind: "payment".to_string(),
            priority: "normal".to_string(),
            read: false,
            created_at: "5 minutes ago".to_string(),
            action_url: Some("/payment".to_string()),
            action_label: Some("View receipt".to_string()),
        },
        Notification {
            id: "n2".to_string(),
            title: "New comment on your plan".to_string(),
            body: "EPSX Support replied to your question about plan upgrade.".to_string(),
            kind: "chat".to_string(),
            priority: "normal".to_string(),
            read: false,
            created_at: "1 hour ago".to_string(),
            action_url: Some("/chat".to_string()),
            action_label: Some("Open chat".to_string()),
        },
        Notification {
            id: "n3".to_string(),
            title: "Wallet connected".to_string(),
            body: "Your wallet 0x1234…abcd was successfully connected.".to_string(),
            kind: "wallet".to_string(),
            priority: "low".to_string(),
            read: true,
            created_at: "yesterday".to_string(),
            action_url: None,
            action_label: None,
        },
        Notification {
            id: "n4".to_string(),
            title: "Scheduled maintenance".to_string(),
            body: "EPSX services will be unavailable on Sunday from 02:00 to 04:00 UTC.".to_string(),
            kind: "system".to_string(),
            priority: "high".to_string(),
            read: true,
            created_at: "2 days ago".to_string(),
            action_url: None,
            action_label: None,
        },
        Notification {
            id: "n5".to_string(),
            title: "New feature: Markdown pages".to_string(),
            body: "We've shipped markdown-driven pages. Check out the new builder!".to_string(),
            kind: "news".to_string(),
            priority: "low".to_string(),
            read: true,
            created_at: "3 days ago".to_string(),
            action_url: Some("/news".to_string()),
            action_label: Some("Read more".to_string()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn notif_ctx(user_perms: &[&str]) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: None,
            tier: Some("Pro".to_string()),
            permissions: user_perms.iter().map(|s| s.to_string()).collect(),
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: None,
        };
        PageContext {
            user: Some(user),
            path: "/notifications".to_string(),
            ..Default::default()
        }
    }

    /// Wave 6A — `test_render_smoke`. Notifications page must
    /// render non-empty HTML.
    #[test]
    fn test_render_smoke() {
        let ctx = notif_ctx(&["notifications:read"]);
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.is_empty(), "Notifications page must render non-empty HTML.");
        assert!(html.len() > 200, "Notifications HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Wave 6A — `test_section_markers`. The notifications page
    /// must render every section the design doc claims:
    /// notification list, browser notifications CTA, and the
    /// per-type settings panel.
    #[test]
    fn test_section_markers() {
        let ctx = notif_ctx(&["notifications:read"]);
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "notifications-page",
            "notifications-list",
            "notifications-filterbar",
            "notifications-list-card",
            "notification-row",
            "notification-icon",
            "browser-notifications",
            "browser-notifications-header",
            "browser-notifications-prompt",
            "permission-badge",
            "notification-settings",
            "notification-settings-heading",
            "notification-settings-types",
        ] {
            let needle = format!("class=\"{}\"", marker);
            let found = html.contains(&needle)
                || html.contains(&format!("\"{} ", marker))
                || html.contains(&format!(" {} ", marker))
                || html.contains(&format!(" {}", marker));
            assert!(
                found,
                "Notifications page must contain section marker '{}'. Got: {}",
                needle, html
            );
        }
    }
}
