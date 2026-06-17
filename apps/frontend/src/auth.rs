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
///
/// Wave 21 — if `EPSX_DEV_AUTH_BYPASS=1` is set, returns the hardcoded
/// dev admin user (`epsx_bff::dev_bypass::dev_bypass_user()`) instead of
/// reading the `epsx_token` cookie / `Authorization` header. This is
/// the dev-loop / pixel-recheck escape hatch; default is OFF, no
/// behavior change when unset.
///
/// Wave 24 t3p — if `EPSX_DEV_AUTH_FORCE_UNAUTH=1` is ALSO set, the
/// bypass short-circuit is skipped and the request falls through to
/// the normal JWT-verify path (which will return `None` because the
/// dev-bypass cookie is NOT a real JWT). This is the pixel-recheck
/// escape hatch for the `redirect-chain-differs` issue — see
/// `epsx_bff::dev_bypass::DEV_FORCE_UNAUTH_ENV` for the full
/// rationale. Default is OFF, no behavior change when unset.
pub fn current_user(headers: &HeaderMap, jwt: &JwtAuth) -> Option<AuthUser> {
    if !epsx_bff::dev_bypass::is_dev_force_unauth_enabled() {
        if let Some(user) = epsx_bff::dev_bypass::dev_bypass_user() {
            return Some(user);
        }
    }
    let token = bearer_token(headers)?;
    jwt.verify(&token).ok()
}

