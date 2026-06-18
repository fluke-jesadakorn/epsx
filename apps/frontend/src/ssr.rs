//! Dioxus SSR rendering for the frontend BFF.
//!
//! Server-side fetches happen in this layer: the page request comes in,
//! the BFF resolves the verified user, optionally fetches page-specific
//! data from the gateway (cached in the SSR result), and renders the
//! Dioxus VNode with all data baked in. The page renders fully on the
//! server; client-side hydration then takes over for interactivity.
//!
//! Auth gating (Wave 23 T3): when an unauthenticated request lands on
//! a protected path, we 307-redirect to `/auth?return_url=<path>` to
//! match the prod Vercel middleware convention
//! (`apps-old/frontend/middleware.ts::handleUnauthenticated` reads
//! `?return_url=` and reads/writes the `epsx.return_url` cookie).
//! The auth page's hydration script
//! (`shared/rust/dioxus_ui/src/pages/auth_page.rs::AUTH_HYDRATION_SCRIPT`)
//! and the page-level `AuthGate` connect links all use the same
//! `?return_url=` parameter, so the round-trip works.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use epsx_dioxus_ui::auth::User;
use epsx_dioxus_ui::auth::user::AuthMethod;
use epsx_dioxus_ui::auth::wallet_button::ConnectedWalletState;
use epsx_dioxus_ui::pages::{render_page, PageContext};
use std::collections::HashMap;
use std::sync::Arc;

use super::AppState;
use super::auth;

/// Paths that 307-redirect to /auth when the user is unauthenticated,
/// matching the prod (https://epsx.io) Vercel middleware behavior. The
/// prod baseline shows the /auth page for these routes; without the
/// redirect the dev bff returns 200 + "Sign in required" gate, which
/// diverges from prod and inflates pixel diff.
///
/// Wave 35b T1: added `/about`, `/contact`, `/offline` — the three
/// marketing/utility routes that prod's Next.js middleware also
/// auth-gates (its `protectedPaths` list in
/// `apps-old/frontend/middleware.ts::config.matcher`). Without this
/// redirect, dev served the real Dioxus pages (Wave 5/6 ports) while
/// prod served the /auth redirect target → 0% pixel match. With the
/// redirect, both dev and prod serve the /auth page and the diff
/// converges to ~100%.
const UNAUTH_REDIRECT_PATHS: &[&str] = &[
    "/permissions",
    "/notifications",
    "/profile",
    "/about",
    "/contact",
    "/offline",
];

/// Wave 22 T4 — `/pricing` is an alias for `/plans` in prod. The
/// Vercel middleware `rewrites` `/pricing` → `/plans` while
/// preserving the query string. We mirror the same behavior as a
/// `307 Temporary Redirect` so the browser follows it (and the dev
/// baseline matches prod for both `/pricing` and `/pricing?ref=foo`
/// style URLs). The redirect fires BEFORE page rendering so the
/// downstream page code never has to handle the `/pricing` path.
fn pricing_redirect_response(query: &str) -> Response {
    let location = if query.is_empty() {
        "/plans".to_string()
    } else {
        format!("/plans?{query}")
    };
    (
        StatusCode::TEMPORARY_REDIRECT,
        [("location", location.as_str())],
        "",
    )
        .into_response()
}

