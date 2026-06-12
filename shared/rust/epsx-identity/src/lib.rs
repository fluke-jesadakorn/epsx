//! `epsx-identity` — WAVE 9 / TRACK B STUB
//!
//! Real contents ship from `wave9/epsx-identity`. This stub
//! exists on the `wave9/epsx-web-middleware` branch ONLY so
//! the `epsx-web-middleware` path dep resolves.

#![doc = "Stub crate. Real surface ships from wave9/epsx-identity."]

/// Stub OpenID token service. The real
/// `epsx_identity::OpenIDTokenService` ships from Track B.
pub mod token_service {
    /// Stub error type.
    #[derive(Debug, thiserror::Error)]
    pub enum OpenIDTokenError {
        #[error("not implemented (stub)")]
        NotImplemented,
    }

    /// Stub claims. The real claims type ships from Track B
    /// with the OIDC standard fields.
    #[derive(Debug, Clone)]
    pub struct OidcClaims {
        pub sub: String,
        pub wallet_address: String,
        pub scope: String,
        pub auth_method: String,
        pub jti: String,
        pub exp: i64,
        pub iat: i64,
        pub auth_time: i64,
    }

    /// Stub service. The real one ships from Track B.
    pub struct OpenIDTokenService;

    impl OpenIDTokenService {
        /// Stub — the real `validate_access_token` is the
        /// single source of truth for JWT validation and is
        /// pure CPU (in-memory RSA public key, no DB, no
        /// Redis, ~0.05–0.2 ms per request).
        pub async fn validate_access_token(
            &self,
            _token: &str,
        ) -> Result<OidcClaims, OpenIDTokenError> {
            Err(OpenIDTokenError::NotImplemented)
        }
    }
}

/// Stub permission service. The real
/// `epsx_identity::UnifiedPermissionService` ships from Track B.
pub mod unified_permission_service {
    /// Stub service.
    pub struct UnifiedPermissionService;

    impl UnifiedPermissionService {
        /// Stub — real implementation ships from Track B.
        pub fn has_permission(_granted: &[String], _required: &str) -> bool {
            false
        }

        /// Stub — real implementation ships from Track B.
        pub fn is_admin(_granted: &[String]) -> bool {
            false
        }
    }
}

/// Stub unified web3 auth service. The real
/// `epsx_identity::UnifiedWeb3AuthService` ships from Track B.
pub mod auth_service {
    /// Stub service.
    pub struct UnifiedWeb3AuthService;

    impl UnifiedWeb3AuthService {
        /// Stub — real implementation ships from Track B.
        pub fn has_permission(_granted: &[String], _required: &str) -> bool {
            false
        }

        /// Stub — real implementation ships from Track B.
        pub fn is_admin(_granted: &[String]) -> bool {
            false
        }
    }
}
