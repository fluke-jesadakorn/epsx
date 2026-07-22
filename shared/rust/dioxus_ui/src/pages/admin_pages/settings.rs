//! `/settings` — authenticated admin settings workspace.
//!
//! The Rust admin does not yet consume a backend-authoritative settings read
//! model or mutation contract. Rendering defaults, API keys, active sessions,
//! account/security records, or editable values would therefore imply state
//! that has not been verified or cannot be persisted. Keep the page private
//! and fail closed until the backend supplies typed, authorized settings data.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::admin_shell::AdminShell;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Settings unavailable");
    (meta, rsx! { RenderSettings { ctx: ctx.clone() } })
}

/// Session presence is the only frontend gate. Roles, permissions, query
/// values, and route parameters are deliberately not treated as settings data
/// or authorization policy; those decisions belong to the backend.
#[component]
fn RenderSettings(ctx: PageContext) -> Element {
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
                    "data-admin-settings-state": "unavailable",
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