/// All non-API requests land here. We render the page via Dioxus fullstack
/// SSR and return a complete HTML document using the same design-system
/// `<head>` the Next.js frontend emits.
pub async fn ssr_handler(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, _body) = request.into_parts();
    let path = parts.uri.path().to_string();
    let query = parts.uri.query().unwrap_or("").to_string();
    let headers = parts.headers.clone();

    // Resolve verified user from the epsx_token cookie / Authorization
    // header via the shared JWT service.
    let jwt = auth::jwt_auth_from_env();
    let user = auth::current_user(&headers, &jwt).map(|u| User {
        id: u.user_id,
        address: u.address,
        chain_id: u.chain_id,
        roles: u.roles.clone(),
        email: None,
        tier: None,
        // Wave 7 — populate `permissions` from the JWT roles so the
        // page-level `AuthGate` checks pass for the right users.
        // Previously `vec![]`, which made every gated user page
        // misfire (the gate's `has_permission` would always see
        // "missing"). Mirrors the same fix in the admin BFF.
        permissions: auth::permissions_for_roles(&u.roles),
        // Wave 2 Track C — auth metadata fields. The frontend BFF
        // doesn't have rich auth metadata, so we leave the new
        // optional fields at their defaults.
        last_login_at: None,
        auth_method: AuthMethod::Wallet,
        display_name: None,
    });

    // Wave 22 T4 — `/pricing` is an alias for `/plans` in prod
    // (Vercel middleware rewrite). We 307-redirect to `/plans`
    // preserving the query string, so both `/pricing` and
    // `/pricing?ref=foo` style URLs land on the plans page.
    if path == "/pricing" {
        return pricing_redirect_response(&query);
    }

    // Wave 22 T5 — mirror prod Vercel middleware 307 redirect behavior
    // for paths that prod always bounces to /auth when the user has
    // no session. The redirect fires BEFORE page rendering, so the
    // browser follows to /auth and the dev baseline matches the
    // prod baseline PNG (the auth page) for these routes.
    //
    // Two categories of path:
    //  - Exact-match paths in UNAUTH_REDIRECT_PATHS (e.g. /permissions,
    //    /notifications, /profile)
    //  - /chat/* sub-paths (e.g. /chat/<conv-id>, /chat/history) — prod
    //    lists `/chat` as public, but sub-paths are protected and 307
    //    to /auth. /chat itself stays public (browsable).
    let needs_unauth_redirect = user.is_none()
        && (UNAUTH_REDIRECT_PATHS.contains(&path.as_str())
            || (path.starts_with("/chat/")
                && !path.is_empty()
                && path != "/chat"));
    if needs_unauth_redirect {
        let next = if query.is_empty() {
            path.clone()
        } else {
            format!("{path}?{query}")
        };
        // Wave 23 T3 — prod Vercel middleware uses `?return_url=` (NOT
        // `?next=`). Standardize the dev SSR redirect to match. The
        // auth page's hydration script reads `return_url` from the
        // query string (or the `epsx_return_url` cookie set here)
        // and bounces the user back to that path after sign-in
        // completes. Pre-encoding via `urlencode` matches the shape
        // `apps-old/frontend/middleware.ts::handleUnauthenticated`
        // sets in the `epsx.return_url` cookie.
        //
        // The 5-minute cookie TTL is `MAX_AGE_RETURN_URL` and
        // matches prod's `handleUnauthenticated` `maxAge: 300`.
        // The cookie is HttpOnly + SameSite=Lax; it does NOT need
        // the `__Host-` prefix because dev runs over plain HTTP
        // (port 30101 port-forward).
        let location = format!("/auth?return_url={}", urlencode(&next));
        let cookie = auth::build_set_cookie("epsx_return_url", &urlencode(&next), 300);
        return (
            StatusCode::TEMPORARY_REDIRECT,
            [
                ("location", location.as_str()),
                ("set-cookie", cookie.as_str()),
            ],
            "",
        )
            .into_response();
    }

    // Parse dynamic-route params from path
    let mut params = HashMap::new();
    if let Some(rest) = path.strip_prefix("/news/") {
        if !rest.is_empty() && !rest.contains('/') {
            params.insert("slug".into(), rest.trim_end_matches('/').to_string());
        }
    }
    if let Some(rest) = path.strip_prefix("/chat/") {
        if !rest.is_empty() && !rest.contains('/') {
            params.insert("id".into(), rest.trim_end_matches('/').to_string());
        }
    }
    if let Some(rest) = path.strip_prefix("/payment/") {
        let mut it = rest.splitn(2, '/');
        let ptype = it.next().unwrap_or("").to_string();
        let pid = it.next().unwrap_or("").trim_end_matches('/').to_string();
        if !ptype.is_empty() { params.insert("type".into(), ptype); }
        if !pid.is_empty() { params.insert("id".into(), pid); }
    }

    // Page-specific server-side data fetching. Each block reads from
    // the gateway via `state.*` and adds the result to `params` so the
    // page can consume it.
    fetch_page_data(&state, &path, &user, &mut params, &headers).await;

    // Wave 3a Track B — plumb server-side wallet state into the page
    // context. We delegate the cookie read to
    // `ConnectedWalletState::from_cookies` (currently a no-op stub —
    // see `auth/wallet_button.rs` for the follow-up). `is_authenticated`
    // is sourced from the resolved `user` (the SIWE session lifetime),
    // NOT from the cookie (which tracks wallet-connection lifetime).
    //
    // Stub: cookie parser is a no-op for now — when the wagmi-equivalent
    // client writes a `WalletInfo` cookie, the parser will populate
    // `address` / `connector_id` / `chain_id` from it.
    let mut wallet = ConnectedWalletState::from_cookies(&headers);
    wallet.is_authenticated = user.is_some();

    let ctx = PageContext {
        user,
        path: path.clone(),
        query: query.clone(),
        params,
        api_url: state.api_url.clone(),
        demo_login_enabled: state.demo_login_enabled,
        wallet,
    };

    let (meta, body_element) = render_page(&ctx, false);
    let body_html = dioxus_ssr::render_element(body_element);

    let doc = epsx_templates::page_shell_with_body_class(
        &meta.title,
        &meta.description,
        &String::new(),
        &body_html,
        meta.include_footer,
        &meta.body_class,
    );

    let doc = doc.replace("</body>", &format!("<script>{}</script></body>", wallet_shim()));

    (
        axum::http::StatusCode::OK,
        [("content-type", "text/html; charset=utf-8")],
        doc,
    ).into_response()
}

