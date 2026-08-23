//! Frontend adapter — now a thin wrapper over `epsx-bff::TypedBffSession`.
//! BIG-BANG: deduplicates `apps/frontend/src/auth.rs` vs `apps/admin/src/auth.rs` (~400 LOC).
//! All logic lives in `shared/rust/bff/src/typed_session.rs`; this file keeps
//! the existing public API so `main.rs`/`ssr.rs`/`api.rs` need no changes.

use axum::http::HeaderMap;
use epsx_bff::{
    cookies::{CookieClient, CookieEnvironment},
    session::{AccessVerification, JwksVerifier, SessionUser},
    typed_session::TypedBffSession,
};
use epsx_dioxus_ui::auth::User;

#[allow(dead_code)]
pub fn access_token(headers: &HeaderMap, environment: CookieEnvironment) -> Option<String> {
    TypedBffSession::access_token_for(headers, environment, CookieClient::Frontend)
}

#[allow(dead_code)]
pub fn refresh_token(headers: &HeaderMap, environment: CookieEnvironment) -> Option<String> {
    TypedBffSession::refresh_token_for(headers, environment, CookieClient::Frontend)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub async fn access_verification(
    headers: &HeaderMap,
    verifier: &JwksVerifier,
    environment: CookieEnvironment,
) -> AccessVerification {
    verifier
        .verify_optional_access_token(access_token(headers, environment))
        .await
}

#[allow(dead_code)]
pub async fn current_user(
    headers: &HeaderMap,
    verifier: &JwksVerifier,
    environment: CookieEnvironment,
) -> Option<SessionUser> {
    verified_access_token(headers, verifier, environment)
        .await
        .map(|(_, user)| user)
}

#[allow(dead_code)]
pub fn ui_user(session: SessionUser, chain_id: Option<u64>) -> User {
    TypedBffSession::ui_user_static(session, chain_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header, HeaderValue};
    use epsx_dioxus_ui::auth::user::AuthMethod;

    #[test]
    fn canonical_cookie_precedes_legacy_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("epsx_token=legacy; epsx.frontend.access_token=canonical"),
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
            HeaderValue::from_static("epsx.frontend.refresh_token=opaque-refresh"),
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
            permissions: vec!["epsx:analytics:read".into(), "admin:users:manage".into()],
            capabilities: vec!["ranking-export".into()],
            auth_method: Some("web3_siwe".into()),
            created_at: None,
            last_login: Some("2026-07-22T00:00:00Z".into()),
        };

        let user = ui_user(session, Some(56));
        assert!(user.roles.is_empty());
        assert_eq!(
            user.permissions,
            vec!["epsx:analytics:read", "admin:users:manage"]
        );
        assert_eq!(user.chain_id, "56");
        assert_eq!(user.auth_method, AuthMethod::Siwe);
    }
}
