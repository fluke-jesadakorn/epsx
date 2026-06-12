use axum::http::HeaderMap;
use epsx_auth::{AuthError, AuthUser, JwtAuth};
use std::collections::HashMap;
use std::sync::Arc;

/// Role → permission set expansion for the admin BFF SSR layer.
///
/// `User::has_permission` in `epsx_dioxus_ui::auth::user` does exact-string
/// match only (no wildcards), so the 2-segment strings the admin pages
/// declare on `<AdminAuthGate required_permissions=...>` must appear
/// literally in `UiUser.permissions` for the gate to pass.
///
/// Mirrors the role predicates on `AuthUser` (`is_admin`, `is_editor`,
/// `is_merchant` in `shared/rust/auth/src/lib.rs`): same role strings
/// the JWT already carries, no new auth grammar introduced.
///
/// Order of role checks matters: `admin` / `super_admin` get the full
/// admin set; `editor` / `content_manager` get content-moderation perms;
/// `merchant` gets financial perms. The returned set is the union, so
/// a user with multiple roles gets the broadest grant.
///
/// Wave 7 — the admin BFF previously set `permissions: vec![]` when
/// building `UiUser`, which made every `AdminAuthGate` misfire (the
/// gate's `has_permission` check would always say "missing"). This
/// function is the fix.
pub fn permissions_for_roles(roles: &[String]) -> Vec<String> {
    // Use a set internally to dedupe, then return as Vec<String> for
    // serialization stability. The dedupe also makes the union case
    // (admin + editor in the same token) cheap.
    let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    if is_admin_role(roles) {
        // Full admin set — every 2-segment perm checked by
        // `AdminAuthGate` across the 16 admin routes in
        // `shared/rust/dioxus_ui::pages::admin_pages::*`. Keep this
        // list in sync with the call sites; a missing entry will
        // show up as a smoke-test PARTIAL route.
        for p in [
            // Dashboard / cross-cutting
            "admin:*",
            "admin:auth",
            // Analytics
            "analytics:read",
            // Content moderation
            "audit:read",
            "news:manage",
            "notifications:manage",
            "policies:read",
            "policies:manage",
            "media:read",
            "media:manage",
            // Financial surface
            "payments:read",
            "payments:manage",
            "wallets:manage",
            // Platform
            "settings:manage",
            "chat:manage",
            "developer:manage",
        ] {
            set.insert(p.to_string());
        }
    }

    if is_editor_role(roles) {
        for p in [
            "audit:read",
            "news:manage",
            "notifications:manage",
            "policies:manage",
            "media:manage",
        ] {
            set.insert(p.to_string());
        }
    }

    if is_merchant_role(roles) {
        for p in [
            "payments:read",
            "payments:manage",
            "wallets:manage",
        ] {
            set.insert(p.to_string());
        }
    }

    set.into_iter().collect()
}

fn is_admin_role(roles: &[String]) -> bool {
    roles.iter().any(|r| r == "admin" || r == "super_admin" || r == "Admin")
}

fn is_editor_role(roles: &[String]) -> bool {
    roles.iter().any(|r| r == "editor" || r == "content_manager")
}

fn is_merchant_role(roles: &[String]) -> bool {
    roles.iter().any(|r| r == "merchant")
}

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

pub fn jwt_auth_from_env() -> Arc<JwtAuth> {
    let secret = std::env::var("EPSX_JWT_SECRET")
        .unwrap_or_else(|_| "epsx-dev-secret-do-not-use-in-prod".to_string());
    Arc::new(JwtAuth::from_secret(&secret))
}

#[cfg(test)]
mod tests {
    //! Unit tests for `permissions_for_roles` — the role→permission
    //! expansion used by the admin BFF when building `UiUser` for SSR.
    //!
    //! These tests pin the exact 2-segment permission strings the
    //! `AdminAuthGate` call sites check across the 16 admin pages.
    //! If a new admin page adds a `required_permissions` entry that's
    //! not in the `admin` set here, the wave6b admin-smoke harness
    //! will flag it as a PARTIAL route — that's the regression signal.
    use super::*;

