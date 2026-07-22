//! Dioxus SSR rendering for the admin BFF.
//!
//! The HTTP request is parsed into a `PageContext` (path, query, user) and
//! dispatched to the appropriate `rsx!` page from `epsx_dioxus_ui::pages`.
//! The HTML is wrapped in the EPSX design-system page shell so the visuals
//! match the Next.js admin 1:1.
//!
//! Wave 3a Track C — the rendered page body is wrapped in
//! `AdminLayout::Auth` (from `epsx_dioxus_ui::layout::shell`) so the
//! admin chrome (`Header` + `Sidebar` + `AdminFooter`) is owned by the
//! layout, not by each page. Pages are body-only after this wave.

use axum::{
    extract::{Request, State},
    response::{IntoResponse, Response},
};
use epsx_client::RequestContext;
use epsx_dioxus_ui::layout::shell::{AdminLayout, ServerUser};
use epsx_dioxus_ui::pages::admin_pages::news::{
    ADMIN_NEWS_DATA_PARAM, ADMIN_NEWS_EMPTY, ADMIN_NEWS_FORBIDDEN, ADMIN_NEWS_MALFORMED,
    ADMIN_NEWS_PAGE_PARAM, ADMIN_NEWS_READY, ADMIN_NEWS_STATE_PARAM, ADMIN_NEWS_STATUS_PARAM,
    ADMIN_NEWS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::admin_pages::payments::{
    decode_admin_payment_intent_list, ADMIN_PAYMENTS_DATA_PARAM, ADMIN_PAYMENTS_EMPTY,
    ADMIN_PAYMENTS_LIMIT_PARAM, ADMIN_PAYMENTS_MALFORMED, ADMIN_PAYMENTS_OFFSET_PARAM,
    ADMIN_PAYMENTS_PAYER_PARAM, ADMIN_PAYMENTS_READY, ADMIN_PAYMENTS_STATE_PARAM,
    ADMIN_PAYMENTS_STATUS_PARAM, ADMIN_PAYMENTS_TAB_PARAM, ADMIN_PAYMENTS_UNAVAILABLE,
};
use epsx_dioxus_ui::pages::{admin_pages, render_page, PageContext, PageStatus};
use std::collections::HashMap;

use super::auth;
use super::news_adapter::{load_admin_news, AdminNewsLoad, AdminNewsQuery};
use super::AppState;

fn record_admin_news_load(
    params: &mut HashMap<String, String>,
    query: &AdminNewsQuery,
    load: AdminNewsLoad,
) {
    params.remove(ADMIN_NEWS_DATA_PARAM);
    params.insert(ADMIN_NEWS_PAGE_PARAM.to_string(), query.page.to_string());
    params.insert(
        ADMIN_NEWS_STATUS_PARAM.to_string(),
        query.status.to_string(),
    );

    let state = match load {
        AdminNewsLoad::Ready(payload) => {
            params.insert(
                ADMIN_NEWS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed admin-news projection is serializable"),
            );
            ADMIN_NEWS_READY
        }
        AdminNewsLoad::Empty(payload) => {
            params.insert(
                ADMIN_NEWS_DATA_PARAM.to_string(),
                serde_json::to_string(&payload)
                    .expect("the typed empty admin-news projection is serializable"),
            );
            ADMIN_NEWS_EMPTY
        }
        AdminNewsLoad::Forbidden => ADMIN_NEWS_FORBIDDEN,
        AdminNewsLoad::Unavailable => ADMIN_NEWS_UNAVAILABLE,
        AdminNewsLoad::Malformed => ADMIN_NEWS_MALFORMED,
    };
    params.insert(ADMIN_NEWS_STATE_PARAM.to_string(), state.to_string());
}

fn private_admin_html_response(status: axum::http::StatusCode, doc: String) -> Response {
    (
        status,
        [
            ("content-type", "text/html; charset=utf-8"),
            ("cache-control", "private, no-store"),
            ("vary", "Cookie, Authorization"),
        ],
        doc,
    )
        .into_response()
}

fn record_payment_intent_load(
    params: &mut HashMap<String, String>,
    result: Result<serde_json::Value, ()>,
) {
    params.remove(ADMIN_PAYMENTS_DATA_PARAM);
    match result {
        Ok(value) => match decode_admin_payment_intent_list(value) {
            Some(payload) => {
                let state = if payload.items.is_empty() && payload.total == 0 {
                    ADMIN_PAYMENTS_EMPTY
                } else {
                    ADMIN_PAYMENTS_READY
                };
                params.insert(
                    ADMIN_PAYMENTS_DATA_PARAM.to_string(),
                    serde_json::to_string(&payload)
                        .expect("the typed payment-intent response is serializable"),
                );
                params.insert(ADMIN_PAYMENTS_STATE_PARAM.to_string(), state.to_string());
            }
            None => {
                params.insert(
                    ADMIN_PAYMENTS_STATE_PARAM.to_string(),
                    ADMIN_PAYMENTS_MALFORMED.to_string(),
                );
            }
        },
        Err(()) => {
            params.insert(
                ADMIN_PAYMENTS_STATE_PARAM.to_string(),
                ADMIN_PAYMENTS_UNAVAILABLE.to_string(),
            );
        }
    }
}

pub async fn ssr_handler(State(state): State<AppState>, request: Request) -> Response {
    let (parts, _body) = request.into_parts();
    let path = parts.uri.path().to_string();
    let query = parts.uri.query().unwrap_or("").to_string();
    let headers = parts.headers.clone();

    // Resolve only a cryptographically verified canonical cookie/bearer user.
    // Permissions are backend-issued and remain verbatim; the admin UI does no
    // role-to-permission expansion.
    let verified_session =
        auth::verified_access_token(&headers, state.verifier.as_ref(), state.cookie_environment)
            .await;
    let (verified_access_token, user) = match verified_session {
        Some((token, session)) => (Some(token), Some(auth::ui_user(session, None))),
        None => (None, None),
    };

    // Admin: load only the bounded, read-only payment-intent dependency. Every
    // outcome is explicit; an upstream error or malformed payload is never
    // represented as an authoritative empty list.
    let mut params = HashMap::new();
    let route_path = if path.starts_with("/admin") {
        let stripped = path.trim_start_matches("/admin");
        if stripped.is_empty() {
            "/"
        } else {
            stripped
        }
    } else {
        path.as_str()
    };
    if route_path == "/payments" {
        match (
            super::payment_tab(&query),
            super::PaymentIntentQuery::from_raw(&query),
        ) {
            (Ok(tab), Ok(payment_query)) => {
                params.insert(ADMIN_PAYMENTS_TAB_PARAM.to_string(), tab.to_string());
                params.insert(
                    ADMIN_PAYMENTS_LIMIT_PARAM.to_string(),
                    payment_query.limit.to_string(),
                );
                params.insert(
                    ADMIN_PAYMENTS_OFFSET_PARAM.to_string(),
                    payment_query.offset.to_string(),
                );
                if let Some(payer) = &payment_query.payer {
                    params.insert(ADMIN_PAYMENTS_PAYER_PARAM.to_string(), payer.clone());
                }
                if let Some(status) = &payment_query.status {
                    params.insert(ADMIN_PAYMENTS_STATUS_PARAM.to_string(), status.clone());
                }

                if tab == "payments" {
                    match verified_access_token.as_ref() {
                        Some(token) => {
                            let mut request_context = RequestContext::from_headers(&headers);
                            request_context.auth_token = Some(token.clone());
                            let result = state
                                .payment
                                .get_with_ctx(&payment_query.upstream_path(), &request_context)
                                .await
                                .map_err(|error| {
                                    tracing::warn!(
                                        "admin payment-intent SSR load unavailable: {error}"
                                    );
                                });
                            record_payment_intent_load(&mut params, result);
                        }
                        None => record_payment_intent_load(&mut params, Err(())),
                    }
                }
            }
            _ => {
                params.insert(ADMIN_PAYMENTS_TAB_PARAM.to_string(), "payments".to_string());
                params.insert(
                    ADMIN_PAYMENTS_STATE_PARAM.to_string(),
                    ADMIN_PAYMENTS_MALFORMED.to_string(),
                );
            }
        }
    }
    // The pinned admin news list still has one exact, backend-owned Rust read
    // contract. Use that compatibility endpoint only for `/news`; the public
    // file-backed content-service feed is not an admin record authority. The
    // adapter projects away article bodies and rejects every contract drift.
    if route_path == "/news" {
        match AdminNewsQuery::from_raw(&query) {
            Ok(news_query) => match verified_access_token.as_ref() {
                Some(token) => {
                    let mut request_context = RequestContext::from_headers(&headers);
                    request_context.auth_token = Some(token.clone());
                    let load = load_admin_news(&state.content, &news_query, &request_context).await;
                    record_admin_news_load(&mut params, &news_query, load);
                }
                None => {
                    record_admin_news_load(&mut params, &news_query, AdminNewsLoad::Unavailable)
                }
            },
            Err(()) => {
                let default_query =
                    AdminNewsQuery::from_raw("").expect("the empty admin-news query is valid");
                record_admin_news_load(&mut params, &default_query, AdminNewsLoad::Malformed);
            }
        }
    }
    // Wave 3a Track B — admin doesn't render the wallet dropdown yet,
    // so the BFF just plumbs the default `ConnectedWalletState`. The
    // type is here so Track A's MainLayout can read `ctx.wallet`
    // uniformly; admin pages ignore it for now.
    let ctx = PageContext {
        user: user.clone(),
        path: path.clone(),
        query: query.clone(),
        params,
        api_url: state.api_url.clone(),
        demo_login_enabled: state.demo_login_enabled,
        wallet: epsx_dioxus_ui::auth::wallet_button::ConnectedWalletState::default(),
    };

    // Use the dedicated admin dispatcher regardless of `is_admin` so the
    // admin's own auth middleware (if installed) can decide. The frontend
    // BFF will have the same UX.
    //
    // Wave 38b T2 — also derive the `layout_path` (the path with
    // the `/admin` prefix stripped) and pass it as the
    // `AdminLayout::Auth`'s `current_path`. The `default_no_layout_
    // paths()` registry uses the un-prefixed path (e.g.
    // `/access-denied`) so the layout's `is_no_layout` check
    // (`current_path == *p || current_path.starts_with(p)`) only
    // matches when we pass the stripped path. Previously the BFF
    // passed the raw `/admin/access-denied` path which made the
    // check fail — the AuthGate overlay then masked the
    // red-shield Access Denied panel and ballooned the
    // pixel-diff to ~99%.
    let (meta, body_element, layout_path) = if path.starts_with("/admin") {
        let p = path.trim_start_matches("/admin").to_string();
        let stripped = if p.is_empty() { "/".to_string() } else { p };
        let mut c = ctx.clone();
        c.path = stripped.clone();
        let (m, b) = admin_pages::dispatch(&c);
        (m, b, stripped)
    } else {
        let (m, b) = render_page(&ctx, true);
        (m, b, path.clone())
    };

    // Wave 3a Track C — wrap the page body in `AdminLayout::Auth` so the
    // admin shell chrome is rendered by the layout, not by each page.
    //
    // The admin BFF does not yet plumb a server user into the layout —
    // the cookie-based session check happens higher in the request
    // lifecycle. Until Track B's `wallet` field lands on `PageContext`
    // we pass a default `ConnectedWalletState` (no wallet dropdown for
    // admin yet) and let the layout's `is_authenticated` default to
    // `false` — pages still get the chrome and the AuthGate will
    // overlay when needed.
    //
    // Wave 38b T2 — `no_layout_paths` extension. The 3 outlier
    // routes (`/access-denied`, `/unauthorized`,
    // `/developer-portal/api-keys/create`) render the SAME SSR
    // "Access Denied" panel in prod (verified by owner probe
    // 2026-06-18) — there is NO admin sidebar / header / footer
    // on those pages. The 2 first routes are already in the
    // shared `default_no_layout_paths()`; we add the 3rd here so
    // the dev BFF strips the chrome and the AuthGate overlay
    // (which would otherwise mask the centered Access Denied
    // panel and balloon the pixel-diff to ~99% per Wave 24 T1'
    // report).
    let server_user: Option<ServerUser> = user.as_ref().map(|u| ServerUser {
        id: u.id.clone(),
        email: u.email.clone().unwrap_or_default(),
        name: None,
        role: u.roles.first().cloned().unwrap_or_default(),
    });
    let is_authenticated = user.is_some();
    let shell_layout_path = safe_admin_layout_path(&layout_path, is_authenticated);
    let no_layout_paths_override = Some(vec![
        "/auth".to_string(),
        "/login".to_string(),
        "/unauthorized".to_string(),
        "/access-denied".to_string(),
        "/permissions/policies".to_string(),
        "/developer-portal/api-keys/create".to_string(),
    ]);
    // === Wave 49+ — Wave 6B pages provide their own chrome ===
    //
    // The 5 Wave 6B admin pages (`/admin/dashboard`,
    // `/admin/analytics`, `/admin/media`, `/admin/policies`,
    // `/admin/settings`) wrap themselves in `<AdminShell>` (from
    // `shared/rust/dioxus_ui::layout::admin_shell`), which renders
    // the full sidebar + breadcrumb header + main + footer chrome.
    // The BFF's `AdminLayout::Auth` ALSO renders that chrome (via
    // `shell::MainLayout`). Wrapping a Wave 6B page in
    // `AdminLayout::Auth` therefore produced a structural
    // double-sidebar / double-header / double-footer bug on every
    // tablet+ viewport.
    //
    // The fix: for routes whose page owns `<AdminShell>`, skip the
    // BFF-level `AdminLayout::Auth` wrap entirely. The page's own
    // authentication gate still covers the signed-out case.
    let wave6b_paths: &[&str] = &[
        "/",      // admin home → dashboard::render → AdminShell
        "/index", // target-only dashboard alias → dashboard::render → AdminShell
        "/analytics",
        "/media",
        "/policies",
        "/settings",
    ];
    let is_wave6b = wave6b_paths.contains(&layout_path.as_str());
    let body_element = if meta.status == PageStatus::NotFound || is_wave6b {
        // Page provides its own chrome via `<AdminShell>`; don't
        // double-wrap. Its own authentication gate still handles
        // the signed-out overlay.
        body_element
    } else {
        AdminLayout::Auth {
            current_path: shell_layout_path,
            server_user,
            is_authenticated,
            is_gated: None,
            no_layout_paths: no_layout_paths_override,
        }
        .render(body_element, None, None, None)
    };

    let body_html = dioxus_ssr::render_element(body_element);
    let status = match meta.status {
        PageStatus::Ok => axum::http::StatusCode::OK,
        PageStatus::NotFound => axum::http::StatusCode::NOT_FOUND,
    };

    let doc = epsx_templates::page_shell_with_body_class_and_keywords(
        &meta.title,
        &meta.description,
        meta.keywords.as_deref(),
        &String::new(),
        &body_html,
        meta.include_footer,
        // Wave 38c T1 — body_class is now Option<String>. None
        // means "no body class override beyond the page shell's
        // default `min-h-screen`". The 3 admin outliers
        // (`/access-denied`, `/unauthorized`,
        // `/developer-portal/api-keys/create`) set their own body
        // class via `PageMeta::admin_with_body_class(...)` to
        // mirror prod's `h-screen overflow-hidden font-sans`
        // wrapper.
        meta.body_class.as_deref().unwrap_or(""),
    );

    let denial_runtime = matches!(layout_path.as_str(), "/access-denied" | "/unauthorized")
        .then_some(admin_denial_runtime_script())
        .unwrap_or("");
    let doc = doc.replace(
        "</body>",
        &format!(
            "<script>{}</script>{denial_runtime}</body>",
            epsx_bff::browser_auth::browser_auth_script(),
        ),
    );

    private_admin_html_response(status, doc)
}

/// Keep security-sensitive dynamic chat, news, wallet, and plan references out
/// of signed-out shell breadcrumbs and return URLs. The dispatcher still
/// receives the real path so authenticated routing can resolve it normally.
fn safe_admin_layout_path(layout_path: &str, is_authenticated: bool) -> String {
    if !is_authenticated {
        if let Some(reference) = layout_path.strip_prefix("/chat/") {
            if !reference.is_empty() && !reference.contains('/') {
                return "/chat".to_string();
            }
        }

        if let Some(rest) = layout_path.strip_prefix("/news/") {
            if let Some((reference, suffix)) = rest.split_once('/') {
                if !reference.is_empty() && suffix == "edit" {
                    return "/news".to_string();
                }
            }
        }

        if let Some(reference) = layout_path.strip_prefix("/wallet-management/access/plans/") {
            if !reference.is_empty() && !reference.contains('/') {
                return "/wallet-management/access/plans".to_string();
            }
        }

        if let Some(rest) = layout_path.strip_prefix("/wallet-management/wallets/") {
            if let Some((reference, suffix)) = rest.split_once('/') {
                if !reference.is_empty() && suffix == "disable" {
                    return "/wallet-management/wallets".to_string();
                }
            }
        }

        if let Some(reference) = layout_path.strip_prefix("/wallet-management/") {
            if !reference.is_empty()
                && !reference.contains('/')
                && !matches!(reference, "wallets" | "credits" | "access")
            {
                return "/wallet-management/wallets".to_string();
            }
        }
    }

    layout_path.to_string()
}

/// The denial pages are rendered as hydration-free SSR. This constant,
/// route-scoped controller restores the source actions without embedding any
/// query or user value in JavaScript. Reauthentication uses the canonical
/// same-origin logout endpoint and always follows the already-sanitized auth
/// link. The back action uses history only for a same-origin referrer; the
/// anchor remains a safe static fallback when the page was opened directly.
fn admin_denial_runtime_script() -> &'static str {
    r#"<script data-epsx-admin-denial-runtime>
(function () {
  var root = document.querySelector('[data-admin-denial-runtime="true"]');
  if (!root) return;
  var auth = root.querySelector('[data-admin-denial-auth="true"]');
  var back = root.querySelector('[data-admin-denial-back="true"]');

  if (auth) auth.addEventListener('click', async function (event) {
    event.preventDefault();
    var target = auth.getAttribute('href') || '/auth?return_url=%2F';
    auth.setAttribute('aria-busy', 'true');
    try {
      await fetch('/api/v1/auth/logout', { method: 'POST', credentials: 'same-origin' });
    } catch (_) {
      // The BFF clears its local cookies even when upstream revocation fails.
    } finally {
      window.location.assign(target);
    }
  });

  if (back) back.addEventListener('click', function (event) {
    if (!document.referrer || window.history.length <= 1) return;
    try {
      var previous = new URL(document.referrer);
      if (previous.origin !== window.location.origin) return;
      event.preventDefault();
      window.history.back();
    } catch (_) {}
  });
})();
</script>"#
}

