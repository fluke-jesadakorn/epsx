//! Admin pages — 1:1 mirror of `apps/admin-frontend/app/**/page.tsx`.
//!
//! Wave 34 T1 — global SSR skeleton mode. Prod renders a generic
//! skeleton page (`<AuthPageOverlay>` + `<SkeletonPage>`) on every
//! admin route before client-side hydration runs. The dev SSR
//! was rendering real content (which causes a ~83% pixel diff
//! against prod's pre-hydration capture). This dispatch now
//! short-circuits to the skeleton whenever the request is
//! unauthed (`ctx.user.is_none()`) or the BFF is started with
//! `EPSX_E2E_SKELETON=1` (the E2E capture harness sets this so
//! dev matches prod's pre-hydration skeleton byte-for-byte).
//!
//! `/dashboard` and `/policies` are exempt — they return 404 in
//! prod (verified by Wave 34 T1 probe 2026-06-18), so falling
//! through to the existing dispatch renders the AdminAuthGate's
//! "Connect Wallet" page which is closer to a 404 in
//! pixel-diff than a generic skeleton.

use dioxus::prelude::*;
use crate::primitives::*;
use crate::components::admin::auth_page_overlay::{AuthPageOverlay, SkeletonPage};
use super::{PageContext, PageMeta};
use super::not_found;

pub mod dashboard;
pub mod analytics;
pub mod audit_log;
pub mod chat;
pub mod developer_portal;
pub mod media;
pub mod news;
pub mod notifications;
pub mod notifications_redirect;
pub mod payments;
pub mod settings;
pub mod unauthorized;
pub mod auth_redirect;
pub mod auth_page;
pub mod access_denied;
pub mod wallet_redirect;
pub mod wallet_wallets;
pub mod wallet_credits;
pub mod wallet_access;
pub mod wallet_plans;
pub mod policies;

pub fn dispatch(ctx: &PageContext) -> (PageMeta, Element) {
    let p = ctx.path.as_str();

    // Wave 34 T1 — SSR skeleton mode gate. Mirrors prod's
    // pre-hydration skeleton. Triggers on:
    //   1. `ctx.user.is_none()` — real unauthed admin request
    //   2. `EPSX_E2E_SKELETON=1` env var — E2E capture harness
    //
    // Exempts `/dashboard` + `/policies` (they 404 in prod).
    let skeleton_mode = ctx.user.is_none()
        || std::env::var("EPSX_E2E_SKELETON").ok().as_deref() == Some("1");
    if skeleton_mode && !matches!(p, "/dashboard" | "/policies") {
        let slug = slug_for_path(p);
        return (
            PageMeta::admin(slug),
            rsx! {
                AuthPageOverlay { return_url: ctx.path.clone() }
                SkeletonPage { route_slug: slug.to_string() }
            },
        );
    }

    match p {
        "/" | "/index" => dashboard::render(ctx),
        "/analytics" => analytics::render(ctx),
        "/audit-log" => audit_log::render(ctx),
        "/chat" => chat::render(ctx),
        "/developer-portal" => developer_portal::render(ctx),
        "/developer-portal/api-keys/create" => developer_portal::render_create_key(ctx),
        "/media" => media::render(ctx),
        "/news" => news::render(ctx),
        "/news/create" => news::render_create(ctx),
        "/notifications" => notifications_redirect::render(ctx),
        "/notifications/create" => notifications::render_create(ctx),
        "/notifications/manage" => notifications::render_manage(ctx),
        "/payments" => payments::render(ctx),
        "/policies" => policies::render(ctx),
        "/settings" => settings::render(ctx),
        "/unauthorized" => unauthorized::render(ctx),
        "/auth" => auth_page::render(ctx),
        "/access-denied" => access_denied::render(ctx),
        "/wallet-management" => wallet_redirect::render(ctx),
        "/wallet-management/wallets" => wallet_wallets::render(ctx),
        "/wallet-management/credits" => wallet_credits::render(ctx),
        "/wallet-management/access" => wallet_access::render(ctx),
        "/wallet-management/access/plans" => wallet_plans::render(ctx),
        _ => {
            if p.starts_with("/chat/") {
                let id = p.trim_start_matches("/chat/").trim_end_matches('/').to_string();
                let mut c = ctx.clone();
                c.params.insert("id".into(), id);
                chat::render_conversation(&c)
            } else if p.starts_with("/news/") && p.ends_with("/edit") {
                let rest = p.trim_start_matches("/news/").trim_end_matches("/edit").trim_end_matches('/');
                let mut c = ctx.clone();
                c.params.insert("id".into(), rest.to_string());
                news::render_edit(&c)
            } else if p.starts_with("/wallet-management/wallets/") && p.ends_with("/disable") {
                let rest = p.trim_start_matches("/wallet-management/wallets/").trim_end_matches("/disable").trim_end_matches('/');
                let mut c = ctx.clone();
                c.params.insert("address".into(), rest.to_string());
                wallet_wallets::render_disable(&c)
            } else if p.starts_with("/wallet-management/access/plans/") {
                let rest = p.trim_start_matches("/wallet-management/access/plans/").trim_end_matches('/');
                let mut c = ctx.clone();
                c.params.insert("planId".into(), rest.to_string());
                wallet_plans::render_editor(&c)
            } else if p.starts_with("/wallet-management/") {
                let addr = p.trim_start_matches("/wallet-management/").trim_end_matches('/');
                if !addr.is_empty() && !addr.contains('/') {
                    let mut c = ctx.clone();
                    c.params.insert("address".into(), addr.to_string());
                    wallet_wallets::render_detail(&c)
                } else {
                    not_found::render(ctx)
                }
            } else {
                not_found::render(ctx)
            }
        }
    }
}

