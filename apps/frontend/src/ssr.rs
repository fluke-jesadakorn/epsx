//! Dioxus SSR rendering for the frontend BFF.
//!
//! Server-side fetches happen in this layer: the page request comes in,
//! the BFF resolves the verified user, optionally fetches page-specific
//! data from the gateway (cached in the SSR result), and renders the
//! Dioxus VNode with all data baked in. The page renders fully on the
//! server; client-side hydration then takes over for interactivity.

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
const UNAUTH_REDIRECT_PATHS: &[&str] = &[
    "/permissions",
    "/notifications",
    "/profile",
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
        let location = format!("/auth?next={}", urlencode(&next));
        return (
            StatusCode::TEMPORARY_REDIRECT,
            [("location", location.as_str())],
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
    let _ = user; // suppress unused warning until more endpoints are wired

    // /dashboard: fetch stat cards + recent activity
    if path == "/dashboard" && user.is_some() {
        if let Ok(v) = state.analytics.get_with_ctx("/api/v1/analytics/metrics/dashboard", &RequestContext::from_headers(headers)).await {
            params.insert("data_dashboard".into(), v.to_string());
        }
    }
    // /notifications: fetch list
    if path == "/notifications" && user.is_some() {
        if let Ok(v) = state.notification.get_with_ctx("/api/v1/notification/list", &RequestContext::from_headers(headers)).await {
            params.insert("data_notifications".into(), v.to_string());
        }
    }
    // /plans: fetch plans
    if path == "/plans" {
        if let Ok(v) = state.subscription.get_plain("/api/v1/subscription/plans").await {
            params.insert("data_plans".into(), v.to_string());
        }
    }
    // /news: fetch news
    if path == "/news" {
        if let Ok(v) = state.content.get_plain("/api/v1/content/news").await {
            params.insert("data_news".into(), v.to_string());
        }
    }
    // /portfolio: fetch holdings
    if path.starts_with("/portfolio") && user.is_some() {
        if let Some(addr) = user.as_ref().map(|u| u.address.clone()) {
            if let Ok(v) = state.wallet.get_plain(&format!("/api/v1/wallet/portfolio/{}", addr)).await {
                params.insert("data_portfolio".into(), v.to_string());
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
        assert_eq!(urlencode("/auth?next=/x"), "%2Fauth%3Fnext%3D%2Fx");
        assert_eq!(urlencode("plain"), "plain");
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
}
