//! Sub-components extracted from `pages/notifications.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Eight named sub-components: `RenderNotifications`,
//! `NotificationListSection`, `NotificationRow`,
//! `BrowserNotificationsPrompt`, `BrowserPermissionBadge`,
//! `ToggleRow`, `SwitchInput`, `NotificationSettingsSection`. Also
//! the `Notification` data type (made `pub` for module surface).

use crate::auth::AuthGate;
use crate::feedback::*;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::pages::PageContext;
use crate::primitives::*;

use dioxus::prelude::*;
use serde::Deserialize;

/// A single notification. Mirrors the source's `Notification` type.
#[derive(Clone, Debug, PartialEq, Default, Deserialize)]
pub struct Notification {
    #[serde(default)] pub id: String,
    #[serde(default)] pub title: String,
    #[serde(default)] pub body: String,
    #[serde(default)] pub kind: String,
    #[serde(default)] pub priority: String,
    #[serde(default)] pub read: bool,
    #[serde(default)] pub created_at: String,
    #[serde(default)] pub action_url: Option<String>,
    #[serde(default)] pub action_label: Option<String>,
}

/// Page-level orchestrator for the `/notifications` route.
#[component]
pub fn RenderNotifications(ctx: PageContext) -> Element {
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
                    NotificationListSection { items: items.clone() }
                    BrowserNotificationsPrompt {}
                    NotificationSettingsSection {}
                }
            }
        }
    }
}

/// Paginated list of notifications with type icons and
/// read/unread state.
#[component]
pub fn NotificationListSection(items: Vec<Notification>) -> Element {
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

/// One row in the notification list.
#[component]
pub fn NotificationRow(n: Notification) -> Element {
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

/// "Allow browser notifications" CTA.
#[component]
pub fn BrowserNotificationsPrompt() -> Element {
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

/// Permission badge for the browser-notifications card.
#[component]
pub fn BrowserPermissionBadge(permission: String) -> Element {
    let (label, class) = match permission.as_str() {
        "granted" => ("Granted", "permission-badge permission-badge-granted"),
        "denied" => ("Blocked", "permission-badge permission-badge-denied"),
        _ => ("Default", "permission-badge permission-badge-default"),
    };
    rsx! { span { class: "{class}", "{label}" } }
}

/// One row in the per-type toggle list.
#[component]
pub fn ToggleRow(label: String, checked: bool) -> Element {
    rsx! {
        div { class: "browser-notifications-toggle-row",
            span { "{label}" }
            SwitchInput { checked: checked, label: None }
        }
    }
}

/// Local switch wrapper. The existing `<Switch>` primitive
/// requires a label; this exposes a label-less variant.
#[component]
pub fn SwitchInput(checked: bool, label: Option<String>) -> Element {
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

/// Per-type notification settings.
#[component]
pub fn NotificationSettingsSection() -> Element {
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
                div { class: "notification-settings-row notification-settings-row-master",
                    span { "Enable Notifications" }
                    SwitchInput { checked: *enabled.read(), label: None }
                }
                if *enabled.read() {
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

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// notifications sub-components.
    #[test]
    fn notifications_subcomponents_render_smoke() {
        // NotificationListSection (with items)
        let items = sample_notifications();
        let el = rsx! { NotificationListSection { items } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("notifications-list"), "NotificationListSection missing section-marker");
        assert!(html.contains("Payment received"));

        // NotificationListSection (empty)
        let el = rsx! { NotificationListSection { items: vec![] } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("all caught up"));

        // NotificationRow
        let n = Notification {
            id: "n1".to_string(),
            title: "Test".to_string(),
            body: "Body".to_string(),
            kind: "payment".to_string(),
            priority: "normal".to_string(),
            read: false,
            created_at: "now".to_string(),
            action_url: None,
            action_label: None,
        };
        let el = rsx! { NotificationRow { n } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("notification-row-unread"));
        assert!(html.contains("notification-icon-payment"));

        // BrowserNotificationsPrompt
        let el = rsx! { BrowserNotificationsPrompt {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("browser-notifications"), "BrowserNotificationsPrompt missing section-marker");
        assert!(html.contains("permission-badge"));

        // NotificationSettingsSection
        let el = rsx! { NotificationSettingsSection {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("notification-settings"), "NotificationSettingsSection missing section-marker");
        assert!(html.contains("Notification Settings"));
    }
}
