//! `/dashboard` — truthful personal-dashboard availability shell.
//!
//! The pinned TypeScript source supplied mock metrics and frontend-derived
//! roles, platforms, tiers, permissions, and feature access. The Rust frontend
//! has no owner-scoped dashboard read contract yet, so this page intentionally
//! ignores legacy `data_dashboard` payloads. An authenticated visitor sees only
//! identity values carried by the locally verified session and an explicit
//! unavailable state; a signed-out visitor sees a native sign-in path.

use dioxus::prelude::*;

use super::{PageContext, PageMeta};
use crate::auth::{AuthMethod, User};
use crate::layout::main_layout::MainLayout;
use crate::primitives::Icon;

const DASHBOARD_SIGN_IN_PATH: &str = "/auth?return_url=%2Fdashboard";

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Dashboard unavailable");
    (meta, rsx! { RenderDashboard { ctx: ctx.clone() } })
}

#[component]
fn RenderDashboard(ctx: PageContext) -> Element {
    // `ctx.params["data_dashboard"]` is deliberately not read. Until an
    // owner-scoped backend response is selected and validated, compatibility
    // payloads cannot establish metrics, activity, access, or entitlements.
    let user = ctx.user.clone();

    rsx! {
        MainLayout { ctx,
            div {
                class: "dashboard-prod-page min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800",
                div { class: "container mx-auto px-4 py-8",
                    header { class: "dashboard-prod-header mb-8",
                        h1 { class: "dashboard-prod-title text-3xl font-bold text-slate-900 dark:text-slate-100",
                            "Personal Dashboard"
                        }
                        p { class: "dashboard-prod-subtitle mt-2 text-slate-600 dark:text-slate-400",
                            "Review verified session identity and dashboard availability."
                        }
                    }

                    if let Some(user) = user {
                        AuthenticatedDashboard { user }
                    } else {
                        SignedOutDashboard {}
                    }
                }
            }
        }
    }
}

#[component]
fn SignedOutDashboard() -> Element {
    rsx! {
        section {
            class: "dashboard-prod-fallback card card-glass mx-auto max-w-2xl overflow-hidden text-center",
            "data-dashboard-state": "signed-out",
            aria_labelledby: "dashboard-sign-in-title",
            div { class: "h-1.5 bg-gradient-to-r from-orange-500 via-yellow-500 to-orange-500" }
            div { class: "card-body space-y-5 p-8",
                div { class: "mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-orange-500/10 text-orange-500",
                    Icon { name: "lock".to_string(), size: Some(30) }
                }
                h2 { id: "dashboard-sign-in-title", class: "text-2xl font-semibold text-foreground",
                    "Sign in required"
                }
                p { class: "text-sm leading-6 text-muted-foreground",
                    "Sign in to review the dashboard state associated with your verified session. No account data is shown while signed out."
                }
                a {
                    class: "btn btn-primary inline-flex items-center gap-2",
                    href: DASHBOARD_SIGN_IN_PATH,
                    Icon { name: "log-in".to_string(), size: Some(16) }
                    "Sign in"
                }
            }
        }
    }
}

#[component]
fn AuthenticatedDashboard(user: User) -> Element {
    rsx! {
        div { class: "dashboard-client relative overflow-hidden rounded-3xl",
            div {
                class: "dashboard-client-bg pointer-events-none absolute inset-0 bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900",
                "aria-hidden": "true"
            }
            div {
                class: "dashboard-client-orb-orange pointer-events-none absolute -left-40 -top-40 h-96 w-96 rounded-full bg-gradient-to-br from-orange-400/15 to-yellow-400/15 blur-3xl",
                "aria-hidden": "true"
            }
            div {
                class: "dashboard-client-orb-blue pointer-events-none absolute -right-32 top-20 h-80 w-80 rounded-full bg-gradient-to-br from-blue-400/12 to-cyan-400/12 blur-3xl",
                "aria-hidden": "true"
            }
            div {
                class: "dashboard-client-orb-purple pointer-events-none absolute bottom-20 left-20 h-72 w-72 rounded-full bg-gradient-to-br from-purple-400/10 to-pink-400/10 blur-3xl",
                "aria-hidden": "true"
            }

            div { class: "dashboard-client-content relative z-10 mx-auto max-w-5xl space-y-8 px-4 py-8 sm:px-6 lg:px-8",
                header { class: "dashboard-client-header text-center",
                    div { class: "dashboard-client-header-icon mb-6 inline-flex h-20 w-20 items-center justify-center rounded-3xl bg-gradient-to-br from-orange-500 to-yellow-500 shadow-2xl",
                        Icon { name: "trending-up".to_string(), size: Some(40), class_name: Some("text-white".to_string()) }
                    }
                    h2 {
                        class: "dashboard-client-title mb-4 bg-gradient-to-r from-orange-600 via-yellow-600 to-orange-600 bg-clip-text text-4xl font-bold text-transparent sm:text-5xl",
                        "Dashboard"
                    }
                    p { class: "text-base text-gray-600 dark:text-gray-300",
                        "You are signed in. Only locally verified session identity is displayed below."
                    }
                }

                div { class: "grid grid-cols-1 gap-6 lg:grid-cols-2",
                    SessionIdentityCard { user }
                    DashboardUnavailableCard {}
                }
            }
        }
    }
}