/// Wave 34 T1 — map an admin path to the route slug used in
/// `tools/e2e-admin/scripts/routes.json`. The slug is forwarded
/// to `<SkeletonPage>` as `route_slug` (a marker / future hook;
/// the current `<SkeletonPage>` rendering is uniform across all
/// slugs).
///
/// For dynamic paths (`/chat/<id>`, `/news/<id>/edit`, etc.) we
/// map the static prefix and use the dynamic ID verbatim to keep
/// the slug readable. Unknown paths fall back to `admin-unknown`.
fn slug_for_path(path: &str) -> &'static str {
    match path {
        "/" => "admin-home",
        "/access-denied" => "admin-access-denied",
        "/unauthorized" => "admin-unauthorized",
        "/auth" => "admin-auth",
        "/dashboard" => "admin-dashboard",
        "/settings" => "admin-settings",
        "/policies" => "admin-policies",
        "/analytics" => "admin-analytics",
        "/audit-log" => "admin-audit-log",
        "/chat" => "admin-chat",
        "/developer-portal" => "admin-developer-portal",
        "/developer-portal/api-keys/create" => "admin-developer-portal-api-keys-create",
        "/media" => "admin-media",
        "/news" => "admin-news",
        "/news/create" => "admin-news-create",
        "/notifications" => "admin-notifications",
        "/notifications/create" => "admin-notifications-create",
        "/notifications/manage" => "admin-notifications-manage",
        "/payments" => "admin-payments",
        "/wallet-management" => "admin-wallet-management",
        "/wallet-management/access" => "admin-wallet-management-access",
        "/wallet-management/access/plans" => "admin-wallet-management-access-plans",
        "/wallet-management/credits" => "admin-wallet-management-credits",
        "/wallet-management/wallets" => "admin-wallet-management-wallets",
        _ => {
            if path.starts_with("/chat/") {
                "admin-chat-sample-id"
            } else if path.starts_with("/news/") && path.ends_with("/edit") {
                "admin-news-sample-id-edit"
            } else if path.starts_with("/wallet-management/access/plans/") {
                "admin-wallet-management-access-plans-sample-plan-id"
            } else if path.starts_with("/wallet-management/wallets/") && path.ends_with("/disable") {
                "admin-wallet-management-wallets-sample-address-disable"
            } else if path.starts_with("/wallet-management/wallets/") {
                "admin-wallet-management-wallets"
            } else if path.starts_with("/wallet-management/") {
                "admin-wallet-management-sample-address"
            } else {
                "admin-unknown"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug_for_path_known_routes() {
        assert_eq!(slug_for_path("/"), "admin-home");
        assert_eq!(slug_for_path("/settings"), "admin-settings");
        assert_eq!(slug_for_path("/news/create"), "admin-news-create");
        assert_eq!(slug_for_path("/wallet-management/credits"), "admin-wallet-management-credits");
    }

    #[test]
    fn test_slug_for_path_dynamic_routes() {
        assert_eq!(slug_for_path("/chat/sample-conv-id"), "admin-chat-sample-id");
        assert_eq!(slug_for_path("/news/sample-id/edit"), "admin-news-sample-id-edit");
        assert_eq!(
            slug_for_path("/wallet-management/access/plans/sample-plan-id"),
            "admin-wallet-management-access-plans-sample-plan-id"
        );
        assert_eq!(
            slug_for_path("/wallet-management/wallets/0x0000/disable"),
            "admin-wallet-management-wallets-sample-address-disable"
        );
        assert_eq!(
            slug_for_path("/wallet-management/0x0000d3c0"),
            "admin-wallet-management-sample-address"
        );
    }

    #[test]
    fn test_slug_for_path_unknown_falls_back() {
        assert_eq!(slug_for_path("/no-such-route"), "admin-unknown");
        assert_eq!(slug_for_path(""), "admin-unknown");
    }
}
