//! `/settings` — authenticated admin settings workspace.
//!
//! Settings values and mutation authority remain in the Rust backend. The page
//! receives only the explicit non-secret settings allowlist and submits bounded
//! values through same-origin forms; it never derives defaults, permissions,
//! or versions.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::auth::AuthGate;
use crate::components::admin::data_state_banner::{AdminDataState, AdminDataStateBanner};
use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const SETTINGS_PATH: &str = "/settings";
const MAX_SETTINGS_CATEGORIES: usize = 100;
const MAX_SETTING_KEYS: usize = 100;
const MAX_SETTING_TEXT_CHARS: usize = 128;

pub const ADMIN_SETTINGS_DATA_PARAM: &str = "data_admin_settings";
pub const ADMIN_SETTINGS_STATE_PARAM: &str = "data_admin_settings_state";
pub const ADMIN_SETTINGS_READY: &str = "ready";
pub const ADMIN_SETTINGS_EMPTY: &str = "empty";
pub const ADMIN_SETTINGS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_SETTINGS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_SETTINGS_MALFORMED: &str = "malformed";
pub const ADMIN_SETTINGS_UNAUTHENTICATED: &str = "unauthenticated";
pub const ADMIN_SETTINGS_UNAUTHORIZED: &str = "unauthorized";
pub const ADMIN_SETTINGS_MUTATION_PARAM: &str = "settings_mutation";
pub const ADMIN_SETTINGS_TAB_PARAM: &str = "admin_settings_tab";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminSettingsQuery {
    pub tab: String,
    pub mutation: Option<String>,
}

impl AdminSettingsQuery {
    #[allow(clippy::result_unit_err)]
    pub fn from_raw(raw: &str) -> Result<Self, ()> {
        let mut tab = "general".to_string();
        let mut mutation = None;
        let mut seen = HashSet::new();
        for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            if !seen.insert(key.to_string()) {
                return Err(());
            }
            match key.as_ref() {
                "tab" if valid_settings_category(value.as_ref()) => tab = value.into_owned(),
                "mutation"
                    if matches!(
                        value.as_ref(),
                        "success"
                            | "conflict"
                            | "forbidden"
                            | "unauthorized"
                            | "invalid"
                            | "unavailable"
                            | "malformed"
                    ) =>
                {
                    mutation = Some(value.into_owned())
                }
                _ => return Err(()),
            }
        }
        Ok(Self { tab, mutation })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminSettingsSnapshot {
    pub categories: Vec<AdminSettingsCategory>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminSettingsCategory {
    pub category: String,
    pub settings: Vec<AdminSetting>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminSetting {
    pub key: String,
    pub value: AdminSettingValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "lowercase")]
pub enum AdminSettingValue {
    Text(String),
    Bool(bool),
    Number(i64),
}

pub fn decode_admin_settings_projection(value: serde_json::Value) -> Option<AdminSettingsSnapshot> {
    let snapshot: AdminSettingsSnapshot = serde_json::from_value(value).ok()?;
    if snapshot.categories.len() > MAX_SETTINGS_CATEGORIES {
        return None;
    }

    let mut categories = HashSet::with_capacity(snapshot.categories.len());
    for category in &snapshot.categories {
        if !valid_settings_category(&category.category)
            || !categories.insert(category.category.as_str())
            || category.settings.len() > MAX_SETTING_KEYS
        {
            return None;
        }
        let mut keys = HashSet::with_capacity(category.settings.len());
        for setting in &category.settings {
            if !safe_setting_text(&setting.key)
                || !keys.insert(setting.key.as_str())
                || !valid_setting_value(&category.category, &setting.key, &setting.value)
            {
                return None;
            }
        }
    }
    Some(snapshot)
}

fn valid_settings_category(value: &str) -> bool {
    matches!(
        value,
        "general" | "notifications" | "security" | "appearance"
    )
}

fn valid_setting_value(category: &str, key: &str, value: &AdminSettingValue) -> bool {
    match (category, key, value) {
        ("general", "systemName", AdminSettingValue::Text(value)) => {
            bounded_setting_value(value, 1, 128)
        }
        ("general", "adminEmail", AdminSettingValue::Text(value)) => {
            bounded_setting_value(value, 3, 254) && value.contains('@')
        }
        ("general", "maintenanceMode", AdminSettingValue::Bool(_))
        | ("notifications", "emailNotifications", AdminSettingValue::Bool(_))
        | ("notifications", "pushNotifications", AdminSettingValue::Bool(_))
        | ("notifications", "smsNotifications", AdminSettingValue::Bool(_))
        | ("notifications", "securityAlerts", AdminSettingValue::Bool(_)) => true,
        ("security", "sessionTimeout", AdminSettingValue::Number(value)) => {
            (1..=1440).contains(value)
        }
        ("appearance", "theme", AdminSettingValue::Text(value)) => {
            matches!(value.as_str(), "light" | "dark" | "system")
        }
        ("appearance", "primaryColor", AdminSettingValue::Text(value)) => {
            value.len() == 7
                && value.starts_with('#')
                && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
        }
        _ => false,
    }
}

fn bounded_setting_value(value: &str, min_chars: usize, max_chars: usize) -> bool {
    let count = value.chars().count();
    (min_chars..=max_chars).contains(&count) && !value.chars().any(char::is_control)
}

fn safe_setting_text(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_SETTING_TEXT_CHARS
        && !value.chars().any(char::is_control)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Settings");
    (meta, rsx! { RenderSettings { ctx: ctx.clone() } })
}

/// Session presence is the only frontend gate. Roles, permissions, query
/// values, and route parameters are deliberately not treated as settings data
/// or authorization policy; those decisions belong to the backend.
#[component]
fn RenderSettings(ctx: PageContext) -> Element {
    let load = settings_load(&ctx);
    let actions_enabled = matches!(&load, SettingsLoad::Ready(_));
    let mutation = ctx.params.get(ADMIN_SETTINGS_MUTATION_PARAM).cloned();
    let tab = ctx
        .params
        .get(ADMIN_SETTINGS_TAB_PARAM)
        .filter(|value| valid_settings_category(value))
        .cloned()
        .unwrap_or_else(|| "general".to_string());
    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private admin settings workspace".to_string()),
            return_url: Some("/settings".to_string()),
            PageLayout {
                max_width: Some(PageMaxWidth::SevenXl),
                PageHeader {
                    title: "Settings Nexus".to_string(),
                    subtitle: Some("Universal configuration interface for security, appearance, and system protocols".to_string()),
                    icon: Some("settings".to_string()),
                    gradient: Some(PageGradient::Warning),
                    centered: Some(true),
                    extra_actions: None,
                    class_name: None,
                }
                SettingsControlBar { tab: tab.clone(), actions_enabled }
                div { class: "admin-settings",
                    SettingsSurface { load, mutation, tab }
                }
            }
        }
    }
}

