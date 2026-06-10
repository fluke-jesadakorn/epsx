//! Dioxus SSR rendering for the admin BFF.
//!
//! The HTTP request is parsed into a `PageContext` (path, query, user) and
//! dispatched to the appropriate `rsx!` page from `epsx_dioxus_ui::pages`.
//! The HTML is wrapped in the EPSX design-system page shell so the visuals
//! match the Next.js admin 1:1.

use axum::{
    extract::{Request, State},
    response::{IntoResponse, Response},
};
use epsx_dioxus_ui::auth::User as UiUser;
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
        roles: u.roles,
        email: None,
        tier: None,
        permissions: vec![],
    });

    // Admin: always render the admin page dispatcher
    let mut params = HashMap::new();
    let ctx = PageContext {
        user,
        path: path.clone(),
        query: query.clone(),
        params,
        api_url: state.api_url.clone(),
        demo_login_enabled: true,
    };

    // Use the dedicated admin dispatcher regardless of `is_admin` so the
    // admin's own auth middleware (if installed) can decide. The frontend
    // BFF will have the same UX.
    let (meta, body_element) = if path.starts_with("/admin") {
        let p = path.trim_start_matches("/admin").to_string();
        let mut c = ctx.clone();
        c.path = if p.is_empty() { "/".to_string() } else { p };
        admin_pages::dispatch(&c)
    } else {
        render_page(&ctx, true)
    };

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
