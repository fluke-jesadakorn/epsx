//! Admin settings sub-components — 1:1 mirror of the Next.js source's
//! `apps-old/admin-frontend/components/admin/settings-dashboard.tsx`
//! (480 LoC) and the developer-portal/api-keys page (referenced by the
//! settings dashboard).
//!
//! Section markers (mirrored 1:1 from `pages/admin_pages/settings.rs`):
//!   - `settings-dashboard`     → `SettingsDashboard`
//!   - `api-keys-list`          → `ApiKeysList`
//!   - `email-settings`         → `EmailSettings`
//!   - `notification-settings`  → `NotificationSettings`
//!   - `session-management`     → `SessionManagement`
//!
//! Wave 6C Track C — extracted from the Wave 6B Track A port of the
//! settings page.

use crate::data_table::{Column, DataTable, Row};
use crate::primitives::*;

use dioxus::prelude::*;

// ============================================================================
// SettingsDashboard — outer wrapper with the global save/reset control bar.
// Source: `settings-dashboard.tsx` lines 438-468 (the "Reset Logic" +
// "Deploy Update" control bar).
// ============================================================================

#[component]
pub fn SettingsDashboard() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c settings-dashboard ===
        div { class: "settings-dashboard flex items-center justify-between gap-4 p-4 rounded-xl bg-card border border-border/20 shadow-xl",
            div {
                h1 { class: "text-2xl font-bold", "Settings Nexus" }
                p { class: "text-muted-foreground text-sm", "Universal configuration interface for security, appearance, and system protocols" }
            }
            div { class: "flex items-center gap-3",
                button { class: "btn btn-outline", r#type: "button", Icon { name: "rotate-ccw".to_string(), size: Some(14) } " Reset Logic" }
                button { class: "btn btn-primary", r#type: "button", Icon { name: "check".to_string(), size: Some(14) } " Deploy Update" }
            }
        }
    }
}

// ============================================================================
// EmailSettings — SMTP / sender / platform designation card.
// Source: the `GeneralSettings` block of `settings-dashboard.tsx`
// (lines 45-119).
// ============================================================================

