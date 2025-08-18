// Simplified auth modules
pub mod jwt;
pub mod flow;
pub mod tokens;
pub mod key_manager;
pub mod revocation;
pub mod refresh_tokens;
pub mod scopes;
pub mod cleanup;


// Clean exports - use new simplified modules
pub use jwt::{Service as JWTService, Claims, User, Error as JWTError, UserData};
pub use key_manager::KeyManager;
pub use flow::{AuthRequest, LoginForm, CodeData, Error as FlowError};
pub use tokens::{TokenRequest, TokenResponse, ErrorResponse as TokenError};
pub use revocation::{TokenRevocationService, RevokedToken, RevocationError, TOKEN_REVOCATION_SERVICE};
pub use refresh_tokens::{RefreshTokenService, RefreshTokenData, RefreshTokenRotation, RefreshTokenError, REFRESH_TOKEN_SERVICE};
pub use scopes::{ScopeService, Scope, ValidatedScopes, ScopeError, SCOPE_SERVICE};
pub use cleanup::{TokenCleanupService, CleanupConfig, CleanupResult, CleanupError, start_cleanup_service, manual_cleanup, get_cleanup_stats};

// Create simplified global JWT service
lazy_static::lazy_static! {
    pub static ref JWT: jwt::Service = jwt::Service::new()
        .expect("Failed to initialize JWT service");
}

