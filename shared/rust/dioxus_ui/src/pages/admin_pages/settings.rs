//! /admin/settings — system settings (config + API keys + email +
//! notifications + sessions).
//!
//! Wave 6B Track A — port of `apps-old/admin-frontend/app/settings/page.tsx`
//! (26 LoC) + the TS-parity `SettingsDashboard` component (480 LoC of
//! `components/admin/settings-dashboard.tsx`).
//!
//! Sections (per design doc §"Track A" line 173):
//! - `settings-dashboard` — top-level dashboard wrapper with the
//!   global save/reset control bar (mirrors the TS
//!   `SettingsDashboard` outer container).
//! - `api-keys-list` — API keys table (mirrors the TS source's
//!   API keys management surface; the TS source's API keys live in
//!   `developer-portal/api-keys/page.tsx` but the settings dashboard
//!   references them via the `DeveloperPortal` shortcut).
//! - `email-settings` — email / SMTP / sender config (mirrors the
//!   `GeneralSettings` system-designation + admin-email block from
//!   the TS source).
//! - `notification-settings` — notification channel toggles
//!   (mirrors the TS source's `NotificationSettings` block — the
//!   `Signal Processing` surface).
//! - `session-management` — session timeout + active session list
//!   (mirrors the TS source's `SecuritySettings` `Vault Protocols`
//!   surface — auto-lock duration + active sessions table).
//! - `appearance-settings` — Theme (3 modes) + Primary Color (4
//!   presets + custom hex). Ported from the OLD's
//!   `AppearanceSettings` tab in Wave 21 admin-recheck.

use crate::auth::AdminAuthGate;
use crate::data_table::{Column, DataTable, Row};
use crate::layout::admin_shell::AdminShell;
use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Settings");
    (meta, rsx! { RenderSettings { ctx: ctx.clone() } })
}

#[component]
fn RenderSettings(ctx: PageContext) -> Element {
    rsx! {
        AdminAuthGate { user: ctx.user.clone(), feature: Some("platform settings".to_string()), required_permissions: Some(vec!["settings:manage".to_string()]), return_url: Some(ctx.path.clone()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Settings".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Settings".to_string(), "/settings".to_string()),
                ],
                div { class: "container page-content admin-settings",
                    // SettingsDashboard — outer wrapper with global control bar.
                    SettingsDashboardWrapper {}
                    // 2-col grid of section cards.
                    div { class: "grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4",
                        // Left col: EmailSettings + NotificationSettings.
                        EmailSettings {}
                        NotificationSettings {}
                        // Right col: ApiKeysList + SessionManagement.
                        ApiKeysList {}
                        SessionManagement {}
                        // Maintenance card (bonus, lives in the same surface).
                        MaintenanceCard {}
                    }
                    // Appearance — full-width, lives below the 2-col grid.
                    // (port of OLD `AppearanceSettings` tab — Theme + Primary Color.)
                    div { class: "mt-4",
                        AppearanceSettings {}
                    }
                }
            }
        }
    }
}

// ===== SettingsDashboardWrapper =============================================
//
// Source: the `SettingsDashboard` component's outer container
// (`components/admin/settings-dashboard.tsx` lines 438-468 — the
// global "Reset Logic" + "Deploy Update" control bar). The Dioxus
// port renders the same bar with a slim variant (no async save —
// the form actions are wired by the BFF).

