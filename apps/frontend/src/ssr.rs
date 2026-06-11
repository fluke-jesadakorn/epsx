//! Dioxus SSR rendering for the frontend BFF.
//!
//! Server-side fetches happen in this layer: the page request comes in,
//! the BFF resolves the verified user, optionally fetches page-specific
//! data from the gateway (cached in the SSR result), and renders the
//! Dioxus VNode with all data baked in. The page renders fully on the
//! server; client-side hydration then takes over for interactivity.

use axum::{
    extract::{Request, State},
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
        roles: u.roles,
        email: None,
        tier: None,
        permissions: vec![],
        // Wave 2 Track C — auth metadata fields. The frontend BFF
        // doesn't have rich auth metadata, so we leave the new
        // optional fields at their defaults.
        last_login_at: None,
        auth_method: AuthMethod::Wallet,
        display_name: None,
    });

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
