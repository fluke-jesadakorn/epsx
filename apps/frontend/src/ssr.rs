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
//! The shared wallet bridge and page-level `AuthGate` connect links use the
//! same `?return_url=` parameter, so the round-trip remains same-origin.

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use epsx_dioxus_ui::auth::wallet_button::ConnectedWalletState;
use epsx_dioxus_ui::auth::User;
use epsx_dioxus_ui::pages::{is_known_frontend_route, render_page, PageContext, PageStatus};
use std::collections::HashMap;

use super::auth;
use super::AppState;

/// Paths that 307-redirect to /auth when the user is unauthenticated,
/// matching the prod (https://epsx.io) Vercel middleware behavior. The
/// prod baseline shows the /auth page for these routes; without the
/// redirect the dev bff returns 200 + "Sign in required" gate, which
/// diverges from prod and inflates pixel diff.
///
/// Wave 35b T1 added `/about`, `/contact`, and `/offline` for pixel
/// parity with the pinned middleware. B7 removes `/offline`: a PWA
/// recovery surface must remain reachable without a session, especially
/// when authentication cannot complete because the browser is disconnected.
const UNAUTH_REDIRECT_PATHS: &[&str] = &[
    "/permissions",
    "/notifications",
    "/profile",
    "/about",
    "/contact",
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
pub async fn ssr_handler(State(state): State<AppState>, request: Request) -> Response {
    let (parts, _body) = request.into_parts();
    let path = parts.uri.path().to_string();
    let query = parts.uri.query().unwrap_or("").to_string();
    let headers = parts.headers.clone();

    let mut wallet = ConnectedWalletState::from_cookies(&headers);
    let verified_session =
        auth::verified_access_token(&headers, state.verifier.as_ref(), state.cookie_environment)
            .await;
    let (verified_access_token, user) = match verified_session {
        Some((token, session)) => (Some(token), Some(auth::ui_user(session, wallet.chain_id))),
        None => (None, None),
    };

    // Wave 22 T4 — `/pricing` is an alias for `/plans` in prod
    // (Vercel middleware rewrite). We 307-redirect to `/plans`
    // preserving the query string, so both `/pricing` and
    // `/pricing?ref=foo` style URLs land on the plans page.
    if path == "/pricing" {
        return pricing_redirect_response(&query);
    }

    let route_is_known = is_known_frontend_route(&path);

    if path == "/auth" && user.is_some() {
        return (
            StatusCode::TEMPORARY_REDIRECT,
            [("location", safe_return_url(&query))],
            "",
        )
            .into_response();
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
    let needs_unauth_redirect = route_is_known
        && user.is_none()
        && (UNAUTH_REDIRECT_PATHS.contains(&path.as_str())
            || (path.starts_with("/chat/") && !path.is_empty() && path != "/chat"));
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
        let location = format!("/auth?return_url={}", urlencode(&next));
        return (StatusCode::TEMPORARY_REDIRECT, [("location", location)], "").into_response();
    }

    // Parse dynamic-route params from path
    let mut params = HashMap::new();
    if let Some(rest) = path.strip_prefix("/news/") {
        if !rest.is_empty() && !rest.contains('/') {
            params.insert("slug".into(), rest.to_string());
        }
    }
    if let Some(rest) = path.strip_prefix("/chat/") {
        if !rest.is_empty() && !rest.contains('/') {
            params.insert("id".into(), rest.to_string());
        }
    }
    if let Some(rest) = path.strip_prefix("/payment/") {
        let mut it = rest.splitn(2, '/');
        let ptype = it.next().unwrap_or("").to_string();
        let pid = it.next().unwrap_or("").to_string();
        if !ptype.is_empty() && !pid.is_empty() && !pid.contains('/') {
            params.insert("type".into(), ptype);
            params.insert("id".into(), pid);
        }
    }

    // Page-specific server-side data fetching. Each block reads from
    // the gateway via `state.*` and adds the result to `params` so the
    // page can consume it.
    if route_is_known {
        fetch_page_data(
            &state,
            &path,
            &user,
            &mut params,
            &headers,
            verified_access_token.as_deref(),
        )
        .await;
    }

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
    let status = match meta.status {
        PageStatus::Ok => StatusCode::OK,
        PageStatus::NotFound => StatusCode::NOT_FOUND,
    };
    let body_html = dioxus_ssr::render_element(body_element);

    // === Wave 49+ — SSR-safe navbar ===
    //
    // The Dioxus `<NavigationClient>` / `<GroupDropdown>` chain uses
    // Dioxus `onclick:` closures to toggle the desktop dropdowns.
    // Dioxus 0.7 SSR is hydration-less — those closures are stripped
    // from the rendered HTML, so clicking "Market / Developer /
    // Company" does nothing and the user can never reach the
    // Rankings / Portfolio / Developer Portal / About / News /
    // Contact / Support pages from the navbar.
    //
    // The fix: use `epsx_templates::epsx_header()` (the static HTML
    // sticky header that mirrors prod's `epsx.io` NavMenu) which
    // emits raw `onclick="epsx.toggleNav(this)"` attributes that the
    // `global_js()` controller already understands. The dropdown
    // menu items are rendered unconditionally (visibility is
    // controlled by CSS `.epsx-nav-wrap.open .epsx-nav-menu { display:
    // block; }`) — so every link is in the DOM and clickable.
    //
    // The auth page hides the navbar via the `path == "/auth"`
    // short-circuit (the dedicated `<AuthLayout>` is full-bleed).
    let nav_html = if path == "/auth" {
        String::new()
    } else {
        epsx_templates::epsx_header()
    };

    // === Wave 49+ — re-enable footer ===
    //
    // Wave 38c dropped `PageMeta::include_footer` to `false` (and
    // removed `<Footer />` from `<MainLayout>`) to fix a structural
    // double-footer on marketing pages. That fix left the public
    // site with NO footer at all — Terms / Privacy / About /
    // Contact / Rankings / Portfolio / Pricing / API Keys /
    // Documentation / Support / News links were unreachable from
    // the footer on every page except `/terms` (which has its own
    // page-local `TermsFooter`).
    //
    // We force the templates `footer()` on at the BFF layer so
    // every page gets a clickable 4-column footer with the same
    // links the templates navbar exposes. The Dioxus `<Footer />`
    // is no longer rendered by `<MainLayout>`, so there is no
    // double-footer risk.
    let include_footer = true;

    let doc = epsx_templates::page_shell_with_body_class_and_keywords(
        &meta.title,
        &meta.description,
        meta.keywords.as_deref(),
        &nav_html,
        &body_html,
        include_footer,
        meta.body_class.as_deref().unwrap_or(""),
    );

    let route_runtime = match path.as_str() {
        "/offline" => offline_runtime_script(),
        "/manual" => manual_runtime_script(),
        "/developer/docs" => developer_docs_runtime_script(),
        _ => "",
    };
    let doc = doc.replace(
        "</body>",
        &format!(
            "<script>{}</script>{}{route_runtime}</body>",
            wallet_shim(),
            offline_worker_registration_script()
        ),
    );

    let mut response =
        (status, [("content-type", "text/html; charset=utf-8")], doc).into_response();
    if path == "/offline" {
        // This marker is the service worker's fail-closed permission to cache
        // the response. The worker fetches it with credentials omitted and
        // refuses any response that loses this exact public-shell contract.
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=0, must-revalidate"),
        );
        response.headers_mut().insert(
            "x-epsx-public-cache",
            HeaderValue::from_static("offline-shell-v1"),
        );
    }
    response
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
    verified_access_token: Option<&str>,
) {
    use epsx_client::RequestContext;
    let mut request_context = RequestContext::from_headers(headers);
    // Browser cookies are not authorization headers. After local RS256/JWKS
    // verification, explicitly forward the canonical token as a bearer value.
    request_context.auth_token = verified_access_token.map(str::to_owned);
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

    let has_session = user.is_some();

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
        params.insert(
            "data_news".into(),
            crate::api::news_list_value().to_string(),
        );
    }
    // /news/[slug]: fetch the article body. Wave 31 T1 — call the
    // BFF's own `news_post_value(slug)` helper in-process.
    if path.starts_with("/news/") {
        if let Some(slug) = path
            .strip_prefix("/news/")
            .map(|s| s.trim_end_matches('/').to_string())
        {
            if !slug.is_empty() && !slug.contains('/') {
                params.insert(
                    "data_news_post".into(),
                    crate::api::news_post_value(&slug).to_string(),
                );
            }
        }
    }
    // /developer/usage: fetch usage stats. Wave 31 T1 — call the
    // BFF's own `developer_usage_value()` helper in-process.
    if path == "/developer/usage" {
        params.insert(
            "data_developer_usage".into(),
            crate::api::developer_usage_value().to_string(),
        );
    }
    // /notifications: fetch list
    if path == "/notifications" && user.is_some() {
        if let Ok(v) = state
            .notification
            .get_with_ctx("/api/v1/notification/list", &request_context)
            .await
        {
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
        } else if let Ok(v) = state
            .subscription
            .get_plain("/api/v1/subscription/plans")
            .await
        {
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
            if let Ok(v) = state
                .wallet
                .get_with_ctx(&format!("/api/v1/portfolio/{}", addr), &request_context)
                .await
            {
                params.insert("data_portfolio".into(), v.to_string());
            } else if let Ok(v) = state
                .wallet
                .get_with_ctx(
                    &format!("/api/v1/wallet/portfolio/{}", addr),
                    &request_context,
                )
                .await
            {
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
        if let Ok(v) = state
            .identity
            .get_with_ctx("/api/v1/account", &request_context)
            .await
        {
            params.insert("data_account".into(), v.to_string());
        } else if let Ok(v) = state
            .identity
            .get_with_ctx("/api/v1/auth/me", &request_context)
            .await
        {
            params.insert("data_account".into(), v.to_string());
        }
    }
    // /account/credits: lifetime earned/spent + transactions.
    // Wave 23 T5 — was previously not wired, page always rendered
    // the OLD "$0 / no transactions" baseline.
    if path == "/account/credits" {
        if let Ok(v) = state
            .identity
            .get_with_ctx("/api/v1/credits", &request_context)
            .await
        {
            params.insert("data_credits".into(), v.to_string());
        }
    }
    // /developer: stats cards + API key list.
    // Wave 23 T5 — was previously not wired, the page rendered its
    // hardcoded `sample_api_keys()` fixture for everyone.
    if path == "/developer" {
        if let Ok(v) = state
            .identity
            .get_with_ctx("/api/v1/developer", &request_context)
            .await
        {
            params.insert("data_developer".into(), v.to_string());
        }
    }
    // `/developer/docs` intentionally does not fetch the historical
    // `/api/v1/developer/docs` canned fixture. Its version-pinned catalog is
    // rendered directly until A5 provides a generated contract that can prove
    // route/auth/rate-limit drift end to end.
    // /analytics: summary stats + top movers.
    // Wave 23 T5 — was previously not wired.
    if path == "/analytics" {
        if let Ok(v) = state
            .analytics
            .get_with_ctx("/api/v1/analytics/summary", &request_context)
            .await
        {
            params.insert("data_analytics".into(), v.to_string());
        }
    }
    // /payment/intent/[id]: payment intent details.
    // Wave 23 T5 — was previously not wired. The dev `payment.rs`
    // reads `type` + `id` from the path params but ignores them
    // (renders a static form), so this is a forward-looking hook.
    if path.starts_with("/payment/intent/") {
        if let Some(id) = path
            .strip_prefix("/payment/intent/")
            .map(|s| s.trim_end_matches('/').to_string())
        {
            if !id.is_empty() {
                if let Ok(v) = state
                    .payment
                    .get_with_ctx(&format!("/api/v1/payment/{}", id), &request_context)
                    .await
                {
                    params.insert("data_payment".into(), v.to_string());
                }
            }
        }
    }
}

/// Shared SSR-safe wallet challenge/sign/verify bridge.
fn wallet_shim() -> &'static str {
    epsx_bff::browser_auth::browser_auth_script()
}