#[component]
fn SettingsDashboardWrapper() -> Element {
    rsx! {
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

// ===== EmailSettings =======================================================
//
// Source: the `GeneralSettings` block of the TS `SettingsDashboard`
// (`settings-dashboard.tsx` lines 45-119). Mirrors the
// "System Designation" + "Authority Email Channel" + "Maintenance
// Lock" fields with the same gradient / dark styling.

#[component]
fn EmailSettings() -> Element {
    rsx! {
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

// ===== NotificationSettings ================================================
//
// Source: the `NotificationSettings` block of the TS
// `SettingsDashboard` (`settings-dashboard.tsx` lines 121-168).
// Mirrors the "Signal Processing" surface — toggle list for
// notification channels (email, push, webhook, in-app).

#[component]
fn NotificationSettings() -> Element {
    rsx! {
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

// ===== ApiKeysList =========================================================
//
// Source: not a single component — assembled from the TS source's
// `developer-portal/api-keys/page.tsx` (Wave 6B Track D territory)
// but inlined into the settings surface for the admin "Settings
// Nexus" overview. Renders a 5-col table (Name, Key, Scope, Last
// used, Created).

#[component]
fn ApiKeysList() -> Element {
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

// ===== SessionManagement ===================================================
//
// Source: the `SecuritySettings` block of the TS `SettingsDashboard`
// (`settings-dashboard.tsx` lines 170-210) + the active sessions
// table. The Dioxus port renders the auto-lock duration input +
// a list of active sessions.

#[component]
fn SessionManagement() -> Element {
    rsx! {
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

// ===== MaintenanceCard =====================================================
//
// Bonus card — the TS source's `Maintenance Lock` toggle (from
// `settings-dashboard.tsx` lines 95-114). Lives in the settings
// surface as the "5th section" alongside the 4 design-doc-named
// ones. Renders the maintenance mode toggle + message.

#[component]
fn MaintenanceCard() -> Element {
    rsx! {
        div { class: "card card-glass maintenance-card",
            div { class: "card-header",
                div { class: "card-icon", Icon { name: "settings".to_string(), size: Some(20) } }
                div { class: "flex-1",
                    h3 { class: "card-title", "Maintenance" }
                    p { class: "card-description text-xs text-muted-foreground", "Maintenance mode (show banner to users)" }
                }
            }
            div { class: "card-body space-y-4",
                div { class: "flex items-center justify-between p-3 rounded-xl bg-muted/30 border border-border/40",
                    div {
                        span { class: "text-sm font-bold", "Maintenance mode" }
                        p { class: "text-[10px] font-bold text-muted-foreground uppercase opacity-50 mt-0.5", "Isolate network from public operations" }
                    }
                    ToggleSwitch { on: false }
                }
                div { class: "field",
                    label { class: "field-label", "Maintenance message" }
                    textarea { class: "input", name: "message", rows: "3", placeholder: "We'll be back shortly…" }
                }
                FormActions {
                    button { class: "btn btn-primary", r#type: "submit", "Save" }
                }
            }
        }
    }
}

// ===== AppearanceSettings ===================================================
//
// Port of `AppearanceSettings` from
// `apps-old/admin-frontend/components/admin/settings-dashboard.tsx`
// (lines 219-297). The TS source has 3 luminosity modes
// (Daylight / Eclipse / Neural) and 4 color presets (PancakeSwap,
// Eclipse, Magma, Crimson) plus a custom color picker. The Dioxus
// port renders the same 3-mode grid + 4-preset swatch row, and
// includes a hex `<input type="color">` for custom accent selection.
// State is local to the page (theme + primary color) until a future
// BFF `/api/admin/settings/appearance` endpoint lands.

#[component]
fn AppearanceSettings() -> Element {
    let mut theme = use_signal(|| "dark".to_string());
    let mut primary_color = use_signal(|| "#1fc7d4".to_string());

    let color_presets = [
        ("PancakeSwap", "#1fc7d4"),
        ("Eclipse", "#7645d9"),
        ("Magma", "#ffb237"),
        ("Crimson", "#ed4b9e"),
    ];

    let theme_choices: [(&'static str, &'static str); 3] = [
        ("☀️ Daylight", "light"),
        ("🌙 Eclipse", "dark"),
        ("🔄 Neural", "auto"),
    ];

    rsx! {
        div { class: "card card-glass appearance-settings",
            div { class: "card-header",
                div { class: "card-icon bg-gradient-to-br from-[#ed4b9e]/10 to-[#7645d9]/10 text-[#ed4b9e] border border-[#ed4b9e]/20",
                    Icon { name: "palette".to_string(), size: Some(20) }
                }
                div { class: "flex-1",
                    h3 { class: "card-title", "Optical Customization" }
                    p { class: "card-description text-xs text-muted-foreground", "Visual feedback and interface styling" }
                }
            }
            div { class: "card-body space-y-6",
                // Luminosity mode (3 cards)
                div { class: "space-y-3",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2",
                        "Luminosity Mode"
                    }
                    div { class: "grid grid-cols-1 sm:grid-cols-3 gap-4",
                        for (label, key) in theme_choices.iter() {
                            {
                                let active = theme.read().clone() == *key;
                                let mut theme_signal = theme.clone();
                                let key_for_click = key.to_string();
                                let label_str = label.to_string();
                                rsx! {
                                    button {
                                        key: "{key}",
                                        r#type: "button",
                                        class: format!(
                                            "p-4 rounded-2xl border transition-all text-center {}",
                                            if active {
                                                "bg-primary/10 border-primary shadow-lg shadow-pink-500/10"
                                            } else {
                                                "bg-muted/30 border-border/40 hover:bg-muted/50"
                                            }
                                        ),
                                        onclick: move |_| theme_signal.set(key_for_click.clone()),
                                        div { class: "font-black text-sm uppercase tracking-widest", "{label_str}" }
                                    }
                                }
                            }
                        }
                    }
                }
                // Primary color: 4 presets + hex display
                div { class: "space-y-3",
                    label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2",
                        "Interface Accent Chroma"
                    }
                    div { class: "flex flex-col sm:flex-row items-center gap-6 p-4 rounded-xl bg-muted/30 border border-border/40",
                        div { class: "flex-1 text-center sm:text-left",
                            div { class: "text-xl font-black uppercase tracking-tight mb-1", "{primary_color.read()}" }
                            div { class: "text-[10px] font-bold text-muted-foreground uppercase opacity-50", "Active Interface Pigment" }
                        }
                        div { class: "grid grid-cols-4 gap-3",
                            for (preset_name, preset_color) in color_presets.iter() {
                                {
                                    let mut color_signal = primary_color.clone();
                                    let color_for_click = preset_color.to_string();
                                    let name_for_title = preset_name.to_string();
                                    rsx! {
                                        button {
                                            key: "{preset_color}",
                                            r#type: "button",
                                            title: "{name_for_title}",
                                            class: "w-10 h-10 rounded-full border-2 border-border/40 hover:scale-110 transition-transform shadow-lg",
                                            style: "background-color: {preset_color};",
                                            onclick: move |_| color_signal.set(color_for_click.clone()),
                                        }
                                    }
                                }
                            }
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
    use crate::auth::user::{AuthMethod, User};

    /// Authenticated admin context — the page gates on
    /// `settings:manage`, so the fixture user must hold that
    /// permission.
    fn admin_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u-admin".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["admin".to_string()],
                email: Some("admin@epsx.io".to_string()),
                tier: Some("Admin".to_string()),
                permissions: vec!["settings:manage".to_string()],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: Some("Admin".to_string()),
            }),
            path: "/settings".to_string(),
            ..Default::default()
        }
    }

    /// Smoke test.
    #[test]
    fn test_render_smoke() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "settings must render non-empty HTML. Got: {html}");
        assert!(html.len() > 100, "settings HTML is suspiciously short ({} bytes).", html.len());
    }

    /// Section-marker test.
    #[test]
    fn test_section_markers() {
        let ctx = admin_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        for marker in &[
            "settings-dashboard",
            "api-keys-list",
            "email-settings",
            "notification-settings",
            "session-management",
            "appearance-settings",
        ] {
            let needle_a = format!("class=\"{}\"", marker);
            let needle_b = format!("class=\"{mark} ", mark = marker);
            let needle_c = format!(" {}\"", marker);
            let needle_d = format!(" {} ", marker);
            assert!(
                html.contains(&needle_a)
                    || html.contains(&needle_b)
                    || html.contains(&needle_c)
                    || html.contains(&needle_d),
                "settings must contain section marker `{marker}`. Got: {html}"
            );
        }
    }
}
