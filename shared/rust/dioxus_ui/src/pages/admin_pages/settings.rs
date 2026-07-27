//! `/settings` — authenticated admin settings workspace.
//!
//! The Rust admin does not yet consume a backend-authoritative settings read
//! model or mutation contract. Rendering defaults, API keys, active sessions,
//! account/security records, or editable values would therefore imply state
//! that has not been verified or cannot be persisted. Keep the page private
//! and fail closed until the backend supplies typed, authorized settings data.

use dioxus::prelude::*;
use serde::Deserialize;

use crate::auth::AuthGate;
use crate::components::admin::page_layout::{PageGradient, PageHeader};
use crate::layout::admin_shell::AdminShell;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

pub const ADMIN_SETTINGS_DATA_PARAM: &str = "data_admin_settings";
pub const ADMIN_SETTINGS_STATE_PARAM: &str = "data_admin_settings_state";
pub const ADMIN_SETTINGS_READY: &str = "ready";
pub const ADMIN_SETTINGS_EMPTY: &str = "empty";
pub const ADMIN_SETTINGS_FORBIDDEN: &str = "forbidden";
pub const ADMIN_SETTINGS_UNAVAILABLE: &str = "unavailable";
pub const ADMIN_SETTINGS_MALFORMED: &str = "malformed";

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdminSettingsProjection {
    categories: Vec<AdminSettingsCategory>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdminSettingsCategory {
    name: String,
    values: Vec<AdminSetting>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdminSetting {
    key: String,
    value: AdminSettingValue,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
enum AdminSettingValue {
    Text(String),
    Bool(bool),
    Number(i64),
}

#[derive(Clone, Debug, PartialEq)]
enum AdminSettingsLoad {
    Ready(AdminSettingsProjection),
    Empty,
    Forbidden,
    Unavailable,
    Malformed,
}

fn settings_load(ctx: &PageContext) -> AdminSettingsLoad {
    let state = ctx
        .params
        .get(ADMIN_SETTINGS_STATE_PARAM)
        .map(String::as_str)
        .unwrap_or(ADMIN_SETTINGS_UNAVAILABLE);
    match state {
        ADMIN_SETTINGS_READY => ctx
            .params
            .get(ADMIN_SETTINGS_DATA_PARAM)
            .and_then(|raw| serde_json::from_str(raw).ok())
            .map(AdminSettingsLoad::Ready)
            .unwrap_or(AdminSettingsLoad::Malformed),
        ADMIN_SETTINGS_EMPTY => AdminSettingsLoad::Empty,
        ADMIN_SETTINGS_FORBIDDEN => AdminSettingsLoad::Forbidden,
        ADMIN_SETTINGS_MALFORMED => AdminSettingsLoad::Malformed,
        ADMIN_SETTINGS_UNAVAILABLE => AdminSettingsLoad::Unavailable,
        _ => AdminSettingsLoad::Malformed,
    }
}

fn settings_state(load: &AdminSettingsLoad) -> &'static str {
    match load {
        AdminSettingsLoad::Ready(_) => ADMIN_SETTINGS_READY,
        AdminSettingsLoad::Empty => ADMIN_SETTINGS_EMPTY,
        AdminSettingsLoad::Forbidden => ADMIN_SETTINGS_FORBIDDEN,
        AdminSettingsLoad::Unavailable => ADMIN_SETTINGS_UNAVAILABLE,
        AdminSettingsLoad::Malformed => ADMIN_SETTINGS_MALFORMED,
    }
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
    let state = settings_state(&load);
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
                    "data-admin-settings-state": state,
                    if let AdminSettingsLoad::Ready(projection) = load {
                        SettingsReady { projection }
                    } else {
                    PageHeader {
                        title: "Settings Nexus".to_string(),
                        subtitle: Some("Universal configuration interface for security, appearance, and system protocols".to_string()),
                        icon: Some("settings".to_string()),
                        gradient: Some(PageGradient::Warning),
                        centered: Some(true),
                    }
                    div { class: "grid gap-6 xl:grid-cols-[minmax(0,1.7fr)_minmax(18rem,0.8fr)]",
                        section {
                            class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                            role: "status",
                            aria_labelledby: "admin-settings-unavailable-title",
                            "data-section": "admin-settings-unavailable",
                            div { class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#ffb237] via-[#ed4b9e] to-[#7645d9]" }
                            div { class: "p-8 md:p-12",
                                div { class: "flex flex-col gap-6 sm:flex-row sm:items-start",
                                    div {
                                        class: "flex h-16 w-16 shrink-0 items-center justify-center rounded-2xl border border-amber-500/20 bg-amber-500/10 text-[#ffb237]",
                                        aria_hidden: "true",
                                        Icon { name: "settings".to_string(), size: Some(30) }
                                    }
                                    div { class: "min-w-0",
                                        p { class: "text-xs font-black uppercase tracking-[0.22em] text-[#ffb237]",
                                            "Configuration workspace"
                                        }
                                        h2 {
                                            id: "admin-settings-unavailable-title",
                                            class: "mt-3 text-3xl font-black tracking-tight text-foreground",
                                            "Platform settings are unavailable"
                                        }
                                        p { class: "mt-4 max-w-3xl text-sm leading-6 text-muted-foreground",
                                            "No configuration values, credentials, account records, or session details are shown because a verified settings response is not connected. Unavailable settings are not presented as defaults or editable local state."
                                        }
                                        nav {
                                            class: "mt-8 flex flex-wrap gap-3",
                                            aria_label: "Settings recovery",
                                            a { class: "btn btn-primary", href: "/settings",
                                                Icon { name: "refresh-cw".to_string(), size: Some(16) }
                                                "Check again"
                                            }
                                            a { class: "btn btn-outline", href: "/", "Admin home" }
                                        }
                                    }
                                }
                            }
                        }

                        aside {
                            class: "rounded-3xl border border-border/40 bg-card/70 p-6",
                            aria_labelledby: "admin-settings-contract-title",
                            "data-section": "admin-settings-backend-contract",
                            h2 {
                                id: "admin-settings-contract-title",
                                class: "text-sm font-bold text-foreground",
                                "Backend settings contract required"
                            }
                            p { class: "mt-3 text-sm leading-6 text-muted-foreground",
                                "The backend must own authenticated reads, field-level authorization, validation, secret handling, concurrency, and audited mutations before settings operations can be enabled."
                            }
                            p { class: "mt-4 text-xs leading-5 text-muted-foreground",
                                "Frontend session roles and permissions are not used to grant settings access or derive configuration policy."
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
fn SettingsReady(projection: AdminSettingsProjection) -> Element {
    rsx! {
        section {
            class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
            aria_labelledby: "admin-settings-ready-title",
            "data-section": "admin-settings-ready",
            div { class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ed4b9e]" }
            div { class: "p-8 md:p-10",
                div { class: "flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between",
                    div {
                        p { class: "text-xs font-black uppercase tracking-[0.22em] text-[#1fc7d4]", "Backend-authoritative configuration" }
                        h2 { id: "admin-settings-ready-title", class: "mt-2 text-2xl font-black tracking-tight text-foreground", "Settings" }
                        p { class: "mt-2 max-w-3xl text-sm leading-6 text-muted-foreground", "These values were read from the backend and are displayed read-only. No local defaults or secret fields are shown." }
                    }
                    span { class: "inline-flex h-fit rounded-full border border-emerald-500/25 bg-emerald-500/10 px-3 py-1 text-xs font-semibold text-emerald-500", "Verified read" }
                }
                div { class: "mt-8 grid gap-6 md:grid-cols-2",
                    for category in projection.categories {
                        section { class: "rounded-2xl border border-border/30 bg-background/30 p-5", aria_labelledby: format!("admin-settings-category-{}", category.name),
                            h3 { id: format!("admin-settings-category-{}", category.name), class: "text-sm font-bold capitalize text-foreground", "{category.name}" }
                            dl { class: "mt-4 space-y-3",
                                for setting in category.values {
                                    div { class: "flex items-start justify-between gap-4 border-b border-border/20 pb-3 last:border-0 last:pb-0",
                                        dt { class: "text-sm text-muted-foreground", "{setting.key}" }
                                        dd { class: "max-w-[60%] break-words text-right text-sm font-semibold text-foreground", {render_setting_value(&setting.value)} }
                                    }
                                }
                            }
                        }
                    }
                }
                p { class: "mt-8 border-t border-border/30 pt-5 text-xs leading-5 text-muted-foreground", "Settings changes remain unavailable until the backend exposes a versioned, idempotent, audited manage contract." }
            }
        }
    }
}

fn render_setting_value(value: &AdminSettingValue) -> String {
    match value {
        AdminSettingValue::Text(value) => value.clone(),
        AdminSettingValue::Bool(value) => value.to_string(),
        AdminSettingValue::Number(value) => value.to_string(),
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
        assert!(rendered.contains("aria-labelledby=\"admin-settings-unavailable-title\""));
        assert!(rendered.contains("Backend settings contract required"));
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
}