/// `/offline` is SSR-only, so ordinary Dioxus event closures are not hydrated.
/// Bind the retry button in the page shell without a `javascript:` URL. The
/// script is constant and contains no request/query/user data.
fn offline_runtime_script() -> &'static str {
    r#"<script data-epsx-offline-runtime>
(function () {
  var button = document.querySelector('[data-offline-reload="true"]');
  if (!button) return;
  button.addEventListener('click', function () {
    button.disabled = true;
    button.setAttribute('aria-busy', 'true');
    var status = document.getElementById('offline-retry-status');
    if (status) status.textContent = 'Checking your connection…';
    window.location.reload();
  });
})();
</script>"#
}

/// Register the public recovery worker from every frontend document so a
/// user can install the `/offline` shell before the connection is lost. The
/// worker itself caches no current page, request data, API response, session,
/// or owner resource.
fn offline_worker_registration_script() -> &'static str {
    r#"<script data-epsx-offline-worker-registration>
(function () {
  if (!('serviceWorker' in navigator) || !window.isSecureContext) return;
  navigator.serviceWorker.register('/service-worker.js', {
    scope: '/',
    updateViaCache: 'none'
  }).catch(function () {
    // Offline recovery is progressive enhancement. Registration failure must
    // not prevent the network-rendered application from remaining usable.
  });
})();
</script>"#
}

