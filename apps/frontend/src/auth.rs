//! Frontend adapter for the canonical RS256/JWKS browser session.

// Development-baseline fixture anchor: EPSX_COOKIE_SECURE. Runtime cookie
// security now fails closed through the typed `CookieEnvironment` contract.

use axum::http::HeaderMap;
use epsx_bff::{
    cookies::{read_access_token, read_refresh_token, CookieClient, CookieEnvironment},
    session::{AccessVerification, JwksVerifier, SessionUser},
};
use epsx_dioxus_ui::auth::{user::AuthMethod, wallet_button::ConnectedWalletState, User};

/// Authorization remains available for server/API callers. Browser sessions use
/// the canonical HttpOnly access cookie, with `epsx_token` accepted read-only
/// during migration by the shared cookie helper.
pub fn access_token(headers: &HeaderMap, environment: CookieEnvironment) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .or_else(|| read_access_token(headers, environment, CookieClient::Frontend))
}

pub fn refresh_token(headers: &HeaderMap, environment: CookieEnvironment) -> Option<String> {
    read_refresh_token(headers, environment, CookieClient::Frontend)
}

/// Resolve and cryptographically verify the browser access token before it is
/// used for authorization or forwarded to an upstream service.
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

/// Convert backend-issued identity data into the UI model. Roles intentionally
/// remain empty: the BFF must not reverse-engineer roles or permissions.
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

/// Build the local visual-test identity when the explicitly opt-in dev
/// bypass is enabled. This is deliberately kept separate from
/// `AccessVerification`: the fixture must never be turned into a bearer
/// token or forwarded to an upstream service. It only lets SSR render the
/// authenticated shell while page data remains unavailable/truthful.
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
        // This wildcard belongs only to the local visual fixture. Production
        // users receive backend-issued permissions through `ui_user`.
        permissions: vec!["*:*:*".to_string()],
        last_login_at: None,
        auth_method: AuthMethod::Demo,
        display_name: Some("EPSX Design Preview".to_string()),
    })
}

/// Map the local `?__design_bypass=1` capture identity into UI-only state.
/// This never creates a bearer token or authorizes an upstream request; page
/// data still follows the normal unavailable/strict outcome paths.
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

/// Wallet state paired with [`dev_bypass_ui_user`]. It is only returned when
/// `EPSX_DEV_AUTH_BYPASS=1`; no production request can opt into this state.
pub fn dev_bypass_wallet_state() -> Option<ConnectedWalletState> {
    if epsx_bff::dev_bypass::is_dev_force_unauth_enabled() {
        return None;
    }
    epsx_bff::dev_bypass::dev_bypass_user().map(|session| ConnectedWalletState {
        address: Some(session.address),
        connector_id: Some("injected".to_string()),
        is_authenticated: true,
        chain_id: Some(56),
        role: Some("super_admin".to_string()),
        tier_level: Some("Design preview".to_string()),
        perm_count: 1,
        ..ConnectedWalletState::default()
    })
}

/// Wallet shell state paired with the local design-capture identity.
pub fn design_bypass_wallet_state(enabled: bool) -> Option<ConnectedWalletState> {
    epsx_bff::dev_bypass::design_bypass_user(enabled).map(|_| ConnectedWalletState {
        // Match the connected-wallet address shown in the supplied source
        // capture. This is presentation-only fixture state; the design
        // bypass never supplies credentials to an upstream service.
        address: Some("0xea6400000000000000000000000000000000E3dF".to_string()),
        connector_id: Some("injected".to_string()),
        is_authenticated: true,
        chain_id: Some(56),
        role: Some("super_admin".to_string()),
        tier_level: Some("Design preview".to_string()),
        perm_count: 1,
        ..ConnectedWalletState::default()
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

    #[test]
    fn dev_bypass_identity_is_opt_in() {
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

    #[test]
    fn design_bypass_wallet_matches_source_capture_address() {
        assert!(design_bypass_wallet_state(false).is_none());
        let wallet = design_bypass_wallet_state(true).expect("design bypass wallet");
        assert_eq!(
            wallet.address.as_deref(),
            Some("0xea6400000000000000000000000000000000E3dF")
        );
        assert_eq!(wallet.chain_id, Some(56));
    }
}