#[cfg(test)]
mod tests {
    //! Smoke tests for Wave 3a Track C — verify that the admin BFF
    //! wraps page bodies in `AdminLayout::Auth` (which renders the
    //! `Header` component with the `admin-header` class).
    //!
    //! The full BFF render path is async/axum-bound; we exercise the
    //! thin render-only path (construct a `PageContext`, dispatch the
    //! page, wrap in `AdminLayout::Auth`, serialize) to confirm the
    //! chrome is present.

    use super::*;
    use epsx_dioxus_ui::{
        auth::{user::AuthMethod, User},
        pages::admin_pages::news::{AdminNewsArticleSummary, AdminNewsList},
        pages::PageContext,
    };

    fn build_ctx(path: &str) -> PageContext {
        PageContext {
            user: None,
            path: path.to_string(),
            query: String::new(),
            params: HashMap::new(),
            api_url: String::new(),
            demo_login_enabled: true,
            // Wave 3a Track B — `PageContext` carries a
            // `ConnectedWalletState` so layouts can read `ctx.wallet`
            // uniformly. Admin pages ignore the wallet field, so the
            // test helper just plugs in a default.
            wallet: epsx_dioxus_ui::auth::wallet_button::ConnectedWalletState::default(),
        }
    }

    fn payment_payload(items: Vec<serde_json::Value>, total: i64) -> serde_json::Value {
        serde_json::json!({ "items": items, "total": total })
    }