/// Versioned, public-shell-only service worker. Its entire CacheStorage write
/// surface is one credential-free, query-free `/offline` document bearing a
/// server-owned public-cache marker. All other requests, including API, auth,
/// user, owner, admin, notification, analytics, and payment traffic, bypass
/// the worker without a cache read or write.
pub(crate) fn offline_service_worker_script() -> &'static str {
    r#"'use strict';

const CACHE_PREFIX = 'epsx-public-offline-';
const CACHE_NAME = 'epsx-public-offline-v1';
const OFFLINE_PATH = '/offline';
const PUBLIC_CACHE_MARKER = 'offline-shell-v1';

self.addEventListener('install', function (event) {
  event.waitUntil((async function () {
    const response = await fetch(new Request(OFFLINE_PATH, {
      method: 'GET',
      credentials: 'omit',
      cache: 'reload',
      redirect: 'error'
    }));
    const contentType = response.headers.get('content-type') || '';
    const responseUrl = new URL(response.url);
    if (!response.ok || response.type !== 'basic' ||
        responseUrl.origin !== self.location.origin ||
        responseUrl.pathname !== OFFLINE_PATH || responseUrl.search !== '' ||
        !contentType.startsWith('text/html') ||
        response.headers.get('x-epsx-public-cache') !== PUBLIC_CACHE_MARKER) {
      throw new Error('offline shell did not satisfy the public cache contract');
    }
    const cache = await caches.open(CACHE_NAME);
    await cache.put(OFFLINE_PATH, response.clone());
    await self.skipWaiting();
  })());
});

