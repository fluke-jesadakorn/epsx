//! `/permissions` — authenticated, fail-closed permission visibility.
//!
//! The pinned TypeScript page derives permission status, expiry, analytics,
//! history, and definitions from authenticated backend responses. The Rust
//! route does not yet have an equivalent owner-data loader. It therefore
//! exposes only raw permission strings from the locally verified session and
//! never interprets those strings as canonical access decisions.

use dioxus::prelude::*;

use super::{PageContext, PageMeta};
use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::primitives::Icon;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Access");
    (meta, rsx! { PermissionsPage { ctx: ctx.clone() } })
}

#[component]
fn PermissionsPage(ctx: PageContext) -> Element {
    let session_permissions = ctx
        .user
        .as_ref()
        .map(|user| user.permissions.clone())
        .unwrap_or_default();
    rsx! { MainLayout { ctx: ctx.clone(), AuthGate {
        user: ctx.user.clone(), feature: Some("your permission information".to_string()),
        return_url: Some(ctx.path.clone()), wallet_connected: ctx.wallet.address.is_some(),
        PermissionsBody { session_permissions }
    } } }
}

#[component]
fn PermissionsBody(session_permissions: Vec<String>) -> Element {
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    rsx! {
                div { class: "container page-content max-w-6xl fe-page-layout",
                    PageHeader {
                        title: "Access".to_string(),
                        description: Some("Review the permissions included in your current sign-in session.".to_string()),
                        icon: Some("shield".to_string())
                    }

                    div { class: "grid grid-cols-1 gap-6 lg:grid-cols-5",
                        section {
                            class: "card card-glass lg:col-span-3 permissions-session-claims fe-surface",
                            "data-permissions-claims-state": "verified-session",
                            "aria-labelledby": "permissions-session-claims-title",
                            div { class: "card-header",
                                h2 {
                                    id: "permissions-session-claims-title",
                                    class: "card-title flex items-center gap-2",
                                    Icon { name: "key".to_string(), size: Some(20) }
                                    "Session permissions"
                                }
                                p { class: "text-sm text-muted-foreground fe-tone-muted",
                                    "These permissions were included when you signed in. Current plan access and expiry details are available in Account."
                                }
                            }
                            div { class: "card-body",
                                if session_permissions.is_empty() {
                                    p {
                                        class: "text-sm text-muted-foreground fe-tone-muted",
                                        "No permissions were included in this sign-in session."
                                    }
                                } else {
                                    ul {
                                        class: "flex flex-wrap gap-2",
                                        "aria-label": "Session permissions",
                                        for permission in session_permissions {
                                            li {
                                                class: "badge badge-outline font-mono",
                                                "data-session-permission-claim": "raw",
                                                "{permission}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        section {
                            class: "card card-glass lg:col-span-2 permissions-unavailable fe-surface",
                            "data-permissions-state": "unavailable",
                            "aria-labelledby": "permissions-unavailable-title",
                            role: "status",
                            div { class: "card-body flex h-full flex-col",
                                div { class: "flex h-12 w-12 items-center justify-center rounded-xl bg-amber-500/10 text-amber-500 fe-tone-warning",
                                    Icon { name: "database".to_string(), size: Some(24) }
                                }
                                p { class: "mt-5 text-xs font-semibold uppercase tracking-widest text-amber-500 fe-tone-warning",
                                    "Permission service unavailable"
                                }
                                h2 {
                                    id: "permissions-unavailable-title",
                                    class: "mt-2 text-xl font-semibold",
                                    "Access details cannot be verified right now"
                                }
                                p { class: "mt-3 text-sm leading-6 text-muted-foreground fe-tone-muted",
                                    "We couldn’t load your access details. Please try again later."
                                }
                                nav {
                                    class: "mt-6 flex flex-wrap gap-3 border-t border-border/40 pt-5",
                                    "aria-label": "Permission page alternatives",
                                    crate::navigation::AppLink {
                                        class: "btn btn-primary",
                                        href: "/account",
                                        onclick: move |event| crate::fullstack::analytics::follow_link(event, navigation, "/account"),
                                        "Back to account"
                                    }
                                }
                            }
                        }
                    }
                }
    }
}

#[component]
pub fn HydratedPermissions() -> Element {
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    use_context_provider(|| crate::fullstack::analytics::AnalyticsNavigation(navigate));
    let mut result = use_server_future(|| async { super::profile::read_profile().await })?;
    let data = result.read().clone();
    rsx! {
        document::Title { "Access — EPSX" }
        document::Meta { name: "description", content: "Review the permissions included in your verified sign-in session." }
        match data {
            Some(Ok(Ok(user))) => rsx! { PermissionsBody { session_permissions: user.permissions } },
            Some(Ok(Err(crate::fullstack::LoadError::Unauthenticated))) => rsx! {
                AuthGate { user: None, feature: Some("your permission information".to_string()), return_url: Some("/permissions".to_string()), wallet_connected: false,
                    div {}
                }
            },
            _ => rsx! { section { class: "fe-page", role: "status",
                p { "Could not load your verified session. Please try again." }
                button { r#type: "button", class: "fe-button", onclick: move |_| result.restart(), "Try again" }
            } },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::AuthMethod;
    use crate::auth::User;

    fn session_user(permissions: Vec<&str>) -> User {
        User {
            id: "owner-canary-subject".to_string(),
            address: "owner-canary-wallet".to_string(),
            chain_id: "owner-canary-chain".to_string(),
            roles: vec!["owner-canary-role".to_string()],
            email: Some("owner-canary@example.invalid".to_string()),
            tier: Some("owner-canary-tier".to_string()),
            permissions: permissions.into_iter().map(str::to_string).collect(),
            last_login_at: Some("owner-canary-login".to_string()),
            auth_method: AuthMethod::Siwe,
            display_name: Some("owner-canary-name".to_string()),
        }
    }

    fn page_ctx(user: Option<User>) -> PageContext {
        PageContext {
            user,
            path: "/permissions".to_string(),
            ..Default::default()
        }
    }

    fn render_page(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    fn assert_no_unsupported_permission_data(html: &str) {
        for forbidden in [
            "permissions:read",
            "Total permissions",
            "Expiring soon",
            "Feature × Plan",
            "Included on your plan",
            "Permissions by category",
            "Permission Activity History",
            "Platform Distribution",
            "Usage Statistics",
            "Pro plan",
            "Enterprise",
            "2024-09-15",
            "within 24 hours",
            "Request access",
            "permissions-tab-nav",
            "permissions-matrix",
            "permissions-history-table",
            "permissions-category-breakdown",
            "<button",
            "onclick=",
        ] {
            assert!(
                !html.contains(forbidden),
                "permissions page must not render unsupported data or client policy `{forbidden}`. Got: {html}"
            );
        }
    }

    #[test]
    fn signed_in_user_needs_no_circular_frontend_permission_gate() {
        let html = render_page(&page_ctx(Some(session_user(vec!["reports:read"]))));

        assert!(html.contains("data-permissions-state=\"unavailable\""));
        assert!(html.contains("data-permissions-claims-state=\"verified-session\""));
        assert!(html.contains("reports:read"));
        assert!(html.contains("aria-label=\"Permission page alternatives\""));
        assert!(html.contains("href=\"/account\""));
        assert!(!html.contains("href=\"/permissions\""));
        assert!(!html.contains("> Retry</a>"));
        assert!(!html.contains("Permission required"));
        assert_no_unsupported_permission_data(&html);
    }

    #[test]
    fn raw_verified_claims_are_rendered_exactly_and_html_escaped() {
        let html = render_page(&page_ctx(Some(session_user(vec![
            "reports:<script>alert(1)</script>&\"read\"",
            "billing:view",
        ]))));

        assert!(
            html.contains("reports:&#60;script&#62;alert(1)&#60;/script&#62;&#38;&#34;read&#34;"),
            "raw claim must be preserved with HTML-significant text escaped. Got: {html}"
        );
        assert!(html.contains("billing:view"));
        assert_eq!(
            html.matches("data-session-permission-claim=\"raw\"")
                .count(),
            2
        );
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains("owner-canary-tier"));
        assert!(!html.contains("owner-canary-role"));
        assert_no_unsupported_permission_data(&html);
    }

    #[test]
    fn empty_verified_claim_set_is_not_interpreted_as_no_access() {
        let html = render_page(&page_ctx(Some(session_user(vec![]))));

        assert!(html.contains("No permissions were included in this sign-in session."));
        assert!(html.contains("Access details cannot be verified right now"));
        assert!(!html.contains("no permissions assigned"));
        assert!(!html.contains("no access"));
        assert_no_unsupported_permission_data(&html);
    }

    #[test]
    fn signed_out_direct_render_reveals_no_owner_claims() {
        let html = render_page(&page_ctx(None));

        assert!(html.contains("Sign in required"));
        assert!(html.contains("href=\"/auth?return_url=%2Fpermissions\""));
        for owner_value in [
            "owner-canary-subject",
            "owner-canary-wallet",
            "owner-canary-role",
            "owner-canary-tier",
            "Session permissions",
            "data-session-permission-claim",
        ] {
            assert!(!html.contains(owner_value));
        }
        assert_no_unsupported_permission_data(&html);
    }

    #[test]
    fn legacy_and_hostile_params_cannot_supply_permission_data_or_policy() {
        let mut ctx = page_ctx(Some(session_user(vec!["verified:claim"])));
        ctx.query = "tab=history&permission=CANARY-QUERY-PERMISSION".to_string();
        for (key, value) in [
            ("tab", "analytics"),
            ("data_permissions", "CANARY-PARAM-PERMISSION"),
            ("plan", "CANARY-PARAM-PLAN"),
        ] {
            ctx.params.insert(key.to_string(), value.to_string());
        }

        let html = render_page(&ctx);
        assert!(html.contains("verified:claim"));
        assert!(html.contains("data-permissions-state=\"unavailable\""));
        for canary in [
            "CANARY-QUERY-PERMISSION",
            "CANARY-PARAM-PERMISSION",
            "CANARY-PARAM-PLAN",
        ] {
            assert!(!html.contains(canary));
        }
        assert_no_unsupported_permission_data(&html);
    }
}