    fn payment_item(id: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "chain_id": "56",
            "payer": "0x1111111111111111111111111111111111111111",
            "payee": "0x2222222222222222222222222222222222222222",
            "amount": "1000000000000000000",
            "token_address": "0x3333333333333333333333333333333333333333",
            "status": "pending",
            "escrow_id": null,
            "tx_hash": null,
            "description": null,
            "expires_at": null,
            "created_at": "2026-07-22T10:00:00Z",
            "updated_at": "2026-07-22T10:00:00Z"
        })
    }

    fn news_query() -> AdminNewsQuery {
        AdminNewsQuery::from_raw("page=2&status=published").unwrap()
    }

    fn news_payload(articles: Vec<AdminNewsArticleSummary>, total: i64) -> AdminNewsList {
        AdminNewsList {
            articles,
            total,
            page: 2,
            limit: 20,
        }
    }

    fn news_item() -> AdminNewsArticleSummary {
        AdminNewsArticleSummary {
            id: "2f68f1aa-08d7-4b40-a25f-b35e7fd0ed31".to_string(),
            title: "Migration status".to_string(),
            slug: "migration-status".to_string(),
            summary: Some("A backend-authoritative update".to_string()),
            status: "published".to_string(),
            tags: vec!["migration".to_string()],
            published_at: Some("2026-07-22T03:04:05Z".to_string()),
            created_at: "2026-07-21T03:04:05Z".to_string(),
            is_pinned: true,
        }
    }

    /// Render a page through the admin BFF render path (without
    /// `page_shell_with_body_class`) so we can assert on the
    /// layout-wrapped HTML in isolation.
    fn render_admin_html(path: &str) -> String {
        render_admin_html_with_user(path, None)
    }

    fn render_admin_html_with_user(path: &str, user: Option<User>) -> String {
        let mut ctx = build_ctx(path);
        ctx.user = user.clone();
        let admin_path = path.trim_start_matches("/admin").to_string();
        let mut c = ctx.clone();
        c.path = if admin_path.is_empty() {
            "/".to_string()
        } else {
            admin_path
        };
        let (_meta, body) = admin_pages::dispatch(&c);
        let server_user = user.as_ref().map(|user| ServerUser {
            id: user.id.clone(),
            email: user.email.clone().unwrap_or_default(),
            name: user.display_name.clone(),
            role: user.roles.first().cloned().unwrap_or_default(),
        });
        let is_authenticated = user.is_some();
        let shell_layout_path = safe_admin_layout_path(&c.path, is_authenticated);
        // Wave 38b T2 — mirror the production `no_layout_paths`
        // override from `ssr_handler` so the test exercises the
        // same render path as the live BFF (the 3 outliers skip
        // the chrome + AuthGate).
        let no_layout_paths_override = Some(vec![
            "/login".to_string(),
            "/unauthorized".to_string(),
            "/access-denied".to_string(),
            "/permissions/policies".to_string(),
            "/developer-portal/api-keys/create".to_string(),
        ]);
        let body = AdminLayout::Auth {
            current_path: shell_layout_path,
            server_user,
            is_authenticated,
            is_gated: None,
            no_layout_paths: no_layout_paths_override,
        }
        .render(body, None, None, None);
        dioxus_ssr::render_element(body)
    }

    #[test]
    fn admin_dashboard_renders_with_admin_header() {
        let html = render_admin_html("/admin");
        // The admin `Header` component renders an element with the
        // `admin-header` class — that's our marker for "the layout
        // chrome is present".
        assert!(
            html.contains("admin-header"),
            "expected rendered admin dashboard HTML to include `admin-header` from the `Header` component rendered by `AdminLayout::Auth`; got: {}",
            html
        );
    }

    #[test]
    fn audit_log_authenticated_render_has_exactly_one_admin_sidebar() {
        let user = User {
            id: "admin-session".to_string(),
            address: "0x1234".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            email: None,
            tier: None,
            permissions: vec![],
            last_login_at: None,
            auth_method: AuthMethod::Siwe,
            display_name: None,
        };
        let html = render_admin_html_with_user("/admin/audit-log", Some(user));

        assert!(html.contains("data-audit-log-state=\"unavailable\""));
        assert_eq!(
            html.matches("class=\"admin-sidebar ").count(),
            1,
            "audit-log must rely on the BFF admin layout instead of nesting a second shell: {html}"
        );
    }

    #[test]
    fn signed_out_dynamic_full_layout_hides_private_route_references() {
        for (path, safe_return_url) in [
            ("/admin/chat/private-case-reference", "/chat"),
            ("/admin/news/private-case-reference/edit", "/news"),
            (
                "/admin/wallet-management/private-case-reference",
                "/wallet-management/wallets",
            ),
            (
                "/admin/wallet-management/wallets/private-case-reference/disable",
                "/wallet-management/wallets",
            ),
            (
                "/admin/wallet-management/access/plans/private-case-reference",
                "/wallet-management/access/plans",
            ),
        ] {
            let html = render_admin_html(path);

            assert!(!html.contains("private-case-reference"), "{path}: {html}");
            assert!(
                html.contains(&format!("data-return-url=\"{safe_return_url}\"")),
                "{path}: {html}"
            );
        }
    }

    #[test]
    fn authenticated_layout_keeps_dynamic_route_references() {
        for path in [
            "/chat/conversation-1",
            "/news/article-1/edit",
            "/wallet-management/0xabc",
            "/wallet-management/wallets/0xabc/disable",
            "/wallet-management/access/plans/pro",
        ] {
            assert_eq!(safe_admin_layout_path(path, true), path);
        }
    }

    #[test]
    fn denial_runtime_uses_only_same_origin_endpoints_and_static_dom_values() {
        let script = admin_denial_runtime_script();
        assert!(script.contains("data-epsx-admin-denial-runtime"));
        assert!(script.contains("fetch('/api/v1/auth/logout'"));
        assert!(script.contains("credentials: 'same-origin'"));
        assert!(script.contains("previous.origin !== window.location.origin"));
        assert!(script.contains("window.location.assign(target)"));
        assert!(!script.contains("innerHTML"));
        assert!(!script.contains("localStorage"));
        assert!(!script.contains("access_token"));
        assert!(!script.contains("refresh_token"));
        assert!(!script.contains("javascript:"));
    }

    #[test]
    fn payment_load_records_ready_with_only_typed_payload() {
        let mut params = HashMap::new();
        record_payment_intent_load(
            &mut params,
            Ok(payment_payload(vec![payment_item("intent-1")], 1)),
        );
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_READY)
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_PAYMENTS_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["items"][0]["id"], "intent-1");
        assert_eq!(stored["total"], 1);
    }

    #[test]
    fn payment_load_records_authoritative_empty() {
        let mut params = HashMap::new();
        record_payment_intent_load(&mut params, Ok(payment_payload(vec![], 0)));
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_EMPTY)
        );
        assert!(params.contains_key(ADMIN_PAYMENTS_DATA_PARAM));
    }

    #[test]
    fn payment_load_keeps_nonzero_total_empty_page_ready_for_recovery() {
        let mut params = HashMap::new();
        record_payment_intent_load(&mut params, Ok(payment_payload(vec![], 41)));
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_READY)
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_PAYMENTS_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["total"], 41);
    }

    #[test]
    fn payment_load_records_malformed_without_payload() {
        let mut params = HashMap::from([(
            ADMIN_PAYMENTS_DATA_PARAM.to_string(),
            "stale-sensitive-data".to_string(),
        )]);
        record_payment_intent_load(
            &mut params,
            Ok(serde_json::json!({ "items": [{ "id": "incomplete" }], "total": 1 })),
        );
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_MALFORMED)
        );
        assert!(!params.contains_key(ADMIN_PAYMENTS_DATA_PARAM));
    }

    #[test]
    fn payment_load_records_unavailable_without_payload() {
        let mut params = HashMap::from([(
            ADMIN_PAYMENTS_DATA_PARAM.to_string(),
            "stale-sensitive-data".to_string(),
        )]);
        record_payment_intent_load(&mut params, Err(()));
        assert_eq!(
            params.get(ADMIN_PAYMENTS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_PAYMENTS_UNAVAILABLE)
        );
        assert!(!params.contains_key(ADMIN_PAYMENTS_DATA_PARAM));
    }

    #[test]
    fn admin_news_load_records_only_the_projection_and_normalized_query() {
        let mut params = HashMap::new();
        record_admin_news_load(
            &mut params,
            &news_query(),
            AdminNewsLoad::Ready(news_payload(vec![news_item()], 41)),
        );

        assert_eq!(
            params.get(ADMIN_NEWS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_NEWS_READY)
        );
        assert_eq!(
            params.get(ADMIN_NEWS_PAGE_PARAM).map(String::as_str),
            Some("2")
        );
        assert_eq!(
            params.get(ADMIN_NEWS_STATUS_PARAM).map(String::as_str),
            Some("published")
        );
        let stored: serde_json::Value =
            serde_json::from_str(params.get(ADMIN_NEWS_DATA_PARAM).unwrap()).unwrap();
        assert_eq!(stored["articles"][0]["slug"], "migration-status");
        assert_eq!(stored["total"], 41);
        for forbidden in [
            "content",
            "author_wallet",
            "cover_image_url",
            "updated_at",
            "pinned_at",
        ] {
            assert!(
                stored["articles"][0].get(forbidden).is_none(),
                "{forbidden}"
            );
        }
    }

    #[test]
    fn admin_news_load_distinguishes_true_empty_from_an_out_of_range_page() {
        let query = news_query();
        let mut empty = HashMap::new();
        record_admin_news_load(
            &mut empty,
            &query,
            AdminNewsLoad::Empty(news_payload(Vec::new(), 0)),
        );
        assert_eq!(
            empty.get(ADMIN_NEWS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_NEWS_EMPTY)
        );
        assert!(empty.contains_key(ADMIN_NEWS_DATA_PARAM));

        let mut recoverable = HashMap::new();
        record_admin_news_load(
            &mut recoverable,
            &query,
            AdminNewsLoad::Ready(news_payload(Vec::new(), 41)),
        );
        assert_eq!(
            recoverable.get(ADMIN_NEWS_STATE_PARAM).map(String::as_str),
            Some(ADMIN_NEWS_READY)
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                recoverable.get(ADMIN_NEWS_DATA_PARAM).unwrap()
            )
            .unwrap()["total"],
            41
        );
    }

    #[test]
    fn admin_news_failure_states_remove_stale_projection_data() {
        for (load, expected) in [
            (AdminNewsLoad::Forbidden, ADMIN_NEWS_FORBIDDEN),
            (AdminNewsLoad::Unavailable, ADMIN_NEWS_UNAVAILABLE),
            (AdminNewsLoad::Malformed, ADMIN_NEWS_MALFORMED),
        ] {
            let mut params = HashMap::from([(
                ADMIN_NEWS_DATA_PARAM.to_string(),
                "stale-sensitive-news-data".to_string(),
            )]);
            record_admin_news_load(&mut params, &news_query(), load);
            assert_eq!(
                params.get(ADMIN_NEWS_STATE_PARAM).map(String::as_str),
                Some(expected)
            );
            assert!(!params.contains_key(ADMIN_NEWS_DATA_PARAM));
        }
    }

    #[test]
    fn admin_html_is_private_no_store_and_varies_by_session_credentials() {
        let response = private_admin_html_response(
            axum::http::StatusCode::OK,
            "private draft summary".to_string(),
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::VARY)
                .and_then(|value| value.to_str().ok()),
            Some("Cookie, Authorization")
        );
    }
}