self.addEventListener('activate', function (event) {
  event.waitUntil((async function () {
    const names = await caches.keys();
    await Promise.all(names.map(function (name) {
      if (name.startsWith(CACHE_PREFIX) && name !== CACHE_NAME) {
        return caches.delete(name);
      }
      return Promise.resolve(false);
    }));
    await self.clients.claim();
  })());
});

function isExactOfflineNavigation(request) {
  if (request.method !== 'GET' || request.mode !== 'navigate') return false;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin || url.search !== '' || url.hash !== '') return false;
  return url.pathname === OFFLINE_PATH;
}

self.addEventListener('fetch', function (event) {
  if (!isExactOfflineNavigation(event.request)) return;
  event.respondWith(fetch(event.request).catch(async function () {
    const cache = await caches.open(CACHE_NAME);
    const cached = await cache.match(OFFLINE_PATH, { ignoreSearch: false });
    return cached || Response.error();
  }));
});
"#
}

/// `/manual` retains the pinned source's screenshot viewer without requiring
/// Dioxus hydration. This route-scoped script reads only static DOM attributes,
/// supplies image-error fallbacks, and implements dialog focus restoration,
/// Escape/backdrop close, and a single-control focus trap.
fn manual_runtime_script() -> &'static str {
    r#"<script data-epsx-manual-runtime>
