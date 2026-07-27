//! Admin adapter for the canonical RS256/JWKS browser session.

use axum::http::HeaderMap;
use epsx_bff::{
    cookies::{read_access_token, read_refresh_token, CookieClient, CookieEnvironment},
    session::{AccessVerification, JwksVerifier, SessionUser},
};
use epsx_dioxus_ui::auth::{user::AuthMethod, User};

/// Authorization remains available for trusted server/API callers. Browser
/// sessions use the canonical HttpOnly cookie. The shared cookie helper accepts
/// the legacy access cookie read-only during migration.
pub fn access_token(headers: &HeaderMap, environment: CookieEnvironment) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .or_else(|| read_access_token(headers, environment, CookieClient::Admin))
}

/// Refresh credentials are deliberately cookie-only. Browser JSON and request
/// bodies are never accepted as refresh-token sources.
pub fn refresh_token(headers: &HeaderMap, environment: CookieEnvironment) -> Option<String> {
    read_refresh_token(headers, environment, CookieClient::Admin)
}

/// Resolve and cryptographically verify the access token before it is used for
/// authorization or forwarded to any upstream service.
pub async fn verified_access_token(
    headers: &HeaderMap,
    verifier: &JwksVerifier,
    environment: CookieEnvironment,
) -> Option<(String, SessionUser)> {
    match access_verification(headers, verifier, environment).await {
        AccessVerification::Verified { token, user } => Some((token, user)),
        AccessVerification::MissingOrRejected | AccessVerification::VerifierUnavailable => None,
    }
}

/// Preserve verifier outages as a distinct SSR outcome so an unavailable
/// JWKS authority cannot trigger a rotate/reload loop.
pub async fn access_verification(
    headers: &HeaderMap,
    verifier: &JwksVerifier,
    environment: CookieEnvironment,
) -> AccessVerification {
    verifier
        .verify_optional_access_token(access_token(headers, environment))
        .await
}

pub async fn current_user(
    headers: &HeaderMap,
    verifier: &JwksVerifier,
    environment: CookieEnvironment,
) -> Option<SessionUser> {
    verified_access_token(headers, verifier, environment)
        .await
        .map(|(_, user)| user)
}

/// Map only backend-issued identity fields. Roles stay empty and permissions
/// remain verbatim; authorization policy belongs to the Rust backend.
pub fn ui_user(session: SessionUser, chain_id: Option<u64>) -> User {
    let auth_method = match session.auth_method.as_deref() {
        Some("web3_siwe") | Some("siwe") => AuthMethod::Siwe,
        Some("wallet") => AuthMethod::Wallet,
        Some("email") => AuthMethod::Email,
        Some("oauth") => AuthMethod::OAuth,
        Some("demo") => AuthMethod::Demo,
        _ => AuthMethod::Unknown,
    };

    User {
        id: session.subject,
        address: session.wallet_address,
        chain_id: chain_id.map(|value| value.to_string()).unwrap_or_default(),
        roles: Vec::new(),
        email: None,
        tier: None,
        permissions: session.permissions,
        last_login_at: session.last_login,
        auth_method,
        display_name: None,
    }
}

/// Local-only visual-test identity. The dev bypass is intentionally mapped to
/// UI state instead of `AccessVerification`, so no synthetic token can reach
/// an upstream service and all page data still renders as unavailable.
pub fn dev_bypass_ui_user(chain_id: Option<u64>) -> Option<User> {
    if epsx_bff::dev_bypass::is_dev_force_unauth_enabled() {
        return None;
    }
    let session = epsx_bff::dev_bypass::dev_bypass_user()?;
    Some(User {
        id: session.user_id,
        address: session.address,
        chain_id: chain_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "56".to_string()),
        roles: session.roles,
        email: Some("dev-bypass@epsx.local".to_string()),
        tier: Some("Design preview".to_string()),
        permissions: vec!["*:*:*".to_string()],
        last_login_at: None,
        auth_method: AuthMethod::Demo,
        display_name: Some("EPSX Design Preview".to_string()),
    })
}

/// Map the local `?__design_bypass=1` capture identity into UI-only state.
/// It never creates a bearer token or authorizes an upstream request; admin
/// data remains governed by the normal backend-owned outcome paths.
pub fn design_bypass_ui_user(enabled: bool, chain_id: Option<u64>) -> Option<User> {
    let session = epsx_bff::dev_bypass::design_bypass_user(enabled)?;
    Some(User {
        id: session.user_id,
        address: session.address,
        chain_id: chain_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "56".to_string()),
        roles: session.roles,
        email: Some("dev-bypass@epsx.local".to_string()),
        tier: Some("Design preview".to_string()),
        permissions: vec!["*:*:*".to_string()],
        last_login_at: None,
        auth_method: AuthMethod::Demo,
        display_name: Some("EPSX Design Preview".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header, HeaderValue};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn canonical_cookie_precedes_legacy_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx_token=legacy; epsx.admin.access_token=canonical"),
        );
        assert_eq!(
            access_token(&headers, CookieEnvironment::Local).as_deref(),
            Some("canonical")
        );
    }

    #[test]
    fn refresh_cookie_is_never_an_access_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx.admin.refresh_token=opaque-refresh"),
        );
        assert!(access_token(&headers, CookieEnvironment::Local).is_none());
        assert_eq!(
            refresh_token(&headers, CookieEnvironment::Local).as_deref(),
            Some("opaque-refresh")
        );
    }

    #[test]
    fn ui_mapping_preserves_backend_permissions_without_roles() {
        let session = SessionUser {
            subject: "0xabc".into(),
            wallet_address: "0xabc".into(),
            permissions: vec!["admin:users:manage".into()],
            capabilities: vec!["admin-console".into()],
            auth_method: Some("web3_siwe".into()),
            created_at: None,
            last_login: Some("2026-07-22T00:00:00Z".into()),
        };

        let user = ui_user(session, Some(56));
        assert!(user.roles.is_empty());
        assert_eq!(user.permissions, vec!["admin:users:manage"]);
        assert_eq!(user.chain_id, "56");
        assert_eq!(user.auth_method, AuthMethod::Siwe);
    }

    #[test]
    fn dev_bypass_identity_has_admin_role_for_local_visual_checks() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var(epsx_bff::dev_bypass::DEV_BYPASS_ENV);
        assert!(dev_bypass_ui_user(Some(56)).is_none());
        std::env::set_var(epsx_bff::dev_bypass::DEV_BYPASS_ENV, "1");
        let user = dev_bypass_ui_user(Some(56)).expect("dev bypass identity");
        assert!(user.is_admin());
        assert!(user.has_permission("admin:analytics:read"));
        assert_eq!(user.auth_method, AuthMethod::Demo);
        std::env::remove_var(epsx_bff::dev_bypass::DEV_BYPASS_ENV);
    }

    #[test]
    fn design_bypass_identity_is_available_only_when_requested() {
        assert!(design_bypass_ui_user(false, Some(56)).is_none());
        let user = design_bypass_ui_user(true, Some(56)).expect("design bypass identity");
        assert_eq!(user.id, "dev-bypass");
        assert_eq!(user.auth_method, AuthMethod::Demo);
        assert!(user.has_permission("any:local:visual-check"));
    }
}