    fn has(perms: &[String], needle: &str) -> bool {
        perms.iter().any(|p| p == needle)
    }

    #[test]
    fn admin_role_gets_full_set() {
        let perms = permissions_for_roles(&["admin".to_string()]);
        // Every perm the wave6b smoke flagged as PARTIAL must be present.
        for needle in [
            "admin:*",
            "analytics:read",
            "policies:read",
            "policies:manage",
            "media:read",
            "media:manage",
            "payments:read",
            "payments:manage",
            "wallets:manage",
            "audit:read",
            "news:manage",
            "notifications:manage",
            "settings:manage",
            "chat:manage",
            "developer:manage",
            "admin:auth",
        ] {
            assert!(has(&perms, needle), "admin role must include {needle}, got {perms:?}");
        }
    }

    #[test]
    fn super_admin_role_gets_full_set() {
        let perms = permissions_for_roles(&["super_admin".to_string()]);
        assert!(has(&perms, "admin:*"));
        assert!(has(&perms, "analytics:read"));
        assert!(has(&perms, "wallets:manage"));
    }

    #[test]
    fn editor_role_gets_content_perms_only() {
        let perms = permissions_for_roles(&["editor".to_string()]);
        // Content-moderation perms present.
        for needle in ["audit:read", "news:manage", "notifications:manage", "policies:manage", "media:manage"] {
            assert!(has(&perms, needle), "editor must have {needle}, got {perms:?}");
        }
        // Admin-only perms NOT present (e.g. settings, wallets, payments).
        for needle in ["settings:manage", "wallets:manage", "payments:read", "developer:manage"] {
            assert!(!has(&perms, needle), "editor must NOT have {needle}, got {perms:?}");
        }
    }

    #[test]
    fn content_manager_role_treated_as_editor() {
        let perms = permissions_for_roles(&["content_manager".to_string()]);
        assert!(has(&perms, "news:manage"));
        assert!(!has(&perms, "wallets:manage"));
    }

    #[test]
    fn merchant_role_gets_financial_perms() {
        let perms = permissions_for_roles(&["merchant".to_string()]);
        for needle in ["payments:read", "payments:manage", "wallets:manage"] {
            assert!(has(&perms, needle), "merchant must have {needle}, got {perms:?}");
        }
        // Out-of-scope for merchant: content, settings, audit, dev.
        for needle in ["audit:read", "news:manage", "settings:manage", "developer:manage"] {
            assert!(!has(&perms, needle), "merchant must NOT have {needle}, got {perms:?}");
        }
    }

    #[test]
    fn unknown_role_gets_empty() {
        let perms = permissions_for_roles(&["user".to_string(), "vip".to_string()]);
        assert!(perms.is_empty(), "unknown roles must yield empty perm set, got {perms:?}");
    }

    #[test]
    fn empty_roles_get_empty() {
        let perms = permissions_for_roles(&[]);
        assert!(perms.is_empty(), "empty roles must yield empty perm set, got {perms:?}");
    }

    #[test]
    fn admin_plus_editor_is_union_deduped() {
        // A user holding both `admin` and `editor` should get the
        // admin superset (since admin already includes editor perms).
        let perms = permissions_for_roles(&["admin".to_string(), "editor".to_string()]);
        assert!(has(&perms, "admin:*"));
        assert!(has(&perms, "audit:read"));
        // Length should equal the admin-set size, not admin+editor size.
        let admin_only = permissions_for_roles(&["admin".to_string()]);
        assert_eq!(perms.len(), admin_only.len(),
            "admin+editor must dedupe to admin-set size, got {} vs admin-only {}",
            perms.len(), admin_only.len());
    }

    #[test]
    fn returned_vec_is_deterministic_order() {
        // The function uses a BTreeSet internally, so output is sorted.
        // Pin that — SSR caching and snapshot tests rely on stable order.
        let perms = permissions_for_roles(&["admin".to_string()]);
        let mut sorted = perms.clone();
        sorted.sort();
        assert_eq!(perms, sorted, "permissions_for_roles must return sorted output");
    }
}