(function () {
  var dialog = document.querySelector('[data-manual-dialog="true"]');
  if (!dialog) return;
  var panel = dialog.querySelector('[data-manual-dialog-panel="true"]');
  var image = dialog.querySelector('[data-manual-dialog-image="true"]');
  var title = dialog.querySelector('[data-manual-dialog-title="true"]');
  var close = dialog.querySelector('[data-manual-dialog-close="true"]');
  var previousFocus = null;

  function hideDialog() {
    dialog.hidden = true;
    document.body.style.overflow = '';
    if (previousFocus && typeof previousFocus.focus === 'function') previousFocus.focus();
    previousFocus = null;
  }

  document.querySelectorAll('[data-manual-screenshot="true"]').forEach(function (button) {
    var thumb = button.querySelector('img');
    function showFallback() {
      button.setAttribute('data-image-error', 'true');
      button.disabled = true;
      button.removeAttribute('aria-haspopup');
      button.setAttribute('aria-label', button.getAttribute('data-screenshot-alt') + ' screenshot unavailable');
    }
    if (thumb) {
      thumb.addEventListener('error', showFallback);
      if (thumb.complete && thumb.naturalWidth === 0) showFallback();
    }
    button.addEventListener('click', function () {
      previousFocus = button;
      image.src = button.getAttribute('data-screenshot-src') || '';
      image.alt = button.getAttribute('data-screenshot-alt') || '';
      title.textContent = button.getAttribute('data-screenshot-alt') || 'Feature screenshot';
      dialog.hidden = false;
      document.body.style.overflow = 'hidden';
      close.focus();
    });
  });

  close.addEventListener('click', hideDialog);
  dialog.addEventListener('click', function (event) {
    if (!panel.contains(event.target)) hideDialog();
  });
  dialog.addEventListener('keydown', function (event) {
    if (event.key === 'Escape') {
      event.preventDefault();
      hideDialog();
    } else if (event.key === 'Tab') {
      event.preventDefault();
      close.focus();
    }
  });

  document.querySelectorAll('[data-route-template="true"]').forEach(function (link) {
    link.addEventListener('click', function (event) { event.preventDefault(); });
  });
})();
</script>"#
}

/// `/developer/docs` is rendered by hydration-less Dioxus SSR. This constant,
/// route-scoped controller restores the pinned source's mobile navigator,
/// accessible endpoint accordions, language tabs, and copy controls. It reads
/// only the static documentation DOM and never sends an API request or handles
/// a credential; the source's live executor remains fail-closed until A4/A5.
fn developer_docs_runtime_script() -> &'static str {
    r#"<script data-epsx-developer-docs-runtime>
