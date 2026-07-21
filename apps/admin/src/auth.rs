//! Admin adapter for the canonical RS256/JWKS browser session.

use axum::http::HeaderMap;
use epsx_bff::{
    cookies::{read_access_token, read_refresh_token, CookieEnvironment},
    session::{JwksVerifier, SessionUser},
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
        .or_else(|| read_access_token(headers, environment))
}

/// Refresh credentials are deliberately cookie-only. Browser JSON and request
/// bodies are never accepted as refresh-token sources.
pub fn refresh_token(headers: &HeaderMap, environment: CookieEnvironment) -> Option<String> {
    read_refresh_token(headers, environment)
}

/// Resolve and cryptographically verify the access token before it is used for
/// authorization or forwarded to any upstream service.
pub async fn verified_access_token(
    headers: &HeaderMap,
    verifier: &JwksVerifier,
    environment: CookieEnvironment,
) -> Option<(String, SessionUser)> {
    let token = access_token(headers, environment)?;
    let user = verifier.verify(&token).await.ok()?.session_user();
    Some((token, user))
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header, HeaderValue};

    #[test]
    fn canonical_cookie_precedes_legacy_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx_token=legacy; epsx.access_token=canonical"),
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
            HeaderValue::from_static("epsx.refresh_token=opaque-refresh"),
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
}
