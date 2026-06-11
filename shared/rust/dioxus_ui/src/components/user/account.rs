//! Sub-components extracted from `pages/account.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Twelve named sub-components to lift: `AccountTabButton`,
//! `ProfileTab`, `ProfileField`, `SubscriptionTab`, `FeatureCheck`,
//! `UsageTab`, `NotificationsTab`, `NotificationToggle`,
//! `ConnectedAccountsTab`, `OAuthRow`, `SessionRow`, `DangerZoneTab`.
//!
//! The page file's `RenderAccount` wrapper orchestrates them via
//! the 6-tab nav + 6 tab bodies.
//!
//! Source: `apps-old/frontend/app/account/page.tsx` (33 LoC) +
//! `components/account/account-client.tsx` (363 LoC) +
//! `access-overview.tsx` (340 LoC) + `credit-balance-widget.tsx`
//! (81 LoC) + `payment-history-tab.tsx` (293 LoC) — totaling
//! ~1,110 LoC of source.

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;

/// Small tab-button helper. Renders a styled button whose
/// `btn-primary` vs `btn-outline` class is set based on the
/// current `tab` signal.
#[component]
pub fn AccountTabButton(tab: Signal<String>, value: String, label: String, icon: String) -> Element {
    let v = value.clone();
    let active = *tab.read() == *value;
    let cls = if active { "tab tab-active" } else { "tab" };
    rsx! {
        button {
            class: "{cls}",
            role: "tab",
            "aria-selected": active,
            onclick: move |_| tab.set(v.clone()),
            Icon { name: icon, size: Some(16) }
            span { " {label}" }
        }
    }
}

// ----- ProfileTab --------------------------------------------------------------