#[component]
fn SettingsControlBar(tab: String, actions_enabled: bool) -> Element {
    rsx! {
        div { class: "flex items-center justify-end gap-4 p-4 rounded-xl bg-card border border-border/20 shadow-xl",
            div { class: "flex items-center gap-4",
                if actions_enabled {
                    form { method: "post", action: "/settings/reset",
                        input { r#type: "hidden", name: "return_tab", value: tab }
                        input { r#type: "hidden", name: "idempotency_key", value: format!("admin.settings.reset.{}", Uuid::new_v4()) }
                        button {
                            class: "flex items-center gap-3 px-4 py-2 rounded-xl bg-muted/30 hover:bg-muted/50 border border-border/40 text-[10px] font-black uppercase tracking-widest transition-all",
                            r#type: "submit",
                            title: "Reset all settings to backend defaults",
                            Icon { name: "rotate-ccw".to_string(), size: Some(16) }
                            "Reset Logic"
                        }
                    }
                } else {
                    button {
                        class: "flex items-center gap-3 px-4 py-2 rounded-xl bg-muted/30 border border-border/40 text-[10px] font-black uppercase tracking-widest text-muted-foreground cursor-not-allowed",
                        r#type: "button",
                        disabled: true,
                        Icon { name: "rotate-ccw".to_string(), size: Some(16) }
                        "Reset Logic"
                    }
                }
                button {
                    class: "flex items-center gap-3 px-4 py-2 rounded-xl bg-muted/30 border border-border/40 text-[10px] font-black uppercase tracking-widest text-muted-foreground cursor-not-allowed",
                    r#type: "button",
                    disabled: true,
                    Icon { name: "save".to_string(), size: Some(16) }
                    "Synchronized"
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SettingsLoad {
    Ready(AdminSettingsSnapshot),
    Empty,
    Unauthenticated,
    Unauthorized,
    Forbidden,
    Unavailable,
    Malformed,
}

fn settings_load(ctx: &PageContext) -> SettingsLoad {
    let state = ctx
        .params
        .get(ADMIN_SETTINGS_STATE_PARAM)
        .map(String::as_str);
    match state {
        Some(ADMIN_SETTINGS_READY) | Some(ADMIN_SETTINGS_EMPTY) => {
            let Some(raw) = ctx.params.get(ADMIN_SETTINGS_DATA_PARAM) else {
                return SettingsLoad::Malformed;
            };
            let Some(snapshot) = serde_json::from_str::<serde_json::Value>(raw)
                .ok()
                .and_then(decode_admin_settings_projection)
            else {
                return SettingsLoad::Malformed;
            };
            if state == Some(ADMIN_SETTINGS_EMPTY) {
                if snapshot.categories.is_empty() {
                    SettingsLoad::Empty
                } else {
                    SettingsLoad::Malformed
                }
            } else if snapshot.categories.is_empty() {
                SettingsLoad::Malformed
            } else {
                SettingsLoad::Ready(snapshot)
            }
        }
        Some(ADMIN_SETTINGS_FORBIDDEN) => SettingsLoad::Forbidden,
        Some(ADMIN_SETTINGS_MALFORMED) => SettingsLoad::Malformed,
        Some(ADMIN_SETTINGS_UNAUTHENTICATED) => SettingsLoad::Unauthenticated,
        Some(ADMIN_SETTINGS_UNAUTHORIZED) => SettingsLoad::Unauthorized,
        Some(ADMIN_SETTINGS_UNAVAILABLE) | None => SettingsLoad::Unavailable,
        Some(_) => SettingsLoad::Malformed,
    }
}

#[component]
fn SettingsSurface(load: SettingsLoad, mutation: Option<String>, tab: String) -> Element {
    match load {
        SettingsLoad::Ready(snapshot) => rsx! {
            div {
                "data-admin-settings-state": ADMIN_SETTINGS_READY,
                if let Some(state) = mutation {
                    SettingsMutationNotice { state }
                }
                SettingsReadyPanel { snapshot, tab }
            }
        },
        SettingsLoad::Empty => rsx! {
            div { "data-admin-settings-state": ADMIN_SETTINGS_EMPTY,
                SettingsProblem { title: "No settings are configured".to_string(), detail: "The backend returned an authoritative empty settings store. No defaults are being invented.".to_string(), tab: tab.clone() }
                SettingsUnavailablePanel { tab }
            }
        },
        SettingsLoad::Forbidden => rsx! {
            div { "data-admin-settings-state": ADMIN_SETTINGS_FORBIDDEN,
                SettingsProblem { title: "Settings access was denied".to_string(), detail: "The backend did not authorize this session to read settings.".to_string(), tab: tab.clone() }
                SettingsUnavailablePanel { tab }
            }
        },
        SettingsLoad::Unavailable => rsx! {
            div { "data-admin-settings-state": ADMIN_SETTINGS_UNAVAILABLE,
                SettingsProblem { title: "Platform settings are unavailable".to_string(), detail: "No verified settings response is available. No configuration values are shown.".to_string(), tab: tab.clone() }
                SettingsUnavailablePanel { tab }
            }
        },
        SettingsLoad::Unauthenticated | SettingsLoad::Unauthorized => {
            let state = if matches!(load, SettingsLoad::Unauthenticated) {
                AdminDataState::Unauthenticated
            } else {
                AdminDataState::Unauthorized
            };
            rsx! {
                AdminDataStateBanner {
                    state,
                    subject: "Settings".to_string(),
                    return_path: SETTINGS_PATH.to_string(),
                    retry_href: SETTINGS_PATH.to_string(),
                }
            }
        }
        SettingsLoad::Malformed => rsx! {
            div { "data-admin-settings-state": ADMIN_SETTINGS_MALFORMED,
                SettingsProblem { title: "Settings data could not be verified".to_string(), detail: "The backend response did not match the strict settings read contract. No settings are shown.".to_string(), tab: tab.clone() }
                SettingsUnavailablePanel { tab }
            }
        },
    }
}

#[component]
fn SettingsReadyPanel(snapshot: AdminSettingsSnapshot, tab: String) -> Element {
    match tab.as_str() {
        "notifications" => {
            let email = bool_setting(&snapshot, "notifications", "emailNotifications");
            let push = bool_setting(&snapshot, "notifications", "pushNotifications");
            let sms = bool_setting(&snapshot, "notifications", "smsNotifications");
            let security = bool_setting(&snapshot, "notifications", "securityAlerts");
            rsx! {
                SettingsPanel {
                    id: "settings-notifications".to_string(),
                    title: "Signal Processing".to_string(),
                    subtitle: "Network Alert Preferences".to_string(),
                    icon: "bell".to_string(),
                    gradient: "from-[#7645d9] to-[#ed4b9e]".to_string(),
                    accent: "text-[#7645d9] border-[#7645d9]/20 bg-[#7645d9]/10".to_string(),
                    div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                        SettingsBoolControl { category: "notifications".to_string(), key_name: "emailNotifications".to_string(), label: "Email Notifications".to_string(), value: email, tab: tab.clone() }
                        SettingsBoolControl { category: "notifications".to_string(), key_name: "pushNotifications".to_string(), label: "Push Notifications".to_string(), value: push, tab: tab.clone() }
                        SettingsBoolControl { category: "notifications".to_string(), key_name: "smsNotifications".to_string(), label: "SMS Notifications".to_string(), value: sms, tab: tab.clone() }
                        SettingsBoolControl { category: "notifications".to_string(), key_name: "securityAlerts".to_string(), label: "Security Alerts".to_string(), value: security, tab }
                    }
                }
            }
        }
        "security" => {
            let timeout = number_setting(&snapshot, "security", "sessionTimeout");
            rsx! {
                SettingsPanel {
                    id: "settings-security".to_string(),
                    title: "Vault Protocols".to_string(),
                    subtitle: "Authentication and Access Controls".to_string(),
                    icon: "shield".to_string(),
                    gradient: "from-[#ffb237] to-[#ed4b9e]".to_string(),
                    accent: "text-[#ffb237] border-[#ffb237]/20 bg-[#ffb237]/10".to_string(),
                    if let Some(timeout) = timeout {
                        SettingsNumberControl { category: "security".to_string(), key_name: "sessionTimeout".to_string(), label: "Auto-Lock Duration (Minutes)".to_string(), value: timeout, min: 1, max: 1440, tab }
                    } else {
                        SettingsUnavailableControl { label: "Auto-Lock Duration (Minutes)".to_string() }
                    }
                    p { class: "ml-2 text-[10px] font-bold uppercase text-muted-foreground opacity-50", "Recommended: 15–60 minutes for optimal security" }
                }
            }
        }
        "appearance" => {
            let theme = text_setting(&snapshot, "appearance", "theme");
            let color = text_setting(&snapshot, "appearance", "primaryColor");
            rsx! {
                SettingsPanel {
                    id: "settings-appearance".to_string(),
                    title: "Optical Customization".to_string(),
                    subtitle: "Visual Feedback and Interface Styling".to_string(),
                    icon: "palette".to_string(),
                    gradient: "from-[#ed4b9e] to-[#7645d9]".to_string(),
                    accent: "text-[#ed4b9e] border-[#ed4b9e]/20 bg-[#ed4b9e]/10".to_string(),
                    p { class: "ml-2 text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Luminosity Mode" }
                    div { class: "grid grid-cols-1 gap-4 sm:grid-cols-3",
                        for (value, label) in [("light", "☀️ Daylight"), ("dark", "🌙 Eclipse"), ("system", "🔄 Neural")] {
                            form { method: "post", action: SETTINGS_PATH,
                                SettingsFormIdentity { category: "appearance".to_string(), key_name: "theme".to_string(), tab: tab.clone() }
                                button {
                                    class: if theme.as_deref() == Some(value) { "w-full rounded-2xl border border-primary bg-primary/10 p-6 text-center text-sm font-black uppercase tracking-widest shadow-lg" } else { "w-full rounded-2xl border border-border/40 bg-muted/30 p-6 text-center text-sm font-black uppercase tracking-widest hover:bg-muted/50" },
                                    r#type: "submit",
                                    name: "value_text",
                                    value,
                                    aria_pressed: (theme.as_deref() == Some(value)).to_string(),
                                    "{label}"
                                }
                            }
                        }
                    }
                    if let Some(color) = color {
                        form { class: "rounded-xl border border-border/40 bg-muted/30 p-4", method: "post", action: SETTINGS_PATH,
                            SettingsFormIdentity { category: "appearance".to_string(), key_name: "primaryColor".to_string(), tab }
                            label { class: "text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "Interface Accent Chroma" }
                            div { class: "mt-4 flex flex-col items-start gap-4 sm:flex-row sm:items-center",
                                input { class: "h-20 w-24 cursor-pointer rounded-2xl border border-border/40 bg-transparent", r#type: "color", name: "value_text", value: color.clone(), aria_label: "Interface accent color" }
                                div { class: "flex-1",
                                    p { class: "text-xl font-black uppercase tracking-tight text-foreground", "{color}" }
                                    p { class: "text-[10px] font-bold uppercase text-muted-foreground", "Active interface pigment" }
                                }
                                button { class: "btn btn-primary", r#type: "submit", "Deploy Update" }
                            }
                        }
                    } else {
                        SettingsUnavailableControl { label: "Interface Accent Chroma".to_string() }
                    }
                }
            }
        }
        _ => {
            let system_name = text_setting(&snapshot, "general", "systemName");
            let admin_email = text_setting(&snapshot, "general", "adminEmail");
            let maintenance = bool_setting(&snapshot, "general", "maintenanceMode");
            rsx! {
                SettingsPanel {
                    id: "settings-general".to_string(),
                    title: "System Configuration".to_string(),
                    subtitle: "Platform Core Parameters".to_string(),
                    icon: "globe".to_string(),
                    gradient: "from-[#1fc7d4] to-[#7645d9]".to_string(),
                    accent: "text-[#1fc7d4] border-[#1fc7d4]/20 bg-[#1fc7d4]/10".to_string(),
                    if let Some(system_name) = system_name {
                        SettingsTextControl { category: "general".to_string(), key_name: "systemName".to_string(), label: "System Designation".to_string(), input_type: "text".to_string(), value: system_name, tab: tab.clone() }
                    } else {
                        SettingsUnavailableControl { label: "System Designation".to_string() }
                    }
                    if let Some(admin_email) = admin_email {
                        SettingsTextControl { category: "general".to_string(), key_name: "adminEmail".to_string(), label: "Authority Email Channel".to_string(), input_type: "email".to_string(), value: admin_email, tab: tab.clone() }
                    } else {
                        SettingsUnavailableControl { label: "Authority Email Channel".to_string() }
                    }
                    SettingsBoolControl { category: "general".to_string(), key_name: "maintenanceMode".to_string(), label: "Maintenance Lock".to_string(), value: maintenance, tab }
                }
            }
        }
    }
}

#[component]
fn SettingsPanel(
    id: String,
    title: String,
    subtitle: String,
    icon: String,
    gradient: String,
    accent: String,
    children: Element,
) -> Element {
    rsx! {
        section { id, class: "overflow-hidden rounded-2xl border border-border/20 bg-card shadow-xl",
            div { class: "h-[3px] bg-gradient-to-r {gradient}" }
            header { class: "flex items-center gap-4 border-b border-border/20 p-5",
                span { class: "inline-flex rounded-[18px] border p-3 {accent}", aria_hidden: "true",
                    Icon { name: icon, size: Some(20) }
                }
                div {
                    h2 { class: "text-base font-bold uppercase tracking-wide text-foreground", "{title}" }
                    p { class: "text-xs text-muted-foreground", "{subtitle}" }
                }
            }
            div { class: "space-y-6 p-6", {children} }
        }
    }
}

#[component]
fn SettingsFormIdentity(category: String, key_name: String, tab: String) -> Element {
    rsx! {
        input { r#type: "hidden", name: "category", value: category }
        input { r#type: "hidden", name: "key", value: key_name }
        input { r#type: "hidden", name: "return_tab", value: tab }
        input { r#type: "hidden", name: "idempotency_key", value: format!("admin.settings.{}", Uuid::new_v4()) }
    }
}

#[component]
fn SettingsTextControl(
    category: String,
    key_name: String,
    label: String,
    input_type: String,
    value: String,
    tab: String,
) -> Element {
    rsx! {
        form { class: "space-y-4", method: "post", action: SETTINGS_PATH,
            SettingsFormIdentity { category, key_name, tab }
            label { class: "ml-2 block text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "{label}" }
            div { class: "flex flex-col gap-3 sm:flex-row",
                input { class: "h-16 min-w-0 flex-1 rounded-2xl border border-border/40 bg-muted/30 px-6 text-lg font-black outline-none transition-all focus:border-cyan-500/50 focus:bg-muted/50", r#type: input_type, name: "value_text", value, required: true, maxlength: "254" }
                button { class: "btn btn-primary h-12 self-end sm:h-16", r#type: "submit", "Deploy Update" }
            }
        }
    }
}

#[component]
fn SettingsNumberControl(
    category: String,
    key_name: String,
    label: String,
    value: i64,
    min: i64,
    max: i64,
    tab: String,
) -> Element {
    rsx! {
        form { class: "space-y-4", method: "post", action: SETTINGS_PATH,
            SettingsFormIdentity { category, key_name, tab }
            label { class: "ml-2 block text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground", "{label}" }
            div { class: "flex max-w-xl flex-col gap-3 sm:flex-row",
                input { class: "h-16 min-w-0 flex-1 rounded-2xl border border-border/40 bg-muted/30 px-6 text-lg font-black outline-none transition-all focus:border-amber-500/50 focus:bg-muted/50", r#type: "number", name: "value_number", value: value.to_string(), min: min.to_string(), max: max.to_string(), required: true }
                button { class: "btn btn-primary h-12 self-end sm:h-16", r#type: "submit", "Deploy Update" }
            }
        }
    }
}

#[component]
fn SettingsBoolControl(
    category: String,
    key_name: String,
    label: String,
    value: Option<bool>,
    tab: String,
) -> Element {
    if let Some(value) = value {
        let next_value = (!value).to_string();
        return rsx! {
            form { class: "flex items-center justify-between gap-4 rounded-xl border border-border/40 bg-muted/30 p-4", method: "post", action: SETTINGS_PATH,
                SettingsFormIdentity { category, key_name, tab }
                div { class: "flex items-center gap-4",
                    span { class: "inline-flex h-12 w-12 items-center justify-center rounded-xl bg-purple-500/15 text-purple-400", aria_hidden: "true",
                        Icon { name: "zap".to_string(), size: Some(22) }
                    }
                    div {
                        p { class: "text-sm font-black uppercase tracking-tight text-foreground", "{label}" }
                        p { class: "text-[10px] font-bold uppercase text-muted-foreground opacity-60", if value { "Active broadcast channel" } else { "Channel disabled" } }
                    }
                }
                button {
                    class: if value { "relative h-10 w-20 shrink-0 rounded-full bg-[#7645d9]" } else { "relative h-10 w-20 shrink-0 rounded-full bg-muted border border-border/40" },
                    r#type: "submit",
                    name: "value_bool",
                    value: next_value,
                    aria_label: format!("Toggle {label}"),
                    aria_pressed: value.to_string(),
                    span { class: if value { "absolute left-[46px] top-1.5 h-7 w-7 rounded-full bg-white" } else { "absolute left-1.5 top-1.5 h-7 w-7 rounded-full bg-white" } }
                }
            }
        };
    }
    rsx! { SettingsUnavailableControl { label } }
}

#[component]
fn SettingsUnavailableControl(label: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-4 rounded-xl border border-border/30 bg-muted/20 p-5",
            p { class: "text-sm font-semibold text-foreground", "{label}" }
            span { class: "font-mono text-xs text-amber-400", "Unavailable" }
        }
    }
}

#[component]
fn SettingsUnavailablePanel(tab: String) -> Element {
    let (title, subtitle, icon, gradient, accent, labels): (&str, &str, &str, &str, &str, &[&str]) =
        match tab.as_str() {
            "notifications" => (
                "Signal Processing",
                "Network Alert Preferences",
                "bell",
                "from-[#7645d9] to-[#ed4b9e]",
                "text-[#7645d9] border-[#7645d9]/20 bg-[#7645d9]/10",
                &[
                    "Email Notifications",
                    "Push Notifications",
                    "SMS Notifications",
                    "Security Alerts",
                ],
            ),
            "security" => (
                "Vault Protocols",
                "Authentication and Access Controls",
                "shield",
                "from-[#ffb237] to-[#ed4b9e]",
                "text-[#ffb237] border-[#ffb237]/20 bg-[#ffb237]/10",
                &["Auto-Lock Duration (Minutes)"],
            ),
            "appearance" => (
                "Optical Customization",
                "Visual Feedback and Interface Styling",
                "palette",
                "from-[#ed4b9e] to-[#7645d9]",
                "text-[#ed4b9e] border-[#ed4b9e]/20 bg-[#ed4b9e]/10",
                &["Luminosity Mode", "Interface Accent Chroma"],
            ),
            _ => (
                "System Configuration",
                "Platform Core Parameters",
                "globe",
                "from-[#1fc7d4] to-[#7645d9]",
                "text-[#1fc7d4] border-[#1fc7d4]/20 bg-[#1fc7d4]/10",
                &[
                    "System Designation",
                    "Authority Email Channel",
                    "Maintenance Lock",
                ],
            ),
        };
    rsx! {
        div { class: "mt-6",
            SettingsPanel { id: format!("settings-{tab}"), title: title.to_string(), subtitle: subtitle.to_string(), icon: icon.to_string(), gradient: gradient.to_string(), accent: accent.to_string(),
                for label in labels {
                    SettingsUnavailableControl { label: (*label).to_string() }
                }
            }
        }
    }
}

fn text_setting(snapshot: &AdminSettingsSnapshot, category: &str, key: &str) -> Option<String> {
    match setting_value(snapshot, category, key)? {
        AdminSettingValue::Text(value) => Some(value.clone()),
        _ => None,
    }
}

fn bool_setting(snapshot: &AdminSettingsSnapshot, category: &str, key: &str) -> Option<bool> {
    match setting_value(snapshot, category, key)? {
        AdminSettingValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn number_setting(snapshot: &AdminSettingsSnapshot, category: &str, key: &str) -> Option<i64> {
    match setting_value(snapshot, category, key)? {
        AdminSettingValue::Number(value) => Some(*value),
        _ => None,
    }
}

fn setting_value<'a>(
    snapshot: &'a AdminSettingsSnapshot,
    category: &str,
    key: &str,
) -> Option<&'a AdminSettingValue> {
    snapshot
        .categories
        .iter()
        .find(|item| item.category == category)?
        .settings
        .iter()
        .find(|item| item.key == key)
        .map(|item| &item.value)
}

#[component]
fn SettingsMutationNotice(state: String) -> Element {
    let (title, class_name) = match state.as_str() {
        "success" => (
            "Settings update committed",
            "border-green-500/30 bg-green-500/10",
        ),
        "conflict" => (
            "Settings changed; reload before retrying",
            "border-amber-500/30 bg-amber-500/10",
        ),
        "forbidden" => (
            "Settings update was denied",
            "border-red-500/30 bg-red-500/10",
        ),
        "invalid" => (
            "Settings update was invalid",
            "border-amber-500/30 bg-amber-500/10",
        ),
        _ => (
            "Settings update is unavailable",
            "border-amber-500/30 bg-amber-500/10",
        ),
    };
    rsx! { p { class: format!("m-6 rounded-xl border p-4 text-sm {class_name}"), role: "status", "data-admin-settings-mutation": state, "{title}" } }
}

#[component]
fn SettingsProblem(title: String, detail: String, tab: String) -> Element {
    rsx! {
        section {
            class: "rounded-xl border border-amber-500/30 bg-amber-500/10 px-5 py-4",
            role: "alert",
            div { class: "flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between",
                div {
                    h2 { class: "font-semibold text-foreground", "{title}" }
                    p { class: "mt-1 max-w-3xl text-sm text-muted-foreground", "{detail}" }
                }
                nav { class: "flex shrink-0 flex-wrap gap-2", aria_label: "Settings recovery",
                    a { class: "btn btn-sm btn-primary", href: format!("{SETTINGS_PATH}?tab={tab}"),
                        Icon { name: "refresh-cw".to_string(), size: Some(15) }
                        "Check again"
                    }
                    a { class: "btn btn-sm btn-outline", href: "/", "Admin home" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "admin-session".to_string(),
                address: "0x1234abcd5678ef90".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: "/settings".to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn settings_query_is_closed_and_preserves_tab_with_mutation() {
        assert_eq!(
            AdminSettingsQuery::from_raw("tab=appearance&mutation=success"),
            Ok(AdminSettingsQuery {
                tab: "appearance".to_string(),
                mutation: Some("success".to_string()),
            })
        );
        assert_eq!(
            AdminSettingsQuery::from_raw(""),
            Ok(AdminSettingsQuery {
                tab: "general".to_string(),
                mutation: None,
            })
        );
        for hostile in [
            "tab=unknown",
            "tab=general&tab=security",
            "mutation=HOSTILE",
            "secret=value",
        ] {
            assert_eq!(AdminSettingsQuery::from_raw(hostile), Err(()));
        }
    }

    #[test]
    fn signed_out_route_keeps_settings_state_private() {
        let rendered = html(&PageContext {
            path: "/settings".to_string(),
            ..Default::default()
        });

        assert!(rendered.contains("Sign in required"));
        assert!(rendered.contains("href=\"/auth?return_url=%2Fsettings\""));
        assert!(!rendered.contains("data-admin-settings-state"));
        assert!(!rendered.contains("admin-shell admin-shell-page"));
        assert!(!rendered.contains("Platform settings are unavailable"));
        assert!(!rendered.contains("Backend settings contract required"));
    }

    #[test]
    fn empty_role_authenticated_session_reaches_explicit_unavailable_state() {
        let rendered = html(&signed_in_ctx());

        assert!(rendered.contains("data-admin-settings-state=\"unavailable\""));
        assert!(rendered.contains("No verified settings response is available"));
        assert!(!rendered.contains("Backend settings contract required"));
        assert!(!rendered.contains("Permission required"));
        assert!(!rendered.contains("admin:settings:manage"));
    }

    #[test]
    fn unavailable_state_has_no_samples_secrets_sessions_or_mutation_labels() {
        let rendered = html(&signed_in_ctx());

        for forbidden in [
            "epx_live_",
            "Production read-only",
            "Webhook delivery",
            "Indexer sync",
            "MacBook Pro",
            "iPhone 15",
            "Bangkok, TH",
            "Active sessions (2)",
            "admin@epsx.io",
            "smtp.sendgrid.net",
            "EPSX Production",
            "Save",
            "Revoke",
            "Create",
            "Regenerate",
            "New key",
            "Deploy Update",
            "<form",
            "<input",
            "<select",
            "<textarea",
            "onclick=",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "sample settings value or mutation control leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn hostile_params_and_query_are_not_reflected() {
        let mut ctx = signed_in_ctx();
        ctx.query =
            "api_key=HOSTILE_SECRET&session=HOSTILE_SESSION&email=HOSTILE_EMAIL&save=HOSTILE_SAVE"
                .to_string();
        ctx.params = HashMap::from([
            ("secret".to_string(), "HOSTILE_PARAMETER_SECRET".to_string()),
            ("device".to_string(), "HOSTILE_PARAMETER_DEVICE".to_string()),
        ]);
        let rendered = html(&ctx);

        for forbidden in [
            "HOSTILE_SECRET",
            "HOSTILE_SESSION",
            "HOSTILE_EMAIL",
            "HOSTILE_SAVE",
            "HOSTILE_PARAMETER_SECRET",
            "HOSTILE_PARAMETER_DEVICE",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "hostile settings value leaked: {forbidden}"
            );
        }
        assert!(rendered.contains("data-admin-settings-state=\"unavailable\""));
    }

    #[test]
    fn direct_page_render_is_body_only_with_production_header_and_safe_recovery() {
        let rendered = html(&signed_in_ctx());

        assert!(rendered.contains("Settings Nexus"));
        assert!(rendered.contains("Universal configuration interface"));
        assert!(rendered.contains("Reset Logic"));
        assert!(rendered.contains("Synchronized"));
        assert!(!rendered.contains("class=\"admin-shell admin-shell-page\""));
        assert!(rendered.contains("href=\"/settings?tab=general\""));
        assert!(rendered.contains(">Check again</a>"));
        assert!(rendered.contains("href=\"/\">Admin home</a>"));
        assert!(!rendered.contains("javascript:"));
        assert!(!rendered.contains("onclick="));
    }

    fn ready_context(snapshot: AdminSettingsSnapshot, state: &str) -> PageContext {
        let mut ctx = signed_in_ctx();
        ctx.params = HashMap::from([
            (ADMIN_SETTINGS_STATE_PARAM.to_string(), state.to_string()),
            (
                ADMIN_SETTINGS_DATA_PARAM.to_string(),
                serde_json::to_string(&snapshot).unwrap(),
            ),
        ]);
        ctx
    }

    #[test]
    fn unauthenticated_and_unauthorized_decode_and_render_the_shared_banner() {
        let mut unauthenticated = signed_in_ctx();
        unauthenticated.params.insert(
            ADMIN_SETTINGS_STATE_PARAM.to_string(),
            ADMIN_SETTINGS_UNAUTHENTICATED.to_string(),
        );
        assert_eq!(
            settings_load(&unauthenticated),
            SettingsLoad::Unauthenticated
        );
        let rendered = html(&unauthenticated);
        assert!(rendered.contains("data-admin-data-state=\"unauthenticated\""));
        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-admin-settings-state"));

        let mut unauthorized = signed_in_ctx();
        unauthorized.params.insert(
            ADMIN_SETTINGS_STATE_PARAM.to_string(),
            ADMIN_SETTINGS_UNAUTHORIZED.to_string(),
        );
        assert_eq!(settings_load(&unauthorized), SettingsLoad::Unauthorized);
        let rendered = html(&unauthorized);
        assert!(rendered.contains("data-admin-data-state=\"unauthorized\""));
        assert!(rendered.contains("Session expired"));
        assert!(!rendered.contains("data-admin-settings-state"));
    }

    #[test]
    fn ready_projection_renders_allowlisted_values_and_bounded_backend_forms() {
        let snapshot = AdminSettingsSnapshot {
            categories: vec![AdminSettingsCategory {
                category: "general".into(),
                settings: vec![AdminSetting {
                    key: "systemName".into(),
                    value: AdminSettingValue::Text("EPSX Admin Console".into()),
                }],
            }],
        };
        let rendered = html(&ready_context(snapshot, ADMIN_SETTINGS_READY));
        assert!(rendered.contains("data-admin-settings-state=\"ready\""));
        assert!(rendered.contains("systemName"));
        assert!(rendered.contains("EPSX Admin Console"));
        assert!(rendered.contains("System Configuration"));
        assert!(rendered.contains("method=\"post\""));
        assert!(rendered.contains("name=\"value_text\""));
        assert!(rendered.contains("name=\"idempotency_key\""));
        assert!(rendered.contains("Deploy Update"));
        assert!(rendered.contains("action=\"/settings/reset\""));
    }

    #[test]
    fn settings_projection_rejects_unknown_fields_and_state_payload_mismatches() {
        let valid = serde_json::json!({
            "categories": [{
                "category": "general",
                "settings": [{"key": "systemName", "value": {"kind": "text", "value": "EPSX"}}]
            }]
        });
        assert!(decode_admin_settings_projection(valid.clone()).is_some());
        let mut unknown = valid.clone();
        unknown["categories"][0]["secret"] = serde_json::json!("private");
        assert!(decode_admin_settings_projection(unknown).is_none());

        let empty_state = ready_context(
            AdminSettingsSnapshot {
                categories: vec![AdminSettingsCategory {
                    category: "general".into(),
                    settings: vec![],
                }],
            },
            ADMIN_SETTINGS_EMPTY,
        );
        assert!(html(&empty_state).contains("data-admin-settings-state=\"malformed\""));
    }
}
