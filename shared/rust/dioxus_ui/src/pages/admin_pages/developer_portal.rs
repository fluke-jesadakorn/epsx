//! Truthful authenticated shell for `/developer-portal`.
//!
//! The legacy page advertised API credentials, usage, modules, documentation,
//! and mutations without obtaining any of that state from an authoritative
//! contract in the Rust admin BFF. Keep the workspace private and fail closed
//! until such a contract is connected: authenticated sessions see an explicit
//! unavailable state, while signed-out sessions see only the session gate.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::primitives::Icon;

use super::super::{PageContext, PageMeta};

const DEVELOPER_PORTAL_PATH: &str = "/developer-portal";

/// `/developer-portal` is session-gated only. Authorization, credential
/// ownership, plan access, and feature availability remain backend concerns.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Developer portal");

    // Legacy query and hydration parameters are intentionally ignored. Only a
    // future backend-owned response may create credential or usage state.
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the private developer portal workspace".to_string()),
                return_url: Some(DEVELOPER_PORTAL_PATH.to_string()),
                DeveloperPortalUnavailable {}
            }
        },
    )
}

/// The dispatcher intentionally maps this route to the shared production
/// access-denied panel before reaching this entry point. Delegating here keeps
/// direct callers fail closed as well and never exposes a credential form.
pub fn render_create_key(ctx: &PageContext) -> (PageMeta, Element) {
    super::access_denied_panel::render(ctx)
}