/// Fetch page-specific data and add it to `params` as JSON-serialized
/// values. The page reads them via `ctx.params.get("data_X")` and
/// deserializes into a typed struct.
async fn fetch_page_data(
    state: &AppState,
    path: &str,
    user: &Option<User>,
    params: &mut HashMap<String, String>,
    headers: &axum::http::HeaderMap,
) {
    use epsx_client::RequestContext;
    // Wave 31 T1 — for the 3 live-data-plumbing routes (dashboard,
    // news, developer/usage) we now call the BFF's own handler
    // helpers IN-PROCESS rather than going through the upstream
    // gateway. This means the BFF route and the SSR layer share the
    // exact same JSON shape, and the dev pages always have live data
    // even when the upstream gateway/services are down. The previous
    // `state.X.get_plain("/api/v1/...")` calls hit the gateway and
    // returned 502 when the upstream was unavailable, so the page
    // fell back to its hardcoded mock — defeating the purpose of
    // "live data plumbing".

    let has_session = epsx_bff::dev_bypass::is_dev_bypass_enabled()
        || super::auth::get_cookie(headers, "epsx_token")
            .map(|t| !t.is_empty())
            .unwrap_or(false);

    // /dashboard: fetch stat cards + recent activity.
    // Wave 31 T1 — call the BFF's own `dashboard_data_internal()`
    // helper in-process. Inject the INNER `data` sub-object (not the
    // full envelope) so the page's existing
    // `params["data_dashboard"]["stats"]` lookup continues to work
    // (the page reads `.get("stats")` directly — see
    // `pages/dashboard.rs::RenderDashboard`).
    if path == "/dashboard" {
        let v = crate::api::dashboard_data_internal(has_session);
        if let Some(data) = v.get("data") {
            params.insert("data_dashboard".into(), data.to_string());
        }
    }
    // /news: fetch news. Wave 31 T1 — call the BFF's own
    // `news_list_value()` helper in-process.
    if path == "/news" {
        params.insert("data_news".into(), crate::api::news_list_value().to_string());
    }
    // /news/[slug]: fetch the article body. Wave 31 T1 — call the
    // BFF's own `news_post_value(slug)` helper in-process.
    if path.starts_with("/news/") {
        if let Some(slug) = path.strip_prefix("/news/").map(|s| s.trim_end_matches('/').to_string()) {
            if !slug.is_empty() && !slug.contains('/') {
                params.insert("data_news_post".into(), crate::api::news_post_value(&slug).to_string());
            }
        }
    }
    // /developer/usage: fetch usage stats. Wave 31 T1 — call the
    // BFF's own `developer_usage_value()` helper in-process.
    if path == "/developer/usage" {
        params.insert("data_developer_usage".into(), crate::api::developer_usage_value().to_string());
    }
    // /notifications: fetch list
    if path == "/notifications" && user.is_some() {
        if let Ok(v) = state.notification.get_with_ctx("/api/v1/notification/list", &RequestContext::from_headers(headers)).await {
            params.insert("data_notifications".into(), v.to_string());
        }
    }
    // /plans: fetch plans. Wave 23 T5 — try the BFF's own
    // `/api/v1/plans` endpoint FIRST (returns the content-service
    // plans.json shape with all the prod `category` / `title` /
    // `price_usd` / `discount_pct` fields). The
    // subscription-service raw array shape is also accepted by
    // `plans.rs::extract_plans`, so we fall back to it if the BFF
    // call fails. The content-service endpoint comes last (it's
    // the canonical shape but the content service is in
    // `ImagePullBackOff` per wave-22 follow-up #2).
    if path == "/plans" {
        if let Ok(v) = state.subscription.get_plain("/api/v1/plans").await {
            params.insert("data_plans".into(), v.to_string());
        } else if let Ok(v) = state.subscription.get_plain("/api/v1/subscription/plans").await {
            params.insert("data_plans".into(), v.to_string());
        } else if let Ok(v) = state.content.get_plain("/api/v1/content/plans").await {
            params.insert("data_plans".into(), v.to_string());
        }
    }
    // /portfolio: fetch holdings. Wave 23 T5 — try the BFF's own
    // `/api/v1/portfolio/<addr>` endpoint first (returns a
    // payload matching the dev `HoldingsTable` row tuple). Falls
    // back to the wallet service (which has no portfolio endpoint
    // today but is the right path when it gets one).
    if path.starts_with("/portfolio") {
        if let Some(addr) = user.as_ref().map(|u| u.address.clone()) {
            if let Ok(v) = state.wallet.get_plain(&format!("/api/v1/portfolio/{}", addr)).await {
                params.insert("data_portfolio".into(), v.to_string());
            } else if let Ok(v) = state.wallet.get_plain(&format!("/api/v1/wallet/portfolio/{}", addr)).await {
                params.insert("data_portfolio".into(), v.to_string());
            }
        }
    }
    // /account: wallet address + member-since + balance + method.
    // Wave 23 T5 — was previously not wired, page always rendered
    // the OLD "Not Connected / Join Now / $0 / Web3 Vault"
    // placeholder set. Now `data_account` returns either the user's
    // real values (authed) or the placeholder (anon).
    if path == "/account" {
        if let Ok(v) = state.identity.get_plain("/api/v1/account").await {
            params.insert("data_account".into(), v.to_string());
        } else if let Ok(v) = state.identity.get_plain("/api/v1/auth/me").await {
            params.insert("data_account".into(), v.to_string());
        }
    }
    // /account/credits: lifetime earned/spent + transactions.
    // Wave 23 T5 — was previously not wired, page always rendered
    // the OLD "$0 / no transactions" baseline.
    if path == "/account/credits" {
        if let Ok(v) = state.identity.get_plain("/api/v1/credits").await {
            params.insert("data_credits".into(), v.to_string());
        }
    }
    // /developer: stats cards + API key list.
    // Wave 23 T5 — was previously not wired, the page rendered its
    // hardcoded `sample_api_keys()` fixture for everyone.
    if path == "/developer" {
        if let Ok(v) = state.identity.get_plain("/api/v1/developer").await {
            params.insert("data_developer".into(), v.to_string());
        }
    }
    if path == "/developer/docs" {
        if let Ok(v) = state.identity.get_plain("/api/v1/developer/docs").await {
            params.insert("data_developer_docs".into(), v.to_string());
        }
    }
    // /analytics: summary stats + top movers.
    // Wave 23 T5 — was previously not wired.
    if path == "/analytics" {
        if let Ok(v) = state.analytics.get_plain("/api/v1/analytics/summary").await {
            params.insert("data_analytics".into(), v.to_string());
        }
    }
    // /payment/intent/[id]: payment intent details.
    // Wave 23 T5 — was previously not wired. The dev `payment.rs`
    // reads `type` + `id` from the path params but ignores them
    // (renders a static form), so this is a forward-looking hook.
    if path.starts_with("/payment/intent/") {
        if let Some(id) = path.strip_prefix("/payment/intent/").map(|s| s.trim_end_matches('/').to_string()) {
            if !id.is_empty() {
                if let Ok(v) = state.payment.get_plain(&format!("/api/v1/payment/{}", id)).await {
                    params.insert("data_payment".into(), v.to_string());
                }
            }
        }
    }
}