#[component]
fn SessionIdentityCard(user: User) -> Element {
    let display_name = non_empty(user.display_name.as_deref());
    let email = non_empty(user.email.as_deref());
    let wallet = non_empty(Some(user.address.as_str()));
    let auth_method = verified_auth_method_label(&user.auth_method);
    let has_claims =
        display_name.is_some() || email.is_some() || wallet.is_some() || auth_method.is_some();

    rsx! {
        section {
            class: "dashboard-session-identity rounded-2xl border border-orange-200/60 bg-white/85 p-6 shadow-xl backdrop-blur-xl dark:bg-slate-900/80",
            "data-dashboard-identity": "verified-session",
            aria_labelledby: "dashboard-session-title",
            div { class: "mb-5 flex items-start gap-3",
                div { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-orange-500 to-yellow-500 text-white",
                    Icon { name: "user".to_string(), size: Some(20) }
                }
                div {
                    h3 { id: "dashboard-session-title", class: "text-lg font-semibold text-foreground",
                        "Verified session identity"
                    }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "These values come from the locally verified access token."
                    }
                }
            }

            if has_claims {
                dl { class: "space-y-4 text-sm",
                    if let Some(display_name) = display_name {
                        SessionClaim { label: "Display name".to_string(), value: display_name.to_string(), monospace: false }
                    }
                    if let Some(email) = email {
                        SessionClaim { label: "Email".to_string(), value: email.to_string(), monospace: false }
                    }
                    if let Some(wallet) = wallet {
                        SessionClaim { label: "Wallet address".to_string(), value: wallet.to_string(), monospace: true }
                    }
                    if let Some(auth_method) = auth_method {
                        SessionClaim { label: "Authentication method".to_string(), value: auth_method.to_string(), monospace: false }
                    }
                }
            } else {
                p { class: "rounded-lg bg-slate-100 p-4 text-sm text-muted-foreground dark:bg-slate-800",
                    "No displayable identity claims were included in this verified session."
                }
            }
        }
    }
}

#[component]
fn SessionClaim(label: String, value: String, monospace: bool) -> Element {
    let value_class = if monospace {
        "mt-1 break-all font-mono text-xs text-foreground"
    } else {
        "mt-1 break-words font-medium text-foreground"
    };

    rsx! {
        div { class: "border-b border-slate-200 pb-3 last:border-b-0 last:pb-0 dark:border-slate-700",
            dt { class: "text-xs font-semibold uppercase tracking-wide text-muted-foreground", "{label}" }
            dd { class: value_class, "{value}" }
        }
    }
}

