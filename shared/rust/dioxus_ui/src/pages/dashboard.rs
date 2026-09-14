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
    // Availability is represented in the dashboard body; keep the document
    // metadata aligned with the source route's user-facing title.
    let meta = PageMeta::app("Dashboard");
    (meta, rsx! { RenderDashboard { ctx: ctx.clone() } })
}

#[component]
fn RenderDashboard(ctx: PageContext) -> Element {
    rsx! { MainLayout { ctx: ctx.clone(), DashboardBody { user: ctx.user } } }
}

#[component]
fn DashboardBody(user: Option<User>) -> Element {
    rsx! {
            div {
                class: "dashboard-prod-page min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800 fe-base-page fe-fill-neutral",
                div { class: "container mx-auto px-4 py-8 fe-page-layout",
                    header { class: "dashboard-prod-header mb-8",
                        h1 { class: "dashboard-prod-title text-3xl font-bold text-slate-900 dark:text-slate-100 fe-tone-text fe-type-title",
                            "Overview"
                        }
                        p { class: "dashboard-prod-subtitle mt-2 text-slate-600 dark:text-slate-400 fe-tone-muted",
                            "Pick up where you left off."
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

#[component]
pub fn HydratedDashboard() -> Element {
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    use_context_provider(|| crate::fullstack::analytics::AnalyticsNavigation(navigate));
    let mut result = use_server_future(|| async { super::profile::read_profile().await })?;
    let data = result.read().clone();
    rsx! {
        document::Title { "Overview — EPSX" }
        document::Meta { name: "description", content: "Return to your workspace and review your verified sign-in identity." }
        match data {
            Some(Ok(Ok(user))) => rsx! { DashboardBody { user: Some(user) } },
            Some(Ok(Err(crate::fullstack::LoadError::Unauthenticated))) => rsx! { DashboardBody { user: None } },
            _ => rsx! { section { class: "fe-page", role: "status",
                p { "Could not load your verified session. Please try again." }
                button { r#type: "button", class: "fe-button", onclick: move |_| result.restart(), "Try again" }
            } },
        }
    }
}

#[component]
fn SignedOutDashboard() -> Element {
    rsx! {
        section {
            // Match the source page's signed-out branch: the fallback is a
            // normal centered `p-8` block after the header, rather than a
            // custom flex/min-height surface that shifts the message upward
            // on the mobile capture.
            class: "dashboard-prod-fallback mx-auto max-w-3xl p-8 text-center",
            "data-dashboard-state": "signed-out",
            aria_labelledby: "dashboard-sign-in-title",
            p { class: "text-base leading-relaxed text-slate-400 sm:text-xl fe-tone-muted",
                "Please sign in to access your dashboard..."
            }
            div { class: "mt-4",
                h2 { id: "dashboard-sign-in-title", "Sign in required" }
                p { "Sign in to review the dashboard state associated with your verified session. No account data is shown while signed out." }
                crate::navigation::AppLink { href: DASHBOARD_SIGN_IN_PATH, "Sign in" }
            }
        }
    }
}

#[component]
fn AuthenticatedDashboard(user: User) -> Element {
    let navigation = try_consume_context::<crate::fullstack::analytics::AnalyticsNavigation>();
    rsx! {
        div { class: "dashboard-client",
            nav { class: "fe-overview-links", aria_label: "Your workspace",
                for (href,title,description) in [
                    ("/analytics","Explore","Browse reported company data and quarterly details."),
                    ("/portfolio","Saved companies","Return to the companies you follow."),
                    ("/account","Account","Manage your profile, preferences and access."),
                ] {
                    crate::navigation::AppLink { href, onclick: move |event| {
                        if crate::fullstack::shell::migrated_link(href) {
                            crate::fullstack::analytics::follow_link(event, navigation, href);
                        }
                    }, strong { "{title}" } span { "{description}" } }
                }
            }
            div { class: "mt-8 grid grid-cols-1 gap-6 lg:grid-cols-2",
                SessionIdentityCard { user }
                DashboardUnavailableCard {}
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
            class: "dashboard-session-identity rounded-2xl border border-orange-200/60 bg-white/85 p-6 shadow-xl backdrop-blur-xl dark:bg-slate-900/80 fe-surface",
            "data-dashboard-identity": "verified-session",
            aria_labelledby: "dashboard-session-title",
            div { class: "mb-5 flex items-start gap-3",
                div { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-orange-500 to-yellow-500 text-white fe-fill-neutral fe-tone-text",
                    Icon { name: "user".to_string(), size: Some(20) }
                }
                div {
                    h3 { id: "dashboard-session-title", class: "text-lg font-semibold text-foreground fe-tone-text",
                        "Verified session identity"
                    }
                    p { class: "mt-1 text-sm text-muted-foreground fe-tone-muted",
                        "Your signed-in account details."
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
                p { class: "rounded-lg bg-slate-100 p-4 text-sm text-muted-foreground dark:bg-slate-800 fe-fill-neutral fe-tone-muted",
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
            dt { class: "text-xs font-semibold uppercase tracking-wide text-muted-foreground fe-tone-muted", "{label}" }
            dd { class: value_class, "{value}" }
        }
    }
}

#[component]
fn DashboardUnavailableCard() -> Element {
    rsx! {
        section {
            class: "dashboard-data-unavailable rounded-2xl border border-amber-300/60 bg-white/85 p-6 shadow-xl backdrop-blur-xl dark:bg-slate-900/80 fe-surface",
            "data-dashboard-state": "unavailable",
            aria_labelledby: "dashboard-unavailable-title",
            role: "status",
            div { class: "mb-5 flex items-start gap-3",
                div { class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-amber-500/10 text-amber-600 fe-tone-warning",
                    Icon { name: "database".to_string(), size: Some(20) }
                }
                div {
                    p { class: "text-xs font-semibold uppercase tracking-widest text-amber-600 fe-tone-warning",
                        "Summary unavailable"
                    }
                    h3 { id: "dashboard-unavailable-title", class: "mt-1 text-lg font-semibold text-foreground fe-tone-text",
                        "Account summaries cannot be verified"
                    }
                }
            }

            p { class: "text-sm leading-6 text-muted-foreground fe-tone-muted",
                "Your account summary is not available yet. You can still explore data and manage your account."
            }

            nav { class: "mt-6 flex flex-wrap gap-3", aria_label: "Dashboard alternatives",
                crate::navigation::AppLink { class: "btn btn-primary inline-flex items-center gap-2", href: "/profile",
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

        assert!(html.contains("Overview"));
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
    fn source_dashboard_cards_are_navigation_only_without_local_capability_controls() {
        let html = render_to_string(&authed_ctx());

        for expected in [
            "Explore",
            "Saved companies",
            "Account",
            "fe-overview-links",
            "href=\"/analytics\"",
            "href=\"/portfolio\"",
            "href=\"/account\"",
        ] {
            assert!(
                html.contains(expected),
                "missing source dashboard surface: {expected}"
            );
        }

        for forbidden in [
            "Total Views",
            "Total Users",
            "Revenue",
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