(function () {
  var root = document.querySelector('.developer-docs-page');
  if (!root) return;
  var sidebar = root.querySelector('[data-docs-sidebar="true"]');
  var sidebarToggle = root.querySelector('[data-docs-sidebar-toggle="true"]');
  var overlay = root.querySelector('[data-docs-sidebar-overlay="true"]');
  var sidebarWasOpened = false;

  function setSidebar(open) {
    if (!sidebar || !sidebarToggle || !overlay) return;
    sidebar.classList.toggle('open', open);
    sidebarToggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    sidebarToggle.setAttribute('aria-label', open ? 'Close API reference navigation' : 'Open API reference navigation');
    sidebarToggle.querySelector('span').textContent = open ? '×' : '☰';
    overlay.hidden = !open;
    if (open) {
      sidebarWasOpened = true;
      var firstLink = sidebar.querySelector('[data-docs-section-link]');
      if (firstLink) firstLink.focus();
    } else if (sidebarWasOpened) {
      window.requestAnimationFrame(function () { sidebarToggle.focus(); });
    }
  }

  if (sidebarToggle) sidebarToggle.addEventListener('click', function () {
    setSidebar(sidebarToggle.getAttribute('aria-expanded') !== 'true');
  });
  if (overlay) overlay.addEventListener('click', function () { setSidebar(false); });
  root.querySelectorAll('[data-docs-section-link]').forEach(function (link) {
    link.addEventListener('click', function (event) {
      event.preventDefault();
      root.querySelectorAll('[data-docs-section-link]').forEach(function (item) { item.classList.remove('active'); });
      link.classList.add('active');
      var section = document.getElementById('section-' + link.getAttribute('data-docs-section-link'));
      if (section) section.scrollIntoView({ behavior: 'smooth', block: 'start' });
      if (window.matchMedia('(max-width: 1023px)').matches) setSidebar(false);
    });
  });

  root.querySelectorAll('[data-docs-endpoint-toggle="true"]').forEach(function (button) {
    var body = document.getElementById(button.getAttribute('aria-controls'));
    if (!body) return;
    button.addEventListener('click', function () {
      var open = button.getAttribute('aria-expanded') !== 'true';
      button.setAttribute('aria-expanded', open ? 'true' : 'false');
      body.hidden = !open;
      var chevron = button.querySelector('.docs-endpoint-card-chevron');
      if (chevron) chevron.textContent = open ? '▾' : '▸';
    });
  });

  root.querySelectorAll('.docs-code-example').forEach(function (example) {
    var tabs = Array.prototype.slice.call(example.querySelectorAll('[data-docs-code-tab]'));
    var panels = Array.prototype.slice.call(example.querySelectorAll('[data-docs-code-panel]'));
    function select(index, focus) {
      tabs.forEach(function (tab, tabIndex) {
        var selected = tabIndex === index;
        tab.classList.toggle('active', selected);
        tab.setAttribute('aria-selected', selected ? 'true' : 'false');
        tab.tabIndex = selected ? 0 : -1;
      });
      panels.forEach(function (panel) {
        panel.hidden = panel.getAttribute('data-docs-code-panel') !== tabs[index].getAttribute('data-docs-code-tab');
      });
      if (focus) tabs[index].focus();
    }
    tabs.forEach(function (tab, index) {
      tab.addEventListener('click', function () { select(index, false); });
      tab.addEventListener('keydown', function (event) {
        var next = index;
        if (event.key === 'ArrowRight') next = (index + 1) % tabs.length;
        else if (event.key === 'ArrowLeft') next = (index - 1 + tabs.length) % tabs.length;
        else if (event.key === 'Home') next = 0;
        else if (event.key === 'End') next = tabs.length - 1;
        else return;
        event.preventDefault();
        select(next, true);
      });
    });
    var copy = example.querySelector('[data-docs-copy-code="true"]');
    if (copy) copy.addEventListener('click', function () {
      var current = example.querySelector('[data-docs-code-panel]:not([hidden]) code');
      if (current && window.epsx) window.epsx.copyText(current.textContent || '', copy);
    });
  });

  root.querySelectorAll('[data-docs-copy-response="true"]').forEach(function (button) {
    button.addEventListener('click', function () {
      var current = button.parentElement.querySelector('.docs-response-panel code');
      if (current && window.epsx) window.epsx.copyText(current.textContent || '', button);
    });
  });

  document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape' && sidebarToggle && sidebarToggle.getAttribute('aria-expanded') === 'true') {
      event.preventDefault();
      setSidebar(false);
    }
  });
})();
</script>"#
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

