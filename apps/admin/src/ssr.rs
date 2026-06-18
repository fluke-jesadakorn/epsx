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
use epsx_dioxus_ui::auth::User as UiUser;
use epsx_dioxus_ui::auth::user::AuthMethod;
use epsx_dioxus_ui::layout::shell::{AdminLayout, ServerUser};
use epsx_dioxus_ui::pages::{admin_pages, render_page, PageContext};
use std::collections::HashMap;

use super::AppState;
use super::auth;

pub async fn ssr_handler(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, _body) = request.into_parts();
    let path = parts.uri.path().to_string();
    let query = parts.uri.query().unwrap_or("").to_string();
    let headers = parts.headers.clone();

    // Resolve verified user from cookies/bearer
    let jwt = auth::jwt_auth_from_env();
    let user = auth::current_user(&headers, &jwt).map(|u| UiUser {
        id: u.user_id,
        address: u.address,
        chain_id: u.chain_id,
        roles: u.roles.clone(),
        email: None,
        tier: None,
        // Wave 7 — populate `permissions` from the JWT roles so the
        // `AdminAuthGate` component (which does exact-string match
        // against `required_permissions`) stops misfiring for admin
        // users. Previously this was `vec![]`, which made every
        // admin page render the gate panel instead of the body,
        // causing the wave6b smoke to flag 5 PARTIAL routes.
        //
        // UI-layer concern only: the canonical permission grammar
        // (`platform:resource:action` with wildcards) lives in
        // `apps/backend/src/core/permissions.rs`. This expansion
        // table mirrors the role strings the backend mints into
        // the JWT, so an admin token mints an admin perm set, an
        // editor token mints content perms, etc.
        permissions: auth::permissions_for_roles(&u.roles),
        // Wave 2 Track C — auth metadata fields. The admin BFF
        // doesn't have rich auth metadata, so we leave the new
        // optional fields at their defaults.
        last_login_at: None,
        auth_method: AuthMethod::Wallet,
        display_name: None,
    });

    // Admin: always render the admin page dispatcher
    let mut params = HashMap::new();
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
        demo_login_enabled: true,
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
    let no_layout_paths_override = Some(vec![
        "/login".to_string(),
        "/unauthorized".to_string(),
        "/access-denied".to_string(),
        "/permissions/policies".to_string(),
        "/developer-portal/api-keys/create".to_string(),
    ]);
    let body_element = AdminLayout::Auth {
        current_path: layout_path.clone(),
        server_user,
        is_authenticated,
        is_gated: None,
        no_layout_paths: no_layout_paths_override,
    }
    .render(
        body_element,
        None,
        None,
        None,
    );

    let body_html = dioxus_ssr::render_element(body_element);

    let doc = epsx_templates::page_shell_with_body_class(
        &meta.title,
        &meta.description,
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

    let doc = doc.replace("</body>", &format!("<script>{}</script></body>", wallet_shim()));

    (
        axum::http::StatusCode::OK,
        [("content-type", "text/html; charset=utf-8")],
        doc,
    ).into_response()
}

fn wallet_shim() -> &'static str {
    r#"
window.epsxWallet = {
  isAvailable: () => typeof window.ethereum !== 'undefined',
  request: (m, p) => window.ethereum ? window.ethereum.request({ method: m, params: p || [] }) : Promise.reject(new Error('No wallet')),
  address: () => window.ethereum && window.ethereum.selectedAddress || null,
  chainId: () => window.ethereum && window.ethereum.chainId || '0x38',
};
"#
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
    use epsx_dioxus_ui::pages::PageContext;

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

    /// Render a page through the admin BFF render path (without
    /// `page_shell_with_body_class`) so we can assert on the
    /// layout-wrapped HTML in isolation.
    fn render_admin_html(path: &str) -> String {
        let ctx = build_ctx(path);
        let admin_path = path.trim_start_matches("/admin").to_string();
        let mut c = ctx.clone();
        c.path = if admin_path.is_empty() { "/".to_string() } else { admin_path };
        let (_meta, body) = admin_pages::dispatch(&c);
        let server_user: Option<ServerUser> = None;
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
            current_path: path.to_string(),
            server_user,
            is_authenticated: false,
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
}
