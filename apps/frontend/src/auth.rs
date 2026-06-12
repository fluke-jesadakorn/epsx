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

/// Role → permission set expansion for the frontend BFF SSR layer.
///
/// Mirrors `apps/admin/src/auth::permissions_for_roles` for the
/// user-side 2-segment permission grammar (e.g. `dashboard:read`,
/// `chat:read`, `payments:read`). `User::has_permission` in
/// `epsx_dioxus_ui::auth::user` does exact-string match only, so
/// these strings must appear literally in `UiUser.permissions` for
/// the gates on the user-side pages to pass.
///
/// UI-layer concern only: the canonical permission grammar
/// (`platform:resource:action` with wildcards) lives in
/// `apps/backend/src/core/permissions.rs`. This function is a
/// presentation-layer translation table that derives a 2-segment
/// perm set from the JWT role list the backend mints. Real
/// authorization enforcement happens in the backend.
pub fn permissions_for_roles(roles: &[String]) -> Vec<String> {
    let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    // Admin / super_admin see everything (admin set + standard read perms).
    if is_admin_role(roles) {
        for p in [
            // Standard read perms (any signed-in user with admin role)
            "dashboard:read",
            "chat:read",
            "chat:write",
            "news:read",
            "notifications:read",
            "permissions:read",
            "profile:read",
            "profile:write",
            "payments:read",
            "analytics:read",
            "developer:read",
            "plan subscription",
            // Admin-side perms (so a signed-in admin navigating to /admin
            // also gets past the AdminAuthGate; matches the admin BFF).
            "admin:*",
            "admin:auth",
            "audit:read",
            "news:manage",
            "notifications:manage",
            "policies:read",
            "policies:manage",
            "media:read",
            "media:manage",
            "payments:manage",
            "wallets:manage",
            "settings:manage",
            "chat:manage",
            "developer:manage",
        ] {
            set.insert(p.to_string());
        }
    }

    // Editor / content_manager — content perms (read-only on user pages).
    if is_editor_role(roles) {
        for p in [
            "dashboard:read",
            "news:read",
            "notifications:read",
            "audit:read",
            "news:manage",
            "notifications:manage",
            "policies:manage",
            "media:manage",
        ] {
            set.insert(p.to_string());
        }
    }

    // Merchant — financial surface.
    if is_merchant_role(roles) {
        for p in [
            "dashboard:read",
            "payments:read",
            "payments:manage",
            "wallets:manage",
            "plan subscription",
        ] {
            set.insert(p.to_string());
        }
    }

    set.into_iter().collect()
}

fn is_admin_role(roles: &[String]) -> bool {
    // Match the frontend BFF's `AuthUserSession::is_admin` convention:
    // `admin` or `super_admin`. (Admin BFF additionally accepts
    // capitalized `Admin`; the JWT mints lowercase tags so this is
    // equivalent in practice.)
    roles.iter().any(|r| r == "admin" || r == "super_admin")
}

fn is_editor_role(roles: &[String]) -> bool {
    roles.iter().any(|r| r == "editor" || r == "content_manager")
}

fn is_merchant_role(roles: &[String]) -> bool {
    roles.iter().any(|r| r == "merchant")
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

#[cfg(test)]
mod tests {
    //! Unit tests for `permissions_for_roles` — the role→permission
    //! expansion used by the frontend BFF when building `UiUser` for SSR.
    //!
    //! Pins the 2-segment permission strings the user-side pages
    //! declare on `<AuthGate required_permissions=...>` and on the
    //! admin-side `AdminAuthGate` (for the case where a user with
    //! `admin` role navigates through the frontend BFF to /admin).
    use super::*;

    fn has(perms: &[String], needle: &str) -> bool {
        perms.iter().any(|p| p == needle)
    }

    #[test]
    fn admin_role_gets_full_set() {
        let perms = permissions_for_roles(&["admin".to_string()]);
        for needle in [
            // user-side reads
            "dashboard:read",
            "chat:read",
            "chat:write",
            "news:read",
            "notifications:read",
            "permissions:read",
            "profile:read",
            "profile:write",
            "payments:read",
            "analytics:read",
            "developer:read",
            "plan subscription",
            // admin-side (so /admin works from frontend BFF)
            "admin:*",
            "wallets:manage",
            "policies:manage",
        ] {
            assert!(has(&perms, needle), "admin must have {needle}, got {perms:?}");
        }
    }

    #[test]
    fn super_admin_role_gets_full_set() {
        let perms = permissions_for_roles(&["super_admin".to_string()]);
        assert!(has(&perms, "admin:*"));
        assert!(has(&perms, "dashboard:read"));
        assert!(has(&perms, "wallets:manage"));
    }

    #[test]
    fn editor_role_gets_content_reads_and_manages() {
        let perms = permissions_for_roles(&["editor".to_string()]);
        for needle in ["dashboard:read", "news:read", "notifications:read", "audit:read", "news:manage", "media:manage"] {
            assert!(has(&perms, needle), "editor must have {needle}, got {perms:?}");
        }
        // Editor must NOT have financial or admin platform perms.
        for needle in ["payments:manage", "wallets:manage", "settings:manage", "admin:*"] {
            assert!(!has(&perms, needle), "editor must NOT have {needle}, got {perms:?}");
        }
    }

    #[test]
    fn merchant_role_gets_financial_perms() {
        let perms = permissions_for_roles(&["merchant".to_string()]);
        for needle in ["dashboard:read", "payments:read", "payments:manage", "wallets:manage", "plan subscription"] {
            assert!(has(&perms, needle), "merchant must have {needle}, got {perms:?}");
        }
        for needle in ["news:manage", "audit:read", "settings:manage", "admin:*"] {
            assert!(!has(&perms, needle), "merchant must NOT have {needle}, got {perms:?}");
        }
    }

    #[test]
    fn empty_roles_yield_empty() {
        let perms = permissions_for_roles(&[]);
        assert!(perms.is_empty(), "empty roles must yield empty perms, got {perms:?}");
    }

    #[test]
    fn unknown_role_yields_empty() {
        let perms = permissions_for_roles(&["user".to_string(), "vip".to_string()]);
        assert!(perms.is_empty(), "unknown roles must yield empty perms, got {perms:?}");
    }

    #[test]
    fn admin_plus_editor_is_deduped() {
        let combined = permissions_for_roles(&["admin".to_string(), "editor".to_string()]);
        let admin_only = permissions_for_roles(&["admin".to_string()]);
        assert_eq!(combined.len(), admin_only.len(),
            "admin+editor must dedupe to admin-set size");
    }

    #[test]
    fn returned_vec_is_sorted() {
        let perms = permissions_for_roles(&["admin".to_string()]);
        let mut sorted = perms.clone();
        sorted.sort();
        assert_eq!(perms, sorted, "permissions_for_roles must return sorted output");
    }
}
