//! Typed BFF session — generic over `CookieClient` to deduplicate frontend/admin auth.
//!
//! BIG-BANG Phase 4: replaces `apps/frontend/src/auth.rs` and `apps/admin/src/auth.rs`
//! (~400 LOC duplication). Single generic `TypedBffSession<C>` holds `JwksVerifier` +
//! `CookieClient` + `CookieEnvironment` and exposes `verified_access_token`,
//! `access_verification`, `current_user`, `ui_user`.

use axum::http::HeaderMap;
use epsx_dioxus_ui::auth::{user::AuthMethod, User};

use crate::{
    cookies::{read_access_token, read_refresh_token, CookieClient, CookieEnvironment},
    session::{AccessVerification, JwksVerifier, SessionUser},
};

/// Generic BFF session helper. `C` is `CookieClient::Frontend` or `Admin`.
#[derive(Clone)]
pub struct TypedBffSession {
    verifier: std::sync::Arc<JwksVerifier>,
    environment: CookieEnvironment,
    client: CookieClient,
}

impl TypedBffSession {
    pub fn new(
        verifier: std::sync::Arc<JwksVerifier>,
        environment: CookieEnvironment,
        client: CookieClient,
    ) -> Self {
        Self {
            verifier,
            environment,
            client,
        }
    }

    pub fn frontend(
        verifier: std::sync::Arc<JwksVerifier>,
        environment: CookieEnvironment,
    ) -> Self {
        Self::new(verifier, environment, CookieClient::Frontend)
    }

    pub fn admin(verifier: std::sync::Arc<JwksVerifier>, environment: CookieEnvironment) -> Self {
        Self::new(verifier, environment, CookieClient::Admin)
    }

    pub fn access_token(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .or_else(|| read_access_token(headers, self.environment, self.client))
    }

    pub fn refresh_token(&self, headers: &HeaderMap) -> Option<String> {
        read_refresh_token(headers, self.environment, self.client)
    }

    pub async fn verified_access_token(
        &self,
        headers: &HeaderMap,
    ) -> Option<(String, SessionUser)> {
        match self.access_verification(headers).await {
            AccessVerification::Verified { token, user } => Some((token, user)),
            AccessVerification::MissingOrRejected | AccessVerification::VerifierUnavailable => None,
        }
    }

    pub async fn access_verification(&self, headers: &HeaderMap) -> AccessVerification {
        self.verifier
            .verify_optional_access_token(self.access_token(headers))
            .await
    }

    pub async fn current_user(&self, headers: &HeaderMap) -> Option<SessionUser> {
        self.verified_access_token(headers).await.map(|(_, u)| u)
    }

    /// Map backend `SessionUser` to UI `User`. Roles stay empty; permissions verbatim.
    pub fn ui_user(&self, session: SessionUser, chain_id: Option<u64>) -> User {
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
            chain_id: chain_id.map(|v| v.to_string()).unwrap_or_default(),
            roles: Vec::new(),
            email: None,
            tier: None,
            permissions: session.permissions,
            last_login_at: session.last_login,
            auth_method,
            display_name: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionUser;

    #[test]
    fn ui_user_maps_permissions_without_roles() {
        // Directly test mapping logic without needing a live verifier
        let session = SessionUser {
            subject: "0xabc".into(),
            wallet_address: "0xabc".into(),
            permissions: vec!["epsx:analytics:read".into()],
            capabilities: vec![],
            auth_method: Some("web3_siwe".into()),
            created_at: None,
            last_login: None,
        };
        // Call associated logic via a dummy TypedBffSession using a fake fetcher
        let config = crate::session::JwksVerifierConfig::new(
            "https://api.epsx.test/.well-known/jwks.json",
            "https://api.epsx.test",
            "epsx-frontend",
            std::time::Duration::from_secs(60),
        )
        .unwrap();
        struct NoopFetcher;
        #[async_trait::async_trait]
        impl crate::session::JwksFetcher for NoopFetcher {
            async fn fetch(&self) -> Result<crate::session::Jwks, crate::session::SessionError> {
                Ok(crate::session::Jwks { keys: vec![] })
            }
        }
        let verifier = std::sync::Arc::new(crate::session::JwksVerifier::new(
            config,
            std::sync::Arc::new(NoopFetcher),
        ));
        let s = TypedBffSession::frontend(verifier, CookieEnvironment::Local);
        let user = s.ui_user(session, Some(56));
        assert!(user.roles.is_empty());
        assert_eq!(user.permissions, vec!["epsx:analytics:read"]);
        assert_eq!(user.chain_id, "56");
    }
}