pub fn require_user(headers: &HeaderMap, jwt: &JwtAuth) -> Result<AuthUser, AuthError> {
    if !epsx_bff::dev_bypass::is_dev_force_unauth_enabled() {
        if let Some(user) = epsx_bff::dev_bypass::dev_bypass_user() {
            return Ok(user);
        }
    }
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
/// `chat:read`, `payments:read`). `User::has_permission` is
/// wildcard-aware (wave7): a 2-seg `<prefix>:*` perm satisfies any
/// 2-seg `<prefix>:<action>` required perm. So the BFF emits a
/// small wildcard set per role group, not a flat list of every
/// literal perm the user-side pages check.
///
/// UI-layer concern only: the canonical permission grammar
/// (`platform:resource:action` with wildcards) lives in
/// `apps/backend/src/core/permissions.rs`. This function is a
/// presentation-layer translation table that derives a 2-segment
/// perm set from the JWT role list the backend mints. Real
/// authorization enforcement happens in the backend.
pub fn permissions_for_roles(roles: &[String]) -> Vec<String> {
    let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    // Admin / super_admin: catch-all `*:*` super-admin wildcard
    // satisfies any 2-seg or 3-seg perm. Plus the `admin:*` 2-seg
    // wildcard for symmetry with the admin BFF and for the
    // 2-seg-only gates on admin pages.
    if is_admin_role(roles) {
        for p in [
            "*:*",
            "admin:*",
            "dashboard:*",
            "chat:*",
            "news:*",
            "notifications:*",
            "permissions:*",
            "profile:*",
            "payments:*",
            "analytics:*",
            "developer:*",
            "wallets:*",
            "policies:*",
            "media:*",
            "audit:*",
            "settings:*",
            "plan subscription",
        ] {
            set.insert(p.to_string());
        }
    }

    // Editor / content_manager: standard read perms + content manage.
    if is_editor_role(roles) {
        for p in [
            "dashboard:read",
            "news:*",
            "notifications:*",
            "audit:read",
            "policies:*",
            "media:*",
        ] {
            set.insert(p.to_string());
        }
    }

    // Merchant — financial surface.
    if is_merchant_role(roles) {
        for p in [
            "dashboard:read",
            "payments:*",
            "wallets:*",
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
        // Admin emits the catch-all `*:*` plus a handful of
        // 2-seg `prefix:*` wildcards. Verify the wildcards are
        // there and the role satisfies the perms the gates check.
        for needle in [
            "*:*",
            "admin:*",
            "dashboard:*",
            "chat:*",
            "news:*",
            "notifications:*",
            "permissions:*",
            "profile:*",
            "payments:*",
            "analytics:*",
            "developer:*",
            "wallets:*",
            "policies:*",
            "media:*",
            "audit:*",
            "settings:*",
            "plan subscription",
        ] {
            assert!(has(&perms, needle), "admin must have {needle}, got {perms:?}");
        }
        // And confirm the wildcard actually satisfies a literal required perm.
        assert!(epsx_dioxus_ui::auth::user::permission_matches("dashboard:*", "dashboard:read"));
        assert!(epsx_dioxus_ui::auth::user::permission_matches("admin:*", "admin:auth"));
    }

    #[test]
    fn super_admin_role_gets_full_set() {
        let perms = permissions_for_roles(&["super_admin".to_string()]);
        assert!(has(&perms, "*:*"));
        assert!(has(&perms, "admin:*"));
        assert!(has(&perms, "dashboard:*"));
    }

    #[test]
    fn editor_role_gets_content_reads_and_manages() {
        let perms = permissions_for_roles(&["editor".to_string()]);
        for needle in ["dashboard:read", "news:*", "notifications:*", "audit:read", "policies:*", "media:*"] {
            assert!(has(&perms, needle), "editor must have {needle}, got {perms:?}");
        }
        // Editor must NOT have financial or admin platform perms.
        for needle in ["payments:manage", "payments:*", "wallets:manage", "wallets:*", "settings:manage", "admin:*", "*:*"] {
            assert!(!has(&perms, needle), "editor must NOT have {needle}, got {perms:?}");
        }
    }

    #[test]
    fn merchant_role_gets_financial_perms() {
        let perms = permissions_for_roles(&["merchant".to_string()]);
        for needle in ["dashboard:read", "payments:*", "wallets:*", "plan subscription"] {
            assert!(has(&perms, needle), "merchant must have {needle}, got {perms:?}");
        }
        for needle in ["news:manage", "audit:read", "settings:manage", "admin:*", "*:*"] {
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
        // The union of admin and editor perm sets must not contain
        // duplicates (BTreeSet-backed). We don't assert
        // "admin ⊇ editor" because editor now adds
        // `dashboard:read` literal + 3-seg `policies:*` / `media:*`
        // wildcards that admin doesn't emit. The union is the
        // actual contract.
        let combined = permissions_for_roles(&["admin".to_string(), "editor".to_string()]);
        let mut sorted: Vec<&String> = combined.iter().collect();
        sorted.sort_by_key(|p| p.as_str());
        let mut prev: Option<&str> = None;
        for p in &sorted {
            if let Some(prev_s) = prev {
                assert_ne!(*p, prev_s, "duplicate perm {p} in union");
            }
            prev = Some(p.as_str());
        }
    }

    #[test]
    fn returned_vec_is_sorted() {
        let perms = permissions_for_roles(&["admin".to_string()]);
        let mut sorted = perms.clone();
        sorted.sort();
        assert_eq!(perms, sorted, "permissions_for_roles must return sorted output");
    }

    // ── Wave 23 T3 — `current_user` dev-bypass short-circuit ─────
    //
    // `current_user` MUST return the hardcoded dev admin when
    // `EPSX_DEV_AUTH_BYPASS=1`, regardless of the request headers.
    // This is the pixel-recheck / dev-loop escape hatch — the
    // harness sets the env var and the SSR layer treats every
    // request as logged in. Default OFF (env var unset) must
    // fall through to the normal JWT-verify path.
    //
    // `std::env::set_var` is not thread-safe; serialize with a
    // Mutex to avoid races against parallel tests in the same
    // process. (Per `memory/tokio-runtime-quirks.md`.)
    use std::sync::Mutex;
    static BYPASS_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn empty_headers() -> HeaderMap { HeaderMap::new() }

    fn dev_jwt() -> Arc<JwtAuth> { jwt_auth_from_env() }

    #[test]
    fn current_user_returns_dev_admin_when_bypass_is_on() {
        let _g = BYPASS_ENV_LOCK.lock().unwrap();
        std::env::set_var("EPSX_DEV_AUTH_BYPASS", "1");
        std::env::remove_var("EPSX_DEV_AUTH_FORCE_UNAUTH");
        let u = current_user(&empty_headers(), &dev_jwt())
            .expect("current_user should return Some when bypass is ON");
        assert_eq!(u.user_id, "dev-bypass", "bypass user_id should be the hardcoded dev-bypass");
        assert_eq!(u.address, "0x000000000000000000000000000000000000d3v1",
                   "bypass address should be the 20-byte hex placeholder, not a real wallet");
        assert!(u.is_admin(), "bypass user must satisfy is_admin() so AdminAuthGate passes");
        // require_user must also short-circuit through the bypass.
        let r = require_user(&empty_headers(), &dev_jwt())
            .expect("require_user should return Ok when bypass is ON");
        assert_eq!(r.user_id, "dev-bypass");
        std::env::remove_var("EPSX_DEV_AUTH_BYPASS");
    }

    #[test]
    fn current_user_returns_none_when_bypass_is_off() {
        let _g = BYPASS_ENV_LOCK.lock().unwrap();
        std::env::remove_var("EPSX_DEV_AUTH_BYPASS");
        std::env::remove_var("EPSX_DEV_AUTH_FORCE_UNAUTH");
        // No token in the headers → current_user must return None,
        // NOT fall through to the bypass (which is off).
        let u = current_user(&empty_headers(), &dev_jwt());
        assert!(u.is_none(),
                "current_user must return None when bypass is OFF and no token is present. \
                 Got: {u:?}");
        // require_user must also return Missing.
        let r = require_user(&empty_headers(), &dev_jwt());
        assert!(r.is_err(),
                "require_user must return Err(Missing) when bypass is OFF and no token is present. \
                 Got: {r:?}");
    }

    #[test]
    fn current_user_bypass_ignores_request_headers() {
        // The dev bypass should NOT inspect the `Cookie` or
        // `Authorization` headers — it returns the hardcoded user
        // regardless. This is the core guarantee: setting the
        // env var turns the whole BFF into a "logged in as admin"
        // mode, no client cooperation needed.
        let _g = BYPASS_ENV_LOCK.lock().unwrap();
        std::env::set_var("EPSX_DEV_AUTH_BYPASS", "1");
        std::env::remove_var("EPSX_DEV_AUTH_FORCE_UNAUTH");
        let mut h = HeaderMap::new();
        // Add a bogus cookie + bearer — the bypass must ignore them.
        h.insert("cookie", "epsx_token=bogus".parse().unwrap());
        h.insert("authorization", "Bearer also-bogus".parse().unwrap());
        let u = current_user(&h, &dev_jwt())
            .expect("bypass must return the dev admin regardless of incoming headers");
        assert_eq!(u.user_id, "dev-bypass");
        assert_eq!(u.address, "0x000000000000000000000000000000000000d3v1");
        std::env::remove_var("EPSX_DEV_AUTH_BYPASS");
    }

    // ── Wave 24 t3p — force-unauth override ───────────────────────
    //
    // `EPSX_DEV_AUTH_FORCE_UNAUTH=1` flips the bypass short-circuit
    // off. With both vars set, `current_user` must return None when
    // the request carries no real JWT, mirroring the unauth prod
    // baseline for the pixel-recheck harness.
    #[test]
    fn current_user_force_unauth_overrides_bypass() {
        let _g = BYPASS_ENV_LOCK.lock().unwrap();
        std::env::set_var("EPSX_DEV_AUTH_BYPASS", "1");
        std::env::set_var("EPSX_DEV_AUTH_FORCE_UNAUTH", "1");
        // No real token in the headers; with force-unauth, the bypass
        // must NOT short-circuit. JWT verify fails on no token → None.
        let u = current_user(&empty_headers(), &dev_jwt());
        assert!(u.is_none(),
                "current_user must return None when both bypass and force-unauth are ON. \
                 Got: {u:?}");
        // require_user must return Err(Missing) — not Ok(dev-bypass).
        let r = require_user(&empty_headers(), &dev_jwt());
        assert!(r.is_err(),
                "require_user must return Err(Missing) when both bypass and force-unauth are ON. \
                 Got: {r:?}");
        // Clean up both vars so the other tests don't see a leak.
        std::env::remove_var("EPSX_DEV_AUTH_BYPASS");
        std::env::remove_var("EPSX_DEV_AUTH_FORCE_UNAUTH");
    }
}
