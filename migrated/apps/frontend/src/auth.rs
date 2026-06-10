//! Authentication helpers — cookie scheme + JWT verification.
//!
//! Mirrors the original Next.js middleware behavior: an `epsx_token` cookie
//! is treated as a bearer token, and we verify it with `epsx_auth::JwtAuth`
//! to get back a verified `AuthUser`. The same scheme is used by the
//! `/api/v1/auth/siwe` and `/api/v1/auth/demo` handlers to set the cookies.
//!
//! All API handlers in `api.rs` and the SSR handler in `ssr.rs` go through
//! `current_user` (or `require_*` variants) to enforce authentication.

use axum::http::HeaderMap;
use epsx_auth::{AuthError, AuthUser, JwtAuth};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub use epsx_auth::AuthUser as VerifiedAuthUser;

pub fn parse_cookies(headers: &HeaderMap) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(cookie_header) = headers.get("cookie").and_then(|h| h.to_str().ok()) {
        for pair in cookie_header.split(';') {
            let pair = pair.trim();
            if let Some(idx) = pair.find('=') {
                map.insert(pair[..idx].to_string(), pair[idx + 1..].to_string());
            }
        }
    }
    map
}

pub fn get_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    parse_cookies(headers).remove(name)
}

pub fn build_set_cookie(name: &str, value: &str, max_age_secs: i64) -> String {
    let secure = if std::env::var("EPSX_COOKIE_SECURE").ok().as_deref() == Some("1") { "; Secure" } else { "" };
    if max_age_secs <= 0 {
        format!("{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}", name, secure)
    } else {
        format!("{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}", name, value, max_age_secs, secure)
    }
}

pub fn build_clear_cookie(name: &str) -> String {
    build_set_cookie(name, "", 0)
}

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    if let Some(h) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
        if let Some(t) = h.strip_prefix("Bearer ") {
            return Some(t.to_string());
        }
    }
    get_cookie(headers, "epsx_token")
}

/// Resolve a verified `AuthUser` from the request. Returns `None` if no
/// token is present or verification fails. This is the only function
/// API handlers and the SSR layer should call to get the current user.
pub fn current_user(headers: &HeaderMap, jwt: &JwtAuth) -> Option<AuthUser> {
    let token = bearer_token(headers)?;
    jwt.verify(&token).ok()
}

pub fn require_user(headers: &HeaderMap, jwt: &JwtAuth) -> Result<AuthUser, AuthError> {
    let token = bearer_token(headers).ok_or(AuthError::Missing)?;
    jwt.verify(&token)
}

pub fn require_admin(headers: &HeaderMap, jwt: &JwtAuth) -> Result<AuthUser, AuthError> {
    let user = require_user(headers, jwt)?;
    if user.is_admin() { Ok(user) } else { Err(AuthError::Forbidden) }
}

pub fn require_editor(headers: &HeaderMap, jwt: &JwtAuth) -> Result<AuthUser, AuthError> {
    let user = require_user(headers, jwt)?;
    if user.is_editor() { Ok(user) } else { Err(AuthError::Forbidden) }
}

/// Construct a `JwtAuth` from the standard `EPSX_JWT_SECRET` env var, or
/// fall back to a deterministic dev secret. Production must set the env.
pub fn jwt_auth_from_env() -> Arc<JwtAuth> {
    let secret = std::env::var("EPSX_JWT_SECRET")
        .unwrap_or_else(|_| "epsx-dev-secret-do-not-use-in-prod".to_string());
    Arc::new(JwtAuth::from_secret(&secret))
}

// Re-export the legacy `AuthUser` for backwards compat with the siwe handler
// which has a different shape (id: Uuid, token: String, ...). We keep that
// struct in the BFF for cookie-set purposes, but the verified one comes
// from `epsx_auth`.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthUserSession {
    pub id: Uuid,
    pub address: String,
    pub chain_id: String,
    pub roles: Vec<String>,
    pub token: String,
}

impl AuthUserSession {
    pub fn display(&self) -> String {
        let s = self.address.as_str();
        if s.len() > 10 {
            format!("{}…{}", &s[..6], &s[s.len() - 4..])
        } else {
            s.to_string()
        }
    }
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin" || r == "super_admin")
    }
    pub fn is_editor(&self) -> bool {
        self.is_admin() || self.roles.iter().any(|r| r == "editor" || r == "content_manager")
    }
}
