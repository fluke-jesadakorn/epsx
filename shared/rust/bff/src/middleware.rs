//! Axum middleware: security headers, ?logout= handler, admin route guard.

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use epsx_auth::JwtAuth;
use std::sync::Arc;

#[derive(Clone)]
pub struct SecurityConfig {
    pub jwt: Arc<JwtAuth>,
    pub admin_only: bool,
    pub hsts: bool,
}

impl SecurityConfig {
    pub fn frontend(jwt: Arc<JwtAuth>) -> Self {
        Self {
            jwt,
            admin_only: false,
            hsts: std::env::var("EPSX_HSTS").ok().as_deref() == Some("1"),
        }
    }
    pub fn admin(jwt: Arc<JwtAuth>) -> Self {
        Self {
            jwt,
            admin_only: true,
            hsts: std::env::var("EPSX_HSTS").ok().as_deref() == Some("1"),
        }
    }
}

pub async fn security_headers(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();

    if query.contains("logout=1") {
        let mut resp = Redirect::to(&path).into_response();
        clear_epsx_cookies(&mut resp);
        apply_security_headers_to(resp.headers_mut(), &path, false, false);
        return resp;
    }

    let mut resp = next.run(req).await;
    let admin = is_admin_path(&path);
    apply_security_headers_to(resp.headers_mut(), &path, admin, true);
    resp
}

fn is_admin_path(p: &str) -> bool {
    matches!(p,
        "/admin" | "/analytics" | "/audit-log" | "/chat" | "/developer-portal"
        | "/media" | "/news" | "/news/create" | "/notifications"
        | "/notifications/create" | "/notifications/manage" | "/payments"
        | "/policies" | "/settings" | "/unauthorized" | "/wallet-management"
    ) || p.starts_with("/admin/")
        || p.starts_with("/wallet-management/")
        || p.starts_with("/news/")
        || p.starts_with("/notifications/")
        || p.starts_with("/developer-portal/")
        || p.starts_with("/chat/")
        || p.starts_with("/audit-log/")
        || p.starts_with("/policies/")
        || p.starts_with("/media/")
        || p.starts_with("/settings/")
        || p.starts_with("/payments/")
}

fn apply_security_headers_to(headers: &mut axum::http::HeaderMap, _path: &str, admin: bool, hsts: bool) {
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("x-frame-options", HeaderValue::from_static(if admin { "SAMEORIGIN" } else { "DENY" }));
    headers.insert("referrer-policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
    headers.insert("permissions-policy", HeaderValue::from_static("camera=(), microphone=(), geolocation=()"));
    if hsts {
        headers.insert("strict-transport-security", HeaderValue::from_static("max-age=31536000; includeSubDomains"));
    }
    if !headers.contains_key("content-security-policy") {
        // Wave 24 t3p — `https://unpkg.com` added to script-src.
        // The footer layout includes `<script src="https://unpkg.com/lucide@latest">`
        // for the Lucide icon set (matches the OLD prod layout). The
        // cdn.jsdelivr.net origin was already allowlisted; unpkg was
        // missed. Without it, every route logged a CSP console
        // error and the icons never rendered. Same allowlist for
        // admin + non-admin — the footer is shared.
        let csp = if admin {
            "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net https://unpkg.com; style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' ws: wss:; frame-ancestors 'self';"
        } else {
            "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net https://unpkg.com; style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' ws: wss:; frame-ancestors 'none';"
        };
        if let Ok(v) = HeaderValue::from_str(csp) {
            headers.insert("content-security-policy", v);
        }
    }
}

fn clear_epsx_cookies(resp: &mut Response) {
    let names = ["epsx_token", "epsx_user_id", "epsx_user_address", "epsx_chain_id", "epsx_refresh"];
    for n in names {
        let v = format!("{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0", n);
        if let Ok(hv) = HeaderValue::from_str(&v) {
            resp.headers_mut().append(header::SET_COOKIE, hv);
        }
    }
}

pub async fn verify_bearer_or_cookie(
    State(cfg): State<Arc<SecurityConfig>>,
    req: Request,
    next: Next,
) -> Response {
    let token = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            req.headers().get("cookie")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| {
                    s.split(';').find_map(|p| {
                        let p = p.trim();
                        if let Some(rest) = p.strip_prefix("epsx_token=") { Some(rest.to_string()) } else { None }
                    })
                })
        });
    if let Some(t) = token {
        if let Ok(user) = cfg.jwt.verify(&t) {
            let mut req = req;
            req.extensions_mut().insert(user);
            return next.run(req).await;
        }
    }
    next.run(req).await
}
