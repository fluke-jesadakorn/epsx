// Simplified auth modules
pub mod jwt;
pub mod flow;
pub mod tokens;
pub mod key_manager;
pub mod permissions; // New permission-only system
pub mod revocation;
pub mod refresh_tokens;
pub mod refresh_token_service;
pub mod scopes;
pub mod cleanup;
pub mod session_cleanup_service;
pub mod session_security_service;
pub mod roles; // TODO: Remove after migration complete


// Clean exports - use new simplified modules
pub use jwt::{Service as JWTService, Claims, User, Error as JWTError, UserData};
pub use key_manager::KeyManager;
pub use flow::{AuthRequest, LoginForm, CodeData, Error as FlowError};
pub use tokens::{TokenRequest, TokenResponse, ErrorResponse as TokenError};
pub use revocation::{TokenRevocationService, RevokedToken, RevocationError, TOKEN_REVOCATION_SERVICE};
pub use refresh_tokens::{RefreshTokenService as LegacyRefreshTokenService, RefreshTokenData, RefreshTokenRotation, RefreshTokenError, REFRESH_TOKEN_SERVICE};
pub use refresh_token_service::{RefreshTokenService, RefreshTokenConfig, DeviceInfo, CreateRefreshTokenRequest, RefreshTokenResponse};
pub use scopes::{ScopeService, Scope, ValidatedScopes, ScopeError, SCOPE_SERVICE};
pub use cleanup::{TokenCleanupService, CleanupConfig, CleanupResult, CleanupError, start_cleanup_service, manual_cleanup, get_cleanup_stats};
pub use session_cleanup_service::{SessionCleanupService, SessionCleanupConfig, CleanupStats, CleanupHealthStatus, init_global_cleanup_service, start_global_cleanup_service, run_manual_cleanup, get_cleanup_health};
pub use session_security_service::{SessionSecurityService, SessionSecurityConfig, DeviceFingerprint, GeoLocation, SecurityEvent, SecurityEventType, SecurityAnalysisResult, UserSessionInfo};
pub use roles::{SimpleUserClaims, check_feature_access, require_feature_async, require_admin_async, require_permission_async}; // Updated exports
pub use permissions::{Permission, UserClaims, check_permission_access, PermissionError, require_permission_pure, PermissionSets};

// Create simplified global JWT service
lazy_static::lazy_static! {
    pub static ref JWT: jwt::Service = jwt::Service::new()
        .expect("Failed to initialize JWT service");
}