#[component]
pub fn EmailSettings() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c email-settings ===
        div { class: "card card-glass email-settings",
            div { class: "card-header",
                div { class: "card-icon", Icon { name: "mail".to_string(), size: Some(20) } }
                div { class: "flex-1",
                    h3 { class: "card-title", "Email & system" }
                    p { class: "card-description text-xs text-muted-foreground", "SMTP, sender, and platform designation" }
                }
            }
            div { class: "card-body space-y-4",
                div { class: "field",
                    label { class: "field-label", "System designation" }
                    input { class: "input", r#type: "text", value: "EPSX Production" }
                }
                div { class: "field",
                    label { class: "field-label", "Authority email" }
                    input { class: "input", r#type: "email", value: "admin@epsx.io" }
                }
                div { class: "field",
                    label { class: "field-label", "SMTP host" }
                    input { class: "input", r#type: "text", value: "smtp.sendgrid.net" }
                }
                div { class: "grid grid-cols-2 gap-4",
                    div { class: "field",
                        label { class: "field-label", "SMTP port" }
                        input { class: "input", r#type: "number", value: "587" }
                    }
                    div { class: "field",
                        label { class: "field-label", "TLS" }
                        select { class: "input",
                            option { value: "starttls", "STARTTLS" }
                            option { value: "tls", "TLS" }
                            option { value: "none", "None" }
                        }
                    }
                }
                FormActions {
                    button { class: "btn btn-primary", r#type: "submit", "Save" }
                }
            }
        }
    }
}

// ============================================================================
// NotificationSettings — notification channel toggles.
// Source: the `NotificationSettings` block of `settings-dashboard.tsx`
// (lines 121-168).
// ============================================================================

#[component]
pub fn NotificationSettings() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c notification-settings ===
        div { class: "card card-glass notification-settings",
            div { class: "card-header",
                div { class: "card-icon", Icon { name: "bell".to_string(), size: Some(20) } }
                div { class: "flex-1",
                    h3 { class: "card-title", "Notification channels" }
                    p { class: "card-description text-xs text-muted-foreground", "Signal processing — choose which channels fire" }
                }
            }
            div { class: "card-body grid grid-cols-1 md:grid-cols-2 gap-3",
                NotificationToggle { name: "Email alerts".to_string(), on: true }
                NotificationToggle { name: "Push notifications".to_string(), on: true }
                NotificationToggle { name: "Webhook delivery".to_string(), on: false }
                NotificationToggle { name: "In-app banners".to_string(), on: true }
                NotificationToggle { name: "Admin digest".to_string(), on: true }
                NotificationToggle { name: "Weekly report".to_string(), on: false }
            }
        }
    }
}

#[component]
fn NotificationToggle(name: String, on: bool) -> Element {
    rsx! {
        div { class: "flex items-center justify-between p-3 rounded-xl bg-muted/30 border border-border/40 hover:bg-muted/50 transition-all",
            div {
                span { class: "text-sm font-bold", "{name}" }
                p { class: "text-[10px] font-bold text-muted-foreground uppercase opacity-50 mt-0.5", "Active broadcast channel" }
            }
            ToggleSwitch { on }
        }
    }
}

#[component]
fn ToggleSwitch(on: bool) -> Element {
    let bg = if on { "bg-primary" } else { "bg-muted/30" };
    let pos = if on { "translate-x-[40px]" } else { "translate-x-0" };
    rsx! {
        button {
            class: format!("relative w-20 h-10 rounded-full transition-all duration-300 {bg}"),
            r#type: "button",
            "aria-pressed": "{on}",
            div { class: format!("absolute top-1.5 left-1.5 w-7 h-7 rounded-full bg-white transition-transform duration-300 {pos}") }
        }
    }
}

// ============================================================================
// ApiKeysList — API keys table (assembled from the TS source's
// `developer-portal/api-keys/page.tsx` + the settings-dashboard
// shortcut). Renders a 5-col table (Name, Key, Scope, Last used, Created).
// ============================================================================

#[component]
pub fn ApiKeysList() -> Element {
    let columns = vec![
        Column { key: "name".into(), label: "Name".into(), sortable: true, align: crate::primitives::data_table::Align::Left, width: Some("25%".into()), class_name: None },
        Column { key: "key".into(), label: "Key".into(), sortable: false, align: crate::primitives::data_table::Align::Left, width: Some("30%".into()), class_name: None },
        Column { key: "scope".into(), label: "Scope".into(), sortable: true, align: crate::primitives::data_table::Align::Center, width: Some("15%".into()), class_name: None },
        Column { key: "last".into(), label: "Last used".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
        Column { key: "created".into(), label: "Created".into(), sortable: true, align: crate::primitives::data_table::Align::Right, width: Some("15%".into()), class_name: None },
    ];
    let rows = vec![
        Row { id: "k1".into(), cells: vec!["Production read-only".into(), "epx_live_…3a4f".into(), "read".into(), "2 min ago".into(), "2024-01-15".into()] },
        Row { id: "k2".into(), cells: vec!["Webhook delivery".into(), "epx_live_…8b2c".into(), "write".into(), "12 min ago".into(), "2024-03-22".into()] },
        Row { id: "k3".into(), cells: vec!["Indexer sync".into(), "epx_live_…9d1e".into(), "read".into(), "1 hour ago".into(), "2024-05-04".into()] },
    ];
    rsx! {
        // === wave6b-admin-pages-depth-track-c api-keys-list ===
        div { class: "card card-glass api-keys-list",
            div { class: "card-header flex items-center justify-between",
                div { class: "card-icon", Icon { name: "key".to_string(), size: Some(20) } }
                div { class: "flex-1",
                    h3 { class: "card-title", "API keys" }
                    p { class: "card-description text-xs text-muted-foreground", "Active API keys for the platform" }
                }
                button { class: "btn btn-sm btn-primary", r#type: "button", Icon { name: "plus".to_string(), size: Some(14) } " New" }
            }
            div { class: "card-body p-0",
                DataTable { columns, rows, striped: true, page_size: 10, filter_placeholder: Some("Filter by name, scope, or key…".to_string()) }
            }
        }
    }
}

// ============================================================================
// SessionManagement — auto-lock duration + active sessions list.
// Source: the `SecuritySettings` block of `settings-dashboard.tsx`
// (lines 170-210) + the active sessions table.
// ============================================================================

#[component]
pub fn SessionManagement() -> Element {
    rsx! {
        // === wave6b-admin-pages-depth-track-c session-management ===
        div { class: "card card-glass session-management",
            div { class: "card-header",
                div { class: "card-icon", Icon { name: "shield".to_string(), size: Some(20) } }
                div { class: "flex-1",
                    h3 { class: "card-title", "Sessions & security" }
                    p { class: "card-description text-xs text-muted-foreground", "Vault protocols — auto-lock, active sessions, and access controls" }
                }
            }
            div { class: "card-body space-y-4",
                div { class: "field",
                    label { class: "field-label", "Auto-lock duration (minutes)" }
                    input { class: "input", r#type: "number", value: "30" }
                    p { class: "text-[10px] font-bold text-muted-foreground uppercase opacity-30 mt-1", "Recommended: 15-60 minutes for optimal security" }
                }
                div {
                    p { class: "text-[10px] font-bold uppercase tracking-widest text-muted-foreground", "Active sessions (2)" }
                    div { class: "space-y-2 mt-2",
                        SessionRow { device: "MacBook Pro · Chrome 128".to_string(), location: "Bangkok, TH", last_active: "Active now", current: true }
                        SessionRow { device: "iPhone 15 · Safari 17".to_string(), location: "Bangkok, TH", last_active: "12 min ago", current: false }
                    }
                }
                FormActions {
                    button { class: "btn btn-danger", r#type: "button", "Revoke all other sessions" }
                }
            }
        }
    }
}

#[component]
fn SessionRow(device: String, location: String, last_active: String, current: bool) -> Element {
    rsx! {
        div { class: "flex items-center justify-between p-3 rounded-lg border border-border/40 bg-muted/20",
            div { class: "min-w-0",
                div { class: "flex items-center gap-2",
                    span { class: "text-sm font-semibold", "{device}" }
                    if current {
                        span { class: "badge badge-success", "this device" }
                    }
                }
                p { class: "text-xs text-muted-foreground", "{location} · {last_active}" }
            }
            if !current {
                button { class: "btn btn-sm btn-ghost text-danger", r#type: "button", "Revoke" }
            }
        }
    }
}

// ============================================================================
// Tests — Wave 6C Track C per-sub-component smoke tests.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_string(el: dioxus::prelude::Element) -> String {
        dioxus_ssr::render_element(el)
    }

    /// `SettingsDashboard` renders the global save/reset control bar.
    #[test]
    fn settings_dashboard_renders() {
        let el = rsx! { SettingsDashboard {} };
        let html = render_to_string(el);
        assert!(html.contains("settings-dashboard"), "SettingsDashboard must emit its section marker. Got: {html}");
        assert!(html.contains("Settings Nexus"), "SettingsDashboard must render the heading. Got: {html}");
    }

    /// `ApiKeysList` renders the 5-col API keys table.
    #[test]
    fn api_keys_list_renders() {
        let el = rsx! { ApiKeysList {} };
        let html = render_to_string(el);
        assert!(html.contains("api-keys-list"), "ApiKeysList must emit its section marker. Got: {html}");
        assert!(html.contains("API keys"), "ApiKeysList must render the heading. Got: {html}");
    }

    /// `EmailSettings` renders the SMTP / sender / designation card.
    #[test]
    fn email_settings_renders() {
        let el = rsx! { EmailSettings {} };
        let html = render_to_string(el);
        assert!(html.contains("email-settings"), "EmailSettings must emit its section marker. Got: {html}");
        // The Dioxus SSR HTML-escapes `&` to `&#38;`.
        assert!(
            html.contains("Email & system") || html.contains("Email &#38; system"),
            "EmailSettings must render the heading. Got: {html}"
        );
    }

    /// `NotificationSettings` renders the channel-toggle list.
    #[test]
    fn notification_settings_renders() {
        let el = rsx! { NotificationSettings {} };
        let html = render_to_string(el);
        assert!(html.contains("notification-settings"), "NotificationSettings must emit its section marker. Got: {html}");
        assert!(html.contains("Notification channels"), "NotificationSettings must render the heading. Got: {html}");
    }

    /// `SessionManagement` renders the auto-lock + active sessions card.
    #[test]
    fn session_management_renders() {
        let el = rsx! { SessionManagement {} };
        let html = render_to_string(el);
        assert!(html.contains("session-management"), "SessionManagement must emit its section marker. Got: {html}");
        // The Dioxus SSR HTML-escapes `&` to `&#38;`.
        assert!(
            html.contains("Sessions & security") || html.contains("Sessions &#38; security"),
            "SessionManagement must render the heading. Got: {html}"
        );
    }
}
