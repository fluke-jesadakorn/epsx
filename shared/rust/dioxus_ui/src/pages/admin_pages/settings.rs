//! `/settings` — authenticated admin settings workspace.
//!
//! Settings values and mutation authority remain in the Rust backend. The page
//! receives only redacted key/type metadata and submits bounded JSON through a
//! same-origin form; it never derives defaults, permissions, or versions.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthGate;
use crate::layout::admin_shell::AdminShell;
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
pub const ADMIN_SETTINGS_MUTATION_PARAM: &str = "settings_mutation";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminSettingsSnapshot {
    pub categories: Vec<AdminSettingsCategory>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminSettingsCategory {
    pub category: String,
    pub settings: Vec<AdminSettingSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminSettingSummary {
    pub key: String,
    pub value_kind: String,
}

pub fn decode_admin_settings_projection(value: serde_json::Value) -> Option<AdminSettingsSnapshot> {
    let snapshot: AdminSettingsSnapshot = serde_json::from_value(value).ok()?;
    if snapshot.categories.len() > MAX_SETTINGS_CATEGORIES
        || snapshot.categories.iter().any(|category| {
            !safe_setting_text(&category.category)
                || category.settings.len() > MAX_SETTING_KEYS
                || category.settings.iter().any(|setting| {
                    !safe_setting_text(&setting.key)
                        || !matches!(
                            setting.value_kind.as_str(),
                            "null" | "boolean" | "number" | "string" | "array" | "object"
                        )
                })
        })
    {
        return None;
    }
    Some(snapshot)
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
    let mutation = ctx.params.get(ADMIN_SETTINGS_MUTATION_PARAM).cloned();
    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private admin settings workspace".to_string()),
            return_url: Some("/settings".to_string()),
            AdminShell {
                ctx: ctx.clone(),
                page_title: "Settings".to_string(),
                breadcrumbs: vec![
                    ("Dashboard".to_string(), "/".to_string()),
                    ("Settings".to_string(), "/settings".to_string()),
                ],
                div {
                    class: "container page-content admin-settings py-8",
                    SettingsSurface { load, mutation }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SettingsLoad {
    Ready(AdminSettingsSnapshot),
    Empty,
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
        Some(ADMIN_SETTINGS_UNAVAILABLE) | None => SettingsLoad::Unavailable,
        Some(_) => SettingsLoad::Malformed,
    }
}

#[component]
fn SettingsSurface(load: SettingsLoad, mutation: Option<String>) -> Element {
    match load {
        SettingsLoad::Ready(snapshot) => rsx! {
            section {
                class: "rounded-3xl border border-border/40 bg-card shadow-2xl",
                role: "region",
                "data-admin-settings-state": ADMIN_SETTINGS_READY,
                h2 { class: "p-8 text-2xl font-black text-foreground", "Authoritative settings" }
                div { class: "grid gap-5 border-t border-border/30 p-6 md:grid-cols-2",
                    for category in snapshot.categories {
                        article { class: "rounded-2xl border border-border/30 bg-background/40 p-5",
                            h3 { class: "font-semibold text-foreground", "{category.category}" }
                            ul { class: "mt-4 space-y-3",
                                for setting in category.settings {
                                    li { class: "flex items-center justify-between gap-4 border-t border-border/20 pt-3 text-sm",
                                        span { class: "break-all text-foreground", "{setting.key}" }
                                        span { class: "shrink-0 rounded-full border border-border/30 px-2 py-1 text-xs text-muted-foreground", "{setting.value_kind}" }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(state) = mutation {
                    SettingsMutationNotice { state }
                }
                section { class: "border-t border-border/30 p-6", aria_label: "Settings mutations",
                    h3 { class: "text-lg font-semibold text-foreground", "Backend-authorized update" }
                    p { class: "mt-2 text-sm text-muted-foreground", "Submit one JSON value. The backend validates the key, permission, idempotency key, and optimistic version contract." }
                    form { class: "mt-5 grid gap-4 md:grid-cols-2", method: "post", action: SETTINGS_PATH,
                        label { class: "grid gap-2 text-sm", "Category", input { name: "category", required: true, maxlength: "64", pattern: "[A-Za-z0-9_-]+" } }
                        label { class: "grid gap-2 text-sm", "Key", input { name: "key", required: true, maxlength: "128", pattern: "[A-Za-z0-9_-]+" } }
                        label { class: "grid gap-2 text-sm md:col-span-2", "Value (JSON)", textarea { name: "value_json", required: true, maxlength: "32768", rows: "4" } }
                        label { class: "grid gap-2 text-sm", "Expected updated at (optional)", input { name: "expected_updated_at", maxlength: "64" } }
                        input { r#type: "hidden", name: "idempotency_key", value: format!("admin.settings.{}", Uuid::new_v4()) }
                        div { class: "flex items-end gap-3", button { r#type: "submit", class: "btn btn-primary", "Update setting" }, a { class: "btn btn-outline", href: SETTINGS_PATH, "Reload" } }
                    }
                    form { class: "mt-4", method: "post", action: "/settings/reset",
                        input { r#type: "hidden", name: "idempotency_key", value: format!("admin.settings.reset.{}", Uuid::new_v4()) }
                        button { r#type: "submit", class: "btn btn-outline", "Reset to backend defaults" }
                    }
                }
                p { class: "border-t border-border/30 p-6 text-sm text-muted-foreground", "Values are owned by the backend; the form submits them for validation and persistence without exposing a client-side settings authority." }
            }
        },
        SettingsLoad::Empty => rsx! {
            SettingsProblem { state: ADMIN_SETTINGS_EMPTY, title: "No settings are configured".to_string(), detail: "The backend returned an authoritative empty settings store. No defaults are being invented.".to_string() }
        },
        SettingsLoad::Forbidden => rsx! {
            SettingsProblem { state: ADMIN_SETTINGS_FORBIDDEN, title: "Settings access was denied".to_string(), detail: "The backend did not authorize this session to read settings.".to_string() }
        },
        SettingsLoad::Unavailable => rsx! {
            SettingsProblem { state: ADMIN_SETTINGS_UNAVAILABLE, title: "Platform settings are unavailable".to_string(), detail: "No verified settings response is available. No configuration values are shown.".to_string() }
        },
        SettingsLoad::Malformed => rsx! {
            SettingsProblem { state: ADMIN_SETTINGS_MALFORMED, title: "Settings data could not be verified".to_string(), detail: "The backend response did not match the strict settings read contract. No settings are shown.".to_string() }
        },
    }
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
fn SettingsProblem(state: &'static str, title: String, detail: String) -> Element {
    rsx! {
        section {
            class: "rounded-3xl border border-amber-500/30 bg-card p-8 shadow-2xl",
            role: "alert",
            "data-admin-settings-state": state,
            h2 { class: "text-2xl font-black text-foreground", "{title}" }
            p { class: "mt-4 max-w-3xl text-sm leading-6 text-muted-foreground", "{detail}" }
            nav { class: "mt-8 flex flex-wrap gap-3", aria_label: "Settings recovery",
                a { class: "btn btn-primary", href: SETTINGS_PATH,
                    Icon { name: "refresh-cw".to_string(), size: Some(16) }
                    "Check again"
                }
                a { class: "btn btn-outline", href: "/", "Admin home" }
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
            "Reset Logic",
            "Deploy Update",
            "Synchronized",
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
    fn direct_page_render_owns_one_shell_and_safe_native_recovery() {
        let rendered = html(&signed_in_ctx());

        assert_eq!(
            rendered
                .matches("class=\"admin-shell admin-shell-page\"")
                .count(),
            1,
            "the settings page must own exactly one admin shell"
        );
        assert!(rendered.contains("class=\"admin-shell-main\""));
        assert!(rendered.contains("href=\"/settings\""));
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
    fn ready_projection_renders_metadata_and_bounded_backend_forms_without_values() {
        let snapshot = AdminSettingsSnapshot {
            categories: vec![AdminSettingsCategory {
                category: "general".into(),
                settings: vec![AdminSettingSummary {
                    key: "systemName".into(),
                    value_kind: "string".into(),
                }],
            }],
        };
        let rendered = html(&ready_context(snapshot, ADMIN_SETTINGS_READY));
        assert!(rendered.contains("data-admin-settings-state=\"ready\""));
        assert!(rendered.contains("systemName"));
        assert!(rendered.contains("string"));
        assert!(!rendered.contains("EPSX Admin Console"));
        assert!(rendered.contains("method=\"post\""));
        assert!(rendered.contains("name=\"value_json\""));
        assert!(rendered.contains("name=\"idempotency_key\""));
        assert!(rendered.contains("Update setting"));
        assert!(rendered.contains("Reset to backend defaults"));
    }

    #[test]
    fn settings_projection_rejects_unknown_fields_and_state_payload_mismatches() {
        let valid = serde_json::json!({
            "categories": [{
                "category": "general",
                "settings": [{"key": "theme", "value_kind": "string"}]
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