#[component]
fn DeveloperPortalUnavailable() -> Element {
    rsx! {
        div {
            class: "container page-content max-w-5xl py-10",
            "data-admin-developer-portal-state": "unavailable",
            section {
                class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                role: "status",
                aria_labelledby: "developer-portal-unavailable-title",
                div {
                    class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ed4b9e]"
                }
                div { class: "grid gap-8 p-8 md:grid-cols-[auto_1fr] md:p-12",
                    div {
                        class: "flex h-16 w-16 items-center justify-center rounded-2xl border border-violet-500/20 bg-violet-500/10 text-violet-400",
                        aria_hidden: "true",
                        Icon { name: "code".to_string(), size: Some(30) }
                    }
                    div {
                        p { class: "text-xs font-black uppercase tracking-[0.22em] text-violet-400",
                            "Developer workspace"
                        }
                        h1 {
                            id: "developer-portal-unavailable-title",
                            class: "mt-3 text-3xl font-black tracking-tight text-foreground",
                            "Developer portal data is unavailable"
                        }
                        div {
                            class: "mt-5 rounded-2xl border border-amber-500/20 bg-amber-500/10 p-5",
                            p { class: "text-sm font-semibold leading-6 text-foreground",
                                "No credential inventory, usage records, module availability, or integration reference is shown because this page is not connected to a backend-authoritative read contract."
                            }
                        }
                        p { class: "mt-5 max-w-3xl text-sm leading-6 text-muted-foreground",
                            "The verified session keeps this workspace private. The Rust backend must supply field-authorized redacted reads and explicitly authorized mutations before credential management can be exposed here."
                        }
                        div { class: "mt-8 grid gap-4 sm:grid-cols-3",
                            BoundaryItem {
                                icon: "database",
                                title: "Credential inventory",
                                detail: "Names, identifiers, status, and activity remain hidden without a verified field-authorized redacted response."
                            }
                            BoundaryItem {
                                icon: "bar-chart-3",
                                title: "Usage reporting",
                                detail: "Request totals, trends, quotas, and module access remain hidden without authoritative data."
                            }
                            BoundaryItem {
                                icon: "shield",
                                title: "Credential operations",
                                detail: "Management actions remain disabled until typed, authorized, and auditable mutations are connected."
                            }
                        }
                        nav {
                            class: "mt-8 flex flex-wrap gap-3",
                            aria_label: "Developer portal recovery",
                            a { class: "btn btn-primary", href: DEVELOPER_PORTAL_PATH,
                                Icon { name: "refresh-cw".to_string(), size: Some(16) }
                                " Retry developer portal"
                            }
                            a { class: "btn btn-ghost", href: "/",
                                Icon { name: "home".to_string(), size: Some(16) }
                                " Admin home"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BoundaryItem(icon: &'static str, title: &'static str, detail: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/20 bg-background/40 p-5",
            div { class: "flex items-center gap-2 font-semibold text-foreground",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{detail}" }
            span {
                class: "mt-3 inline-flex rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-400",
                "Unavailable"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::User;

    fn authenticated_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "session-user".to_string(),
                address: "0x1234".to_string(),
                chain_id: "56".to_string(),
                roles: vec![],
                permissions: vec![],
                ..Default::default()
            }),
            path: DEVELOPER_PORTAL_PATH.to_string(),
            ..Default::default()
        }
    }

    fn html(element: Element) -> String {
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_response_keeps_developer_state_private() {
        let mut ctx = PageContext {
            path: DEVELOPER_PORTAL_PATH.to_string(),
            query: "new_key=private-credential&requests=12450".to_string(),
            ..Default::default()
        };
        ctx.params
            .insert("credential".to_string(), "private-credential".to_string());

        let rendered = html(render(&ctx).1);

        assert!(rendered.contains("Sign in required"));
        assert!(!rendered.contains("data-admin-developer-portal-state"));
        assert!(!rendered.contains("Developer portal data is unavailable"));
        assert!(!rendered.contains("private-credential"));
        assert!(!rendered.contains("12450"));
    }

    #[test]
    fn authenticated_empty_role_session_reaches_unavailable_state() {
        let rendered = html(render(&authenticated_ctx()).1);

        assert!(rendered.contains("data-admin-developer-portal-state=\"unavailable\""));
        assert!(rendered.contains("Developer portal data is unavailable"));
        assert!(!rendered.contains("Permission required"));
        assert!(!rendered.contains("Admin access required"));
    }

    #[test]
    fn unavailable_state_removes_samples_credentials_and_controls() {
        let rendered = html(render(&authenticated_ctx()).1);

        for forbidden in [
            "Production",
            "Staging",
            "Old staging",
            "12,450",
            "12450",
            "API calls (7d)",
            "Available modules",
            "Rate Limits",
            "Authorization: Bearer",
            "epsx_live_",
            "epsx_test_",
            "epsx_dev_",
            "Create API key",
            "Revoke",
            "Edit Expiration",
            ">Copy<",
            "Quick Start",
            "curl -X",
            "<form",
            "<input",
            "<textarea",
            "<select",
            "<button",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "unsupported developer portal UI leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn hostile_params_and_query_values_are_ignored() {
        let mut ctx = authenticated_ctx();
        ctx.query = "new_key=epsx_live_private&success=true&usage=999999".to_string();
        ctx.params = HashMap::from([
            (
                "credential".to_string(),
                "\"><script>alert(1)</script>".to_string(),
            ),
            ("action".to_string(), "Create API key".to_string()),
        ]);

        let rendered = html(render(&ctx).1);

        assert!(rendered.contains("data-admin-developer-portal-state=\"unavailable\""));
        for forbidden in [
            "epsx_live_private",
            "success=true",
            "999999",
            "<script>alert(1)</script>",
            "Create API key",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "hostile value leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn recovery_is_native_exact_route_navigation_and_page_owns_no_admin_shell() {
        let rendered = html(render(&authenticated_ctx()).1);

        assert!(rendered.contains("href=\"/developer-portal\""));
        assert!(rendered.contains("Retry developer portal"));
        assert!(rendered.contains("href=\"/\""));
        assert!(rendered.contains("Admin home"));
        assert!(!rendered.contains("onclick="));
        assert!(!rendered.contains("javascript:"));
        assert!(!rendered.contains("class=\"admin-shell"));
        assert!(!rendered.contains("admin-shell-main"));
        assert!(!rendered.contains("<main"));
    }

    #[test]
    fn create_key_route_remains_access_denied_without_layout_or_form() {
        let mut ctx = authenticated_ctx();
        ctx.path = "/developer-portal/api-keys/create".to_string();

        let direct = html(render_create_key(&ctx).1);
        let dispatched = html(crate::pages::admin_pages::dispatch(&ctx).1);

        for rendered in [direct, dispatched] {
            assert!(rendered.contains("Access Denied"));
            assert!(!rendered.contains("Create API key"));
            assert!(!rendered.contains("<form"));
            assert!(!rendered.contains("class=\"admin-shell"));
            assert!(!rendered.contains("data-admin-developer-portal-state"));
        }
    }
}