/// Profile tab — display name, email, bio, avatar. Ported from
/// `account-client.tsx` `ProfileTab` sub-component.
#[component]
pub fn ProfileTab(user: Option<crate::auth::User>) -> Element {
    rsx! {
        div { class: "account-tab account-profile-tab",
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                div { class: "card card-glass",
                    div { class: "card-header", h3 { class: "card-title", "Avatar" } }
                    div { class: "card-body flex flex-col items-center gap-3",
                        div { class: "w-24 h-24 rounded-full bg-primary/15 flex items-center justify-center",
                            Icon { name: "user".to_string(), size: Some(40), class_name: Some("text-primary".to_string()) }
                        }
                        button { class: "btn btn-sm btn-outline", r#type: "button", "Upload new" }
                    }
                }
                div { class: "card card-glass md:col-span-2",
                    div { class: "card-header", h3 { class: "card-title", "Profile" } }
                    div { class: "card-body",
                        ProfileField { label: "Display name", value: "EPSX user".to_string() }
                        if let Some(u) = &user {
                            ProfileField { label: "Wallet address", value: u.address.clone() }
                            ProfileField { label: "Chain", value: u.chain_id.clone() }
                            ProfileField { label: "Roles", value: u.roles.join(", ") }
                        } else {
                            p { class: "text-sm text-muted-foreground", "Not signed in" }
                        }
                        div { class: "mt-3",
                            label { class: "form-label", "Bio" }
                            textarea { class: "input", rows: "3", placeholder: "Tell us about yourself" }
                        }
                        div { class: "mt-3 flex gap-2",
                            button { class: "btn btn-primary", r#type: "button", "Save changes" }
                            button { class: "btn btn-outline", r#type: "button", "Cancel" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ProfileField(label: String, value: String) -> Element {
    rsx! {
        div { class: "form-row",
            label { class: "form-label", "{label}" }
            input { class: "input", r#type: "text", value: "{value}", readonly: true }
        }
    }
}

// ----- SubscriptionTab ---------------------------------------------------------

/// Subscription tab — current plan, billing cycle, upgrade/downgrade.
#[component]
pub fn SubscriptionTab() -> Element {
    rsx! {
        div { class: "account-tab account-subscription-tab",
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                StatCard { label: "Current plan".to_string(), value: "Pro".to_string(), icon: Some("zap".to_string()) }
                StatCard { label: "Billing cycle".to_string(), value: "Monthly".to_string(), icon: Some("calendar".to_string()) }
                StatCard { label: "Next billing".to_string(), value: "Sep 15".to_string(), icon: Some("clock".to_string()) }
            }
            div { class: "card card-glass mt-4",
                div { class: "card-header", h3 { class: "card-title", "Plan features" } }
                div { class: "card-body",
                    ul { class: "space-y-2",
                        FeatureCheck { label: "Unlimited watchlist" }
                        FeatureCheck { label: "Advanced analytics" }
                        FeatureCheck { label: "Direct API access" }
                        FeatureCheck { label: "Priority support" }
                    }
                }
            }
            div { class: "flex gap-2 mt-4",
                a { class: "btn btn-primary", href: "/plans", "Upgrade" }
                a { class: "btn btn-outline", href: "/plans", "Downgrade" }
                a { class: "btn btn-outline", href: "/payment", "Payment history" }
            }
        }
    }
}

#[component]
pub fn FeatureCheck(label: String) -> Element {
    rsx! {
        li { class: "flex items-center gap-2",
            Icon { name: "check".to_string(), size: Some(16), class_name: Some("text-emerald-500".to_string()) }
            span { "{label}" }
        }
    }
}

// ----- UsageTab ----------------------------------------------------------------

/// Usage tab — API call chart + credits remaining. Ported as a
/// static chart + balance card (no live data).
#[component]
pub fn UsageTab() -> Element {
    let series = vec![Series {
        name: "API calls".to_string(),
        color: "#22d3ee".to_string(),
        points: (1..=7).map(|d| DataPoint {
            x: d as f64,
            y: (40 + d * 7) as f64,
            label: Some(format!("D{d}")),
        }).collect(),
    }];
    rsx! {
        div { class: "account-tab account-usage-tab",
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                StatCard { label: "API calls today".to_string(), value: "1,284".to_string(), icon: Some("code".to_string()) }
                StatCard { label: "Credits remaining".to_string(), value: "1,250".to_string(), icon: Some("coins".to_string()) }
                StatCard { label: "Quota used".to_string(), value: "62%".to_string(), icon: Some("pie-chart".to_string()) }
            }
            div { class: "card card-glass mt-4",
                div { class: "card-header", h3 { class: "card-title", "API calls (last 7 days)" } }
                div { class: "card-body",
                    ChartLine { series: series, width: 720, height: 220 }
                }
            }
        }
    }
}

// ----- NotificationsTab -------------------------------------------------------

/// Notifications tab — per-channel notification preferences.
/// Ported from `notification-settings-panel.tsx` (110 LoC).
#[component]
pub fn NotificationsTab() -> Element {
    let channels = vec![
        ("analytics", "Analytics alerts", "Price movements & portfolio", true),
        ("security",  "Security alerts",  "Auth & security warnings", true),
        ("account",   "Account updates",  "Profile & subscription", true),
        ("system",    "System status",    "Maintenance & features", false),
        ("marketing", "Promotions",       "News & special offers", false),
    ];
    rsx! {
        div { class: "account-tab account-notifications-tab",
            div { class: "card card-glass",
                div { class: "card-header", h3 { class: "card-title", "Notification preferences" } }
                div { class: "card-body",
                    p { class: "text-sm text-muted-foreground mb-4",
                        "Choose exactly what you want to be notified about. We'll send alerts via web push to keep you updated."
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                        for (id, label, desc, on) in channels {
                            NotificationToggle {
                                id: id.to_string(),
                                label: label.to_string(),
                                desc: desc.to_string(),
                                on: on,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn NotificationToggle(id: String, label: String, desc: String, on: bool) -> Element {
    rsx! {
        label { class: "notification-toggle-row flex items-center justify-between p-4 rounded-2xl border border-border cursor-pointer",
            input {
                class: "notification-toggle-input",
                r#type: "checkbox",
                id: "notif-{id}",
                checked: on,
            }
            div { class: "flex-1 px-3",
                div { class: "font-semibold", "{label}" }
                div { class: "text-xs text-muted-foreground", "{desc}" }
            }
        }
    }
}

// ----- ConnectedAccountsTab ---------------------------------------------------

/// Connected accounts tab — wallet, OAuth providers, sessions.
#[component]
pub fn ConnectedAccountsTab() -> Element {
    rsx! {
        div { class: "account-tab account-connected-tab",
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                div { class: "card card-glass",
                    div { class: "card-header flex justify-between items-center",
                        h3 { class: "card-title", "Wallet" }
                        span { class: "badge badge-success", "Connected" }
                    }
                    div { class: "card-body",
                        p { class: "text-sm", "0x1234…abcd" }
                        p { class: "text-xs text-muted-foreground mt-1", "Primary wallet — used for sign-in" }
                        button { class: "btn btn-sm btn-outline mt-3", r#type: "button", "Disconnect" }
                    }
                }
                div { class: "card card-glass",
                    div { class: "card-header flex justify-between items-center",
                        h3 { class: "card-title", "OAuth providers" }
                    }
                    div { class: "card-body flex flex-col gap-2",
                        OAuthRow { provider: "Google", connected: false }
                        OAuthRow { provider: "GitHub", connected: true }
                        OAuthRow { provider: "Twitter", connected: false }
                    }
                }
                div { class: "card card-glass md:col-span-2",
                    div { class: "card-header flex justify-between items-center",
                        h3 { class: "card-title", "Active sessions" }
                        button { class: "btn btn-sm btn-outline", r#type: "button", "Sign out all" }
                    }
                    div { class: "card-body p-0",
                        div { class: "table-wrap",
                            table { class: "table",
                                thead { tr { th { "Device" } th { "Location" } th { "Last active" } th { "" } } }
                                tbody {
                                    SessionRow { device: "Chrome on macOS", location: "Bangkok, TH", last_active: "Now" }
                                    SessionRow { device: "Safari on iOS", location: "Bangkok, TH", last_active: "2h ago" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn OAuthRow(provider: String, connected: bool) -> Element {
    rsx! {
        div { class: "flex items-center justify-between p-2 rounded border border-border",
            span { "{provider}" }
            if connected {
                button { class: "btn btn-sm btn-outline", r#type: "button", "Disconnect" }
            } else {
                button { class: "btn btn-sm btn-primary", r#type: "button", "Connect" }
            }
        }
    }
}

#[component]
pub fn SessionRow(device: String, location: String, last_active: String) -> Element {
    rsx! {
        tr {
            td { "{device}" }
            td { "{location}" }
            td { class: "text-xs text-muted-foreground", "{last_active}" }
            td { button { class: "btn btn-sm btn-outline", r#type: "button", "Revoke" } }
        }
    }
}

// ----- DangerZoneTab -----------------------------------------------------------

/// Danger zone tab — delete account, sign out everywhere.
#[component]
pub fn DangerZoneTab() -> Element {
    rsx! {
        div { class: "account-tab account-danger-tab",
            div { class: "card card-glass border border-danger",
                div { class: "card-header", h3 { class: "card-title text-danger", "Danger zone" } }
                div { class: "card-body",
                    p { class: "text-sm text-muted-foreground mb-4",
                        "These actions are permanent. Proceed with care."
                    }
                    div { class: "flex flex-col gap-2",
                        button { class: "btn btn-outline", r#type: "button",
                            Icon { name: "log-out".to_string(), size: Some(16) }
                            " Sign out of all sessions"
                        }
                        button { class: "btn btn-danger", r#type: "button",
                            Icon { name: "trash-2".to_string(), size: Some(16) }
                            " Delete account"
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
    use crate::auth::user::User;
    use crate::auth::user::AuthMethod;
    use crate::pages::PageContext;

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-1".to_string(),
                address: "0x1234abcd".to_string(),
                chain_id: "1".to_string(),
                roles: vec!["user".to_string()],
                email: Some("test@epsx.io".to_string()),
                tier: Some("pro".to_string()),
                permissions: vec!["profile:read".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::default(),
                display_name: Some("EPSX tester".to_string()),
            }),
            path: "/account".to_string(),
            ..Default::default()
        }
    }

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// account sub-components. Asserts each one carries its
    /// section-marker class.
    #[test]
    fn account_subcomponents_render_smoke() {
        // ProfileTab
        let el = rsx! { ProfileTab { user: None } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-profile-tab"), "ProfileTab missing section-marker");
        assert!(html.contains("Display name"));

        // ProfileField
        let el = rsx! { ProfileField { label: "Test".to_string(), value: "value".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("form-row"), "ProfileField missing class");
        assert!(html.contains("Test"));

        // FeatureCheck
        let el = rsx! { FeatureCheck { label: "feature".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("feature"));

        // SubscriptionTab
        let el = rsx! { SubscriptionTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-subscription-tab"), "SubscriptionTab missing section-marker");
        assert!(html.contains("Current plan"));

        // UsageTab
        let el = rsx! { UsageTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-usage-tab"), "UsageTab missing section-marker");
        assert!(html.contains("API calls today"));

        // NotificationsTab
        let el = rsx! { NotificationsTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-notifications-tab"), "NotificationsTab missing section-marker");
        for ch in &["Analytics alerts", "Security alerts", "Account updates", "System status", "Promotions"] {
            assert!(html.contains(ch), "NotificationsTab missing channel '{}'", ch);
        }

        // NotificationToggle
        let el = rsx! { NotificationToggle { id: "test".to_string(), label: "L".to_string(), desc: "D".to_string(), on: true } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("notification-toggle-row"), "NotificationToggle missing class");
        assert!(html.contains("id=\"notif-test\""));

        // ConnectedAccountsTab
        let el = rsx! { ConnectedAccountsTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-connected-tab"), "ConnectedAccountsTab missing section-marker");
        assert!(html.contains("Google"));
        assert!(html.contains("Chrome on macOS"));

        // OAuthRow
        let el = rsx! { OAuthRow { provider: "Google".to_string(), connected: true } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Google"));
        assert!(html.contains("Disconnect"));

        // OAuthRow disconnected
        let el = rsx! { OAuthRow { provider: "Twitter".to_string(), connected: false } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Connect"));

        // SessionRow
        let el = rsx! { SessionRow { device: "Firefox".to_string(), location: "Bangkok".to_string(), last_active: "1h".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Firefox"));

        // DangerZoneTab
        let el = rsx! { DangerZoneTab {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("account-danger-tab"), "DangerZoneTab missing section-marker");
        assert!(html.contains("Sign out of all sessions"));
        assert!(html.contains("Delete account"));
    }
}