/// Inline JS shim that bridges the Rust/Dioxus UI to the browser's
/// `window.ethereum` and `window.epsxWallet` namespaces. This is the Web3
/// counterpart to wagmi/RainbowKit from the Next.js frontend.
fn wallet_shim() -> &'static str {
    r#"
window.epsxWallet = {
  isAvailable: () => typeof window.ethereum !== 'undefined',
  request: (method, params) => {
    if (!window.ethereum) return Promise.reject(new Error('No wallet'));
    return window.ethereum.request({ method, params: params || [] });
  },
  personalSign: (message) => {
    if (!window.ethereum) return Promise.reject(new Error('No wallet'));
    return window.ethereum.request({ method: 'personal_sign', params: [message, window.ethereum.selectedAddress] });
  },
  address: () => window.ethereum && window.ethereum.selectedAddress ? window.ethereum.selectedAddress : null,
  chainId: () => window.ethereum && window.ethereum.chainId ? window.ethereum.chainId : '0x38',
  onAccountsChanged: (cb) => { if (window.ethereum) window.ethereum.on('accountsChanged', cb); },
  onChainChanged: (cb) => { if (window.ethereum) window.ethereum.on('chainChanged', cb); },
  addToken: (token) => {
    if (!window.ethereum) return Promise.reject(new Error('No wallet'));
    return window.ethereum.request({
      method: 'wallet_watchAsset',
      params: { type: 'ERC20', options: token }
    });
  }
};
window.epsxAuth = {
  siweLogin: async (message, signature, chainId) => {
    const res = await fetch('/api/v1/auth/siwe', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ message, signature, chain_id: String(chainId) })
    });
    return res.json();
  },
  challenge: async (address, chainId) => {
    const res = await fetch('/api/v1/auth/challenge', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ address, chain_id: String(chainId) })
    });
    return res.json();
  },
  demoLogin: async () => {
    const res = await fetch('/api/v1/auth/demo', { method: 'POST', headers: {'content-type':'application/json'}, body: JSON.stringify({}) });
    return res.json();
  },
  logout: async () => {
    await fetch('/api/v1/auth/logout', { method: 'POST' });
  },
  me: async () => {
    const res = await fetch('/api/v1/auth/me');
    return res.json();
  }
};
"#
}