fn safe_return_url(query: &str) -> String {
    let Ok(request_url) = reqwest::Url::parse(&format!("https://frontend.invalid/?{query}")) else {
        return "/".to_string();
    };
    let Some(raw) = request_url
        .query_pairs()
        .find_map(|(key, value)| (key == "return_url").then(|| value.into_owned()))
    else {
        return "/".to_string();
    };
    if !raw.starts_with('/') || raw.starts_with("//") || raw.contains('\\') {
        return "/".to_string();
    }
    let Ok(target) = request_url.join(&raw) else {
        return "/".to_string();
    };
    if target.origin() != request_url.origin() || target.path() == "/auth" {
        return "/".to_string();
    }

    let mut value = target.path().to_string();
    if let Some(query) = target.query() {
        value.push('?');
        value.push_str(query);
    }
    if let Some(fragment) = target.fragment() {
        value.push('#');
        value.push_str(fragment);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::developer_docs_runtime_script;
    use super::manual_runtime_script;
    use super::offline_runtime_script;
    use super::offline_service_worker_script;
    use super::offline_worker_registration_script;
    use super::pricing_redirect_response;
    use super::safe_return_url;
    use super::urlencode;

    #[test]
    fn return_url_must_remain_same_origin() {
        assert_eq!(
            safe_return_url("return_url=%2Fprofile%3Ftab%3Dauth"),
            "/profile?tab=auth"
        );
        assert_eq!(
            safe_return_url("return_url=https%3A%2F%2Fevil.example"),
            "/"
        );
        assert_eq!(safe_return_url("return_url=%2F%2Fevil.example%2Fx"), "/");
        assert_eq!(safe_return_url("return_url=%5C%5Cevil.example"), "/");
        assert_eq!(safe_return_url("return_url=%2Fauth"), "/");
    }
    use axum::http::StatusCode;

    #[test]
    fn urlencode_passes_alnum() {
        // Matches Vercel's prod middleware `epsx.return_url=%2F<path>` shape.
        assert_eq!(urlencode("/notifications"), "%2Fnotifications");
        // Wave 23 T3 — query parameter is now `?return_url=` (NOT
        // `?next=`). The encoder must still produce the same per-byte
        // shape regardless of the query parameter name.
        assert_eq!(
            urlencode("/auth?return_url=/x"),
            "%2Fauth%3Freturn_url%3D%2Fx"
        );
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
        assert_eq!(
            r.headers().get("location").unwrap(),
            "/plans?ref=foo&affiliate=bar"
        );
    }

    // === Wave 35b T1 — AuthGate 307-redirect for marketing routes ===

    /// `/offline` must remain reachable without an authenticated session.
    /// An offline fallback that redirects through wallet auth cannot recover
    /// a disconnected browser. `/about` and `/contact` retain their pinned
    /// middleware behavior.
    #[test]
    fn unauth_redirect_paths_keeps_marketing_routes_but_exposes_offline() {
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
            !paths.contains(&"/offline"),
            "offline fallback must be public"
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

    #[test]
    fn offline_runtime_binds_button_without_user_data_or_javascript_url() {
        let script = offline_runtime_script();
        assert!(script.contains("data-epsx-offline-runtime"));
        assert!(script.contains("[data-offline-reload=\"true\"]"));
        assert!(script.contains("window.location.reload()"));
        assert!(script.contains("aria-busy"));
        assert!(!script.contains("javascript:"));
        assert!(!script.contains("reason"));
        assert!(!script.contains("return_url"));
    }

    #[test]
    fn offline_worker_registration_is_constant_and_body_free() {
        let script = offline_worker_registration_script();
        assert!(script.contains("data-epsx-offline-worker-registration"));
        assert!(script.contains("register('/service-worker.js'"));
        assert!(script.contains("updateViaCache: 'none'"));
        assert!(!script.contains("fetch("));
        assert!(!script.contains("cookie"));
        assert!(!script.contains("Authorization"));
        assert!(!script.contains("location.search"));
    }

    #[test]
    fn offline_worker_cache_write_is_exact_public_shell_only() {
        let script = offline_service_worker_script();
        assert!(script.contains("credentials: 'omit'"));
        assert!(script.contains("url.pathname === OFFLINE_PATH"));
        assert!(script.contains("url.search !== ''"));
        assert!(script.contains("request.mode !== 'navigate'"));
        assert!(script.contains("response.type !== 'basic'"));
        assert!(script.contains("responseUrl.origin !== self.location.origin"));
        assert!(script.contains("responseUrl.pathname !== OFFLINE_PATH"));
        assert!(script.contains("response.headers.get('x-epsx-public-cache')"));
        assert!(script.contains("cache.put(OFFLINE_PATH"));
        assert!(!script.contains("cache.add"));
        assert!(!script.contains("cache.addAll"));
        assert!(!script.contains("cache.put(event.request"));
        assert!(!script.contains("ignoreSearch: true"));
    }

    #[test]
    fn manual_runtime_binds_accessible_dialog_without_request_data() {
        let script = manual_runtime_script();
        assert!(script.contains("data-epsx-manual-runtime"));
        assert!(script.contains("[data-manual-screenshot=\"true\"]"));
        assert!(script.contains("event.key === 'Escape'"));
        assert!(script.contains("previousFocus.focus()"));
        assert!(script.contains("data-image-error"));
        assert!(script.contains("thumb.complete && thumb.naturalWidth === 0"));
        assert!(script.contains("event.preventDefault()"));
        assert!(!script.contains("javascript:"));
        assert!(!script.contains("window.location"));
        assert!(!script.contains("fetch("));
    }

    #[test]
    fn developer_docs_runtime_is_static_accessible_and_fail_closed() {
        let script = developer_docs_runtime_script();
        assert!(script.contains("data-epsx-developer-docs-runtime"));
        assert!(script.contains("data-docs-endpoint-toggle"));
        assert!(script.contains("aria-expanded"));
        assert!(script.contains("ArrowRight"));
        assert!(script.contains("window.epsx.copyText"));
        assert!(!script.contains("fetch("));
        assert!(!script.contains("Authorization"));
    }
}
