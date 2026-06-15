use axum::http::HeaderMap;
use epsx_auth::{AuthError, AuthUser, JwtAuth};
use std::collections::HashMap;
use std::sync::Arc;

/// Role → permission set expansion for the admin BFF SSR layer.
///
/// `User::has_permission` in `epsx_dioxus_ui::auth::user` is
/// wildcard-aware (as of Wave 7): exact match, `*:*` / `*:*:*`
/// super-admin, `<pl>:*:*` / `<pl>:<re>:*` 3-segment wildcards, and
/// 2-segment `<prefix>:*`. So the BFF can emit a small set of
/// wildcard-bearing perms per role group instead of enumerating
/// every literal perm the 16 admin pages check.
///
/// Mirrors the role predicates on `AuthUser` (`is_admin`, `is_editor`,
/// `is_merchant` in `shared/rust/auth/src/lib.rs`): same role strings
/// the JWT already carries, no new auth grammar introduced.
///
/// Wave 7 — the admin BFF previously set `permissions: vec![]` when
/// building `UiUser`, which made every `AdminAuthGate` misfire. This
/// function is the fix.
pub fn permissions_for_roles(roles: &[String]) -> Vec<String> {
    let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    if is_admin_role(roles) {
        // Admin / super_admin: 2-seg `admin:*` covers every admin-gate
        // perm. Add a couple of explicit 2-seg entries for the
        // cross-feature perms (analytics, payments, etc.) — these
        // match the AdminAuthGate call sites that don't use the
        // `admin:` prefix.
        for p in [
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
        ] {
            set.insert(p.to_string());
        }
    }

    if is_editor_role(roles) {
        // Editor / content_manager: 3-seg wildcard covers all content
        // actions; explicit 2-seg perms for the gates that use the
        // UI-layer 2-seg grammar.
        for p in [
            "admin:audit:*",
            "admin:news:*",
            "admin:notifications:*",
            "admin:policies:*",
            "admin:media:*",
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
            "admin:payments:*",
            "admin:wallets:*",
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
    if let Some(user) = epsx_bff::dev_bypass::dev_bypass_user() {
        return Some(user);
    }
    let token = bearer_token(headers)?;
    jwt.verify(&token).ok()
}

pub fn require_user(headers: &HeaderMap, jwt: &JwtAuth) -> Result<AuthUser, AuthError> {
    if let Some(user) = epsx_bff::dev_bypass::dev_bypass_user() {
        return Ok(user);
    }
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
        // Literal perms the BFF explicitly emits. The 2-seg `admin:*`
        // wildcard covers everything else (e.g. `admin:auth`).
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
        ] {
            assert!(has(&perms, needle), "admin role must include {needle}, got {perms:?}");
        }
    }

    #[test]
    fn admin_wildcard_grants_admin_auth_via_2seg_prefix() {
        // The AdminAuthGate for /admin/auth gates on `admin:auth`.
        // The BFF doesn't emit `admin:auth` literally — the 2-seg
        // `admin:*` wildcard should satisfy it via
        // `permission_matches("admin:*", "admin:auth")`.
        let perms = permissions_for_roles(&["admin".to_string()]);
        assert!(!has(&perms, "admin:auth"),
            "admin:auth should NOT be a literal perm (covered by admin:* wildcard)");
        let matches = epsx_dioxus_ui::auth::user::permission_matches("admin:*", "admin:auth");
        assert!(matches, "admin:* wildcard should match admin:auth required perm");
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
        // Content-moderation perms present (literal 2-seg + 3-seg wildcards).
        for needle in [
            "audit:read",
            "news:manage",
            "notifications:manage",
            "policies:manage",
            "media:manage",
            "admin:audit:*",
            "admin:news:*",
            "admin:notifications:*",
            "admin:policies:*",
            "admin:media:*",
        ] {
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
        for needle in [
            "payments:read",
            "payments:manage",
            "wallets:manage",
            "admin:payments:*",
            "admin:wallets:*",
        ] {
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
        // union of both perm sets, with duplicates removed. We
        // don't assert "admin ⊇ editor" because the editor role
        // now contributes 3-seg `admin:audit:*` style wildcards
        // that the admin role does not emit (admin emits 2-seg
        // `admin:*` instead, which is functionally equivalent for
        // the 2-seg gates but doesn't literal-match the 3-seg
        // perm name). So the union is admin-only ∪ editor-only
        // minus literal duplicates.
        let perms = permissions_for_roles(&["admin".to_string(), "editor".to_string()]);
        assert!(has(&perms, "admin:*"));
        assert!(has(&perms, "audit:read"));
        // No duplicates: every perm appears at most once.
        let mut sorted: Vec<&String> = perms.iter().collect();
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
    fn returned_vec_is_deterministic_order() {
        // The function uses a BTreeSet internally, so output is sorted.
        // Pin that — SSR caching and snapshot tests rely on stable order.
        let perms = permissions_for_roles(&["admin".to_string()]);
        let mut sorted = perms.clone();
        sorted.sort();
        assert_eq!(perms, sorted, "permissions_for_roles must return sorted output");
    }
}