#[component]
fn DashboardUnavailableCard() -> Element {
    rsx! {
        section {
            class: "dashboard-data-unavailable rounded-2xl border border-amber-300/60 bg-white/85 p-6 shadow-xl backdrop-blur-xl dark:bg-slate-900/80",
            "data-dashboard-state": "unavailable",
            aria_labelledby: "dashboard-unavailable-title",
            role: "status",
            div { class: "mb-5 flex items-start gap-3",
                div { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-amber-500/10 text-amber-600",
                    Icon { name: "database".to_string(), size: Some(20) }
                }
                div {
                    p { class: "text-xs font-semibold uppercase tracking-widest text-amber-600",
                        "Dashboard unavailable"
                    }
                    h3 { id: "dashboard-unavailable-title", class: "mt-1 text-lg font-semibold text-foreground",
                        "Account summaries cannot be verified"
                    }
                }
            }

            p { class: "text-sm leading-6 text-muted-foreground",
                "There is no owner-scoped dashboard response that this frontend can validate. Metrics, recent activity, portfolio summaries, plan access, roles, permissions, and entitlements are not inferred."
            }

            nav { class: "mt-6 flex flex-wrap gap-3", aria_label: "Dashboard alternatives",
                a { class: "btn btn-primary inline-flex items-center gap-2", href: "/profile",
                    Icon { name: "user".to_string(), size: Some(16) }
                    "Review verified profile"
                }
            }
        }
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn verified_auth_method_label(method: &AuthMethod) -> Option<&'static str> {
    match method {
        AuthMethod::Wallet => Some("Wallet"),
        AuthMethod::Email => Some("Email"),
        AuthMethod::Demo => Some("Demo"),
        AuthMethod::OAuth => Some("OAuth"),
        AuthMethod::Siwe => Some("SIWE"),
        AuthMethod::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/dashboard".to_string(),
            ..Default::default()
        }
    }

    fn authed_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "session-subject-probe".to_string(),
                address: "0x9abc00000000000000000000000000000000def0".to_string(),
                chain_id: "unowned-chain-probe".to_string(),
                roles: vec!["unowned-role-probe".to_string()],
                email: Some("owner@example.invalid".to_string()),
                tier: Some("unowned-tier-probe".to_string()),
                permissions: vec!["unowned:permission:probe".to_string()],
                last_login_at: Some("unowned-last-login-probe".to_string()),
                auth_method: AuthMethod::Siwe,
                display_name: Some("Verified Owner".to_string()),
            }),
            path: "/dashboard".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_route_preserves_truthful_native_sign_in_state() {
        let html = render_to_string(&empty_ctx());

        assert!(html.contains("Personal Dashboard"));
        assert!(html.contains("data-dashboard-state=\"signed-out\""));
        assert!(html.contains("Sign in required"));
        assert!(html.contains("href=\"/auth?return_url=%2Fdashboard\""));
        assert!(!html.contains("data-dashboard-identity=\"verified-session\""));
        assert!(!html.contains("data-dashboard-state=\"unavailable\""));
    }

    #[test]
    fn authenticated_route_shows_only_verified_identity_and_unavailable_state() {
        let html = render_to_string(&authed_ctx());

        for expected in [
            "data-dashboard-identity=\"verified-session\"",
            "Verified session identity",
            "Verified Owner",
            "owner@example.invalid",
            "0x9abc00000000000000000000000000000000def0",
            "SIWE",
            "data-dashboard-state=\"unavailable\"",
            "Account summaries cannot be verified",
            "aria-label=\"Dashboard alternatives\"",
            "href=\"/profile\"",
        ] {
            assert!(
                html.contains(expected),
                "missing verified/unavailable marker: {expected}"
            );
        }
        assert!(!html.contains("href=\"/dashboard\""));
        assert!(!html.contains(">Retry</a>"));

        for forbidden in [
            "session-subject-probe",
            "unowned-chain-probe",
            "unowned-role-probe",
            "unowned-tier-probe",
            "unowned:permission:probe",
            "unowned-last-login-probe",
        ] {
            assert!(
                !html.contains(forbidden),
                "rendered unowned session claim: {forbidden}"
            );
        }
    }

    #[test]
    fn hostile_dashboard_payload_is_ignored() {
        let mut ctx = authed_ctx();
        ctx.params.insert(
            "data_dashboard".to_string(),
            r#"{
                "stats": {
                    "totalViews": 987654321,
                    "totalUsers": 876543210,
                    "revenue": 765432109
                },
                "recentActivity": [{"label": "payload-activity-probe"}],
                "role": "payload-role-probe",
                "tier": "payload-tier-probe",
                "platform": "payload-platform-probe",
                "entitlement": "payload-entitlement-probe"
            }"#
            .to_string(),
        );
        let html = render_to_string(&ctx);

        for forbidden in [
            "987654321",
            "876543210",
            "765432109",
            "payload-activity-probe",
            "payload-role-probe",
            "payload-tier-probe",
            "payload-platform-probe",
            "payload-entitlement-probe",
        ] {
            assert!(
                !html.contains(forbidden),
                "rendered dashboard payload claim: {forbidden}"
            );
        }
        assert!(html.contains("data-dashboard-state=\"unavailable\""));
    }

    #[test]
    fn legacy_business_cards_and_local_capability_controls_are_absent() {
        let html = render_to_string(&authed_ctx());

        for forbidden in [
            "Total Views",
            "Total Users",
            "Revenue",
            "Your Permissions",
            "Group:",
            "Premium Content",
            "Moderator Panel",
            "Configure your preferences",
            "View your data and insights",
            "href=\"/premium\"",
            "href=\"/moderator\"",
            "<button",
            "<form",
            "<input",
            "onclick=",
            "oninput=",
        ] {
            assert!(
                !html.contains(forbidden),
                "rendered unsupported claim/control: {forbidden}"
            );
        }
    }

    #[test]
    fn missing_optional_identity_claims_do_not_gain_fallback_values() {
        let mut ctx = authed_ctx();
        let user = ctx.user.as_mut().expect("authenticated fixture");
        user.address.clear();
        user.email = None;
        user.display_name = None;
        user.auth_method = AuthMethod::Unknown;
        let html = render_to_string(&ctx);

        assert!(html.contains("No displayable identity claims were included"));
        for forbidden in ["Guest", "FREE", "Group: user", "Unknown"] {
            assert!(
                !html.contains(forbidden),
                "rendered invented fallback: {forbidden}"
            );
        }
    }
}