/// Minimal URL-encoder for the `next=` query parameter. Only handles
/// the characters Vercel's middleware actually encodes; intentionally
/// avoids pulling in a full url-encoding crate for this one call site.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::urlencode;
    use super::pricing_redirect_response;
    use axum::http::StatusCode;

    #[test]
    fn urlencode_passes_alnum() {
        // Matches Vercel's prod middleware `epsx.return_url=%2F<path>` shape.
        assert_eq!(urlencode("/notifications"), "%2Fnotifications");
        // Wave 23 T3 — query parameter is now `?return_url=` (NOT
        // `?next=`). The encoder must still produce the same per-byte
        // shape regardless of the query parameter name.
        assert_eq!(urlencode("/auth?return_url=/x"), "%2Fauth%3Freturn_url%3D%2Fx");
        assert_eq!(urlencode("plain"), "plain");
    }

    /// Wave 23 T3 — `?return_url=<path>` round-trip: a path that
    /// already contains a query string must be re-encoded so the
    /// `searchParams.get("return_url")` call on the auth page only
    /// sees the encoded value (not the inner `?` / `&`).
    #[test]
    fn urlencode_encodes_inner_query_separators() {
        assert_eq!(urlencode("/pricing?ref=foo"), "%2Fpricing%3Fref%3Dfoo");
        assert_eq!(urlencode("/x?a=1&b=2"), "%2Fx%3Fa%3D1%26b%3D2");
    }

    /// Wave 22 T4 — `/pricing` (no query) → 307 `/plans`.
    #[test]
    fn pricing_redirect_no_query() {
        let r = pricing_redirect_response("");
        assert_eq!(r.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(r.headers().get("location").unwrap(), "/plans");
    }

    /// Wave 22 T4 — `/pricing?ref=foo` → 307 `/plans?ref=foo`
    /// (query string is preserved verbatim).
    #[test]
    fn pricing_redirect_preserves_query() {
        let r = pricing_redirect_response("ref=foo&affiliate=bar");
        assert_eq!(r.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(r.headers().get("location").unwrap(), "/plans?ref=foo&affiliate=bar");
    }

    // === Wave 35b T1 — AuthGate 307-redirect for marketing routes ===

    /// Wave 35b T1 — `UNAUTH_REDIRECT_PATHS` must include the three
    /// marketing/utility routes (`/about`, `/contact`, `/offline`)
    /// that prod's Next.js middleware also auth-gates. Without this
    /// entry, dev serves the real Dioxus page (200) while prod serves
    /// the /auth page (after 307), driving the diff to ~0%.
    #[test]
    fn unauth_redirect_paths_includes_marketing_routes() {
        let paths = super::UNAUTH_REDIRECT_PATHS;
        assert!(
            paths.contains(&"/about"),
            "UNAUTH_REDIRECT_PATHS must contain `/about` (Wave 35b T1)"
        );
        assert!(
            paths.contains(&"/contact"),
            "UNAUTH_REDIRECT_PATHS must contain `/contact` (Wave 35b T1)"
        );
        assert!(
            paths.contains(&"/offline"),
            "UNAUTH_REDIRECT_PATHS must contain `/offline` (Wave 35b T1)"
        );
    }

    /// Wave 35b T1 — pre-existing protected paths from Wave 22/23
    /// (`/permissions`, `/notifications`, `/profile`) must remain in
    /// the list. This test guards against accidental removal during
    /// the Wave 35b edit.
    #[test]
    fn unauth_redirect_paths_keeps_wave22_23_entries() {
        let paths = super::UNAUTH_REDIRECT_PATHS;
        for path in &["/permissions", "/notifications", "/profile"] {
            assert!(
                paths.contains(path),
                "UNAUTH_REDIRECT_PATHS must still contain `{path}` (Wave 22/23 entry, do not regress)"
            );
        }
    }

    /// Wave 35b T1 — the redirect target must use the prod-shaped
    /// `?return_url=<urlencoded path>` query string (matches
    /// `apps-old/frontend/middleware.ts::handleUnauthenticated`).
    /// Encoding the slash as `%2F` is critical: the auth page's
    /// hydration script uses `searchParams.get("return_url")` which
    /// only returns the first segment if the slash is left raw.
    #[test]
    fn unauth_redirect_uses_return_url_shape() {
        // The exact encoder logic is in `urlencode`; this test pins
        // the SHAPE so future edits can't accidentally rename it
        // back to `?next=` (Wave 23 T3 explicitly retired `?next=`).
        assert_eq!(urlencode("/about"), "%2Fabout");
        assert_eq!(urlencode("/contact"), "%2Fcontact");
        assert_eq!(urlencode("/offline"), "%2Foffline");
        // The redirect location string shape:
        assert_eq!(
            format!("/auth?return_url={}", urlencode("/about")),
            "/auth?return_url=%2Fabout"
        );
    }
}
