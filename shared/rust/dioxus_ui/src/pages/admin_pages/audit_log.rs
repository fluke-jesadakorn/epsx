//! `/audit-log` — authenticated admin audit-log workspace.
//!
//! Audit records are security-sensitive and the Rust frontend does not yet
//! have a backend-owned audit read contract. This route therefore preserves
//! the recognizable admin shell while rendering an explicit unavailable
//! state. It does not infer an empty result, accept legacy hydration data, or
//! expose sample records, client-side filters, row expansion, or exports.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Audit log unavailable");
    (meta, rsx! { RenderAuditLog { ctx: ctx.clone() } })
}

/// Keep audit content private while the backend contract is absent. Query and
/// route parameters are intentionally ignored: neither is a trusted source of
/// audit records, identities, filters, cursors, or authorization decisions.
#[component]
fn RenderAuditLog(ctx: PageContext) -> Element {
    rsx! {
        AuthGate {
            user: ctx.user.clone(),
            feature: Some("the private audit workspace".to_string()),
            return_url: Some("/audit-log".to_string()),
            div { class: "container page-content admin-audit-log",
                    header { class: "flex flex-col gap-2 mb-6",
                        div { class: "flex items-center gap-3",
                            div {
                                class: "inline-flex h-11 w-11 items-center justify-center rounded-xl bg-primary/10 text-primary",
                                aria_hidden: "true",
                                Icon { name: "history".to_string(), size: Some(22) }
                            }
                            div {
                                h1 { class: "text-2xl font-bold", "Audit log" }
                                p { class: "text-muted-foreground",
                                    "Security-sensitive platform activity"
                                }
                            }
                        }
                    }

                    div { class: "grid grid-cols-1 gap-6 lg:grid-cols-3",
                        section {
                            class: "card card-glass lg:col-span-2",
                            role: "status",
                            aria_labelledby: "audit-log-unavailable-title",
                            "data-section": "audit-log-unavailable",
                            "data-audit-log-state": "unavailable",
                            div { class: "card-body py-10 text-center",
                                div {
                                    class: "mx-auto mb-4 inline-flex h-14 w-14 items-center justify-center rounded-full bg-muted text-muted-foreground",
                                    aria_hidden: "true",
                                    Icon { name: "shield".to_string(), size: Some(28) }
                                }
                                h2 {
                                    id: "audit-log-unavailable-title",
                                    class: "text-xl font-semibold",
                                    "Audit records unavailable"
                                }
                                p { class: "mx-auto mt-2 max-w-2xl text-sm text-muted-foreground",
                                    "No audit result is being inferred. Records, actors, network details, timestamps, outcomes, and totals remain hidden until a dedicated backend audit contract is available."
                                }
                                div { class: "mt-6 flex flex-wrap justify-center gap-3",
                                    a { class: "btn btn-primary", href: "/audit-log", "Check again" }
                                    a { class: "btn btn-outline", href: "/", "Back to dashboard" }
                                }
                            }
                        }

                        aside {
                            class: "card card-glass",
                            aria_labelledby: "audit-log-contract-title",
                            "data-section": "audit-log-backend-contract",
                            div { class: "card-body",
                                h2 {
                                    id: "audit-log-contract-title",
                                    class: "text-sm font-semibold",
                                    "Backend audit contract required"
                                }
                                p { class: "mt-2 text-sm text-muted-foreground",
                                    "The backend must own dedicated audit authorization, immutable cursor pagination, sensitive-field redaction, and authorized server-side export."
                                }
                                p { class: "mt-3 text-xs text-muted-foreground",
                                    "Frontend session claims are not used to grant audit access or derive these policies."
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

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u1".to_string(),
                address: "0x1234".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: "/audit-log".to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_route_keeps_audit_state_private() {
        let rendered = html(&PageContext {
            path: "/audit-log".to_string(),
            ..Default::default()
        });

        assert!(rendered.contains("Sign in required"));
        assert!(rendered.contains("href=\"/auth?return_url=%2Faudit-log\""));
        assert!(!rendered.contains("data-audit-log-state"));
        assert!(!rendered.contains("Audit records unavailable"));
    }

    #[test]
    fn authenticated_user_sees_explicit_unavailable_state_without_ui_permission_gate() {
        let rendered = html(&signed_in_ctx());

        assert!(rendered.contains("data-audit-log-state=\"unavailable\""));
        assert!(rendered.contains("aria-labelledby=\"audit-log-unavailable-title\""));
        assert!(rendered.contains("Backend audit contract required"));
        assert!(rendered.contains("dedicated audit authorization"));
        assert!(!rendered.contains("Permission required"));
    }

    #[test]
    fn sample_and_hostile_legacy_values_are_suppressed() {
        let mut ctx = signed_in_ctx();
        ctx.query = "actor=admin%40epsx.io&ip=192.168.1.1&result=success&total=4".to_string();
        ctx.params.insert(
            "entries".to_string(),
            "0xADMIN0000000000000000000000000000000001".to_string(),
        );
        let rendered = html(&ctx);

        for forbidden in [
            "admin@epsx.io",
            "192.168.1.1",
            "10.0.0.1",
            "2024-09-20",
            "user.create",
            "plan.update",
            "wallet.connect",
            "news.publish",
            "0xADMIN0000000000000000000000000000000001",
            "Total: 4",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "sample or hostile audit value leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn unavailable_workspace_has_no_fake_audit_controls() {
        let rendered = html(&signed_in_ctx());

        for forbidden in [
            "audit-filters",
            "audit-timeline-row",
            "audit-entry-detail",
            "audit-severity-breakdown",
            "audit-export-button",
            "data-export",
            "Export CSV",
            "Export JSON",
            "Search by actor",
            "All Actions",
            "Severity breakdown",
            "Page 1 of 1",
            "<input",
            "<select",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "fake audit control leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn recovery_uses_safe_native_links() {
        let rendered = html(&signed_in_ctx());

        assert!(rendered.contains("href=\"/audit-log\""));
        assert!(rendered.contains("href=\"/\""));
        assert!(!rendered.contains("javascript:"));
        assert!(!rendered.contains("onclick="));
    }
}
