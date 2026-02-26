// ============================================================================
// OPENID BEARER TOKEN AUTHENTICATION MIDDLEWARE
// Standard OpenID Connect Bearer token validation for all API requests
// ============================================================================

//! CORE PRINCIPLES:
//! - Uses OpenIDTokenService::validate_access_token() as SINGLE SOURCE OF TRUTH
//! - User context extraction from validated tokens
//! - Unified error responses for authentication failures

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    auth::OpenIDTokenError,
    infrastructure::adapters::repositories::developer_portal::ApiKeyRepository,
    web::auth::AppState,
};

/// OpenID Bearer Token User Context
/// Extracted from validated JWT access tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenIDUserContext {
    /// Subject (wallet address)
    pub sub: String,
    /// Wallet address (primary identifier)
    pub wallet_address: String,
    /// User permissions from token scope claim
    pub permissions: Vec<String>,
    /// Authentication method
    pub auth_method: String,
    /// JWT ID (unique token identifier)
    pub jti: String,
    /// Token expiration timestamp
    pub exp: i64,
    /// Issued at timestamp
    pub iat: i64,
    /// Authentication time
    pub auth_time: i64,
}

/// Unified API Error Response
#[derive(Debug, Serialize)]
pub struct UnifiedErrorResponse {
    pub success: bool,
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: u16,
    pub message: String,
    pub reason: String,
}

impl UnifiedErrorResponse {
    /// Create a new error response
    pub fn new(code: u16, message: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            success: false,
            error: ErrorDetails {
                code,
                message: message.into(),
                reason: reason.into(),
            },
        }
    }

    /// Wrap in Json for Axum handler returns
    pub fn json(code: u16, message: impl Into<String>, reason: impl Into<String>) -> Json<Self> {
        Json(Self::new(code, message, reason))
    }
}

/// OpenID Bearer Token Authentication Middleware
/// Validates JWT Bearer tokens and extracts user context
pub async fn bearer_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<UnifiedErrorResponse>)> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    // Try to get token from Authorization header first
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            header.strip_prefix("Bearer ").unwrap_or("").to_string()
        }
        _ => {
            debug!("No Bearer token found in Authorization header");
            return Err(create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Bearer token required",
                "Authorization header with Bearer token required"
            ));
        }
    };

    if token.is_empty() {
        debug!("Empty Bearer token");
        return Err(create_auth_error(
            StatusCode::UNAUTHORIZED,
            "Invalid token format",
            "Bearer token cannot be empty"
        ));
    }

    // Try JWT first (fast, no DB), then fall back to API key validation
    let user_context = match validate_bearer_token(&token, &app_state).await {
        Ok(context) => context,
        Err(_) => {
            // JWT failed — try API key fallback (SHA-256 hash + DB lookup)
            match validate_api_key(&token, &app_state).await {
                Ok(context) => context,
                Err((status, err)) => return Err((status, err)),
            }
        }
    };

    debug!(
        "Bearer token validated successfully for user: {} (method: {})",
        user_context.wallet_address, user_context.auth_method
    );

    // Add user context to request extensions
    request.extensions_mut().insert(user_context);

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Validate Bearer JWT token and extract user context
/// 
/// Uses OpenIDTokenService::validate_access_token() as the SINGLE SOURCE OF TRUTH
/// for all JWT validation. This ensures consistent validation across the entire application.
pub async fn validate_bearer_token(
    token: &str,
    app_state: &AppState,
) -> Result<OpenIDUserContext, OpenIDTokenError> {
    // Get OpenID token service for validation
    let token_service = app_state
        .domain_container
        .get_token_service()
        .ok_or_else(|| {
            OpenIDTokenError::TokenGenerationFailed("Token service not available".to_string())
        })?;

    // Use the SINGLE SOURCE OF TRUTH for token validation
    let claims = token_service.validate_access_token(token).await?;

    // Parse permissions from OIDC standard scope claim
    // OIDC standard: scope is space-separated string like "openid profile epsx:analytics:read admin:users:manage"
    let permissions: Vec<String> = claims.scope
        .split_whitespace()
        .filter(|s| *s != "openid" && *s != "profile") // Remove standard OIDC scopes
        .map(|s| s.to_string())
        .collect();

    // Extract user context from claims
    let user_context = OpenIDUserContext {
        sub: claims.sub,
        wallet_address: claims.wallet_address,
        permissions,  // Parsed from OIDC scope claim
        auth_method: claims.auth_method,
        jti: claims.jti,
        exp: claims.exp,
        iat: claims.iat,
        auth_time: claims.auth_time,
    };

    debug!(
        "JWT token validated for user: {} (permissions: {})",
        user_context.wallet_address,
        user_context.permissions.len()
    );

    Ok(user_context)
}

/// Create standardized authentication error response
fn create_auth_error(
    status: StatusCode,
    message: &str,
    reason: &str,
) -> (StatusCode, Json<UnifiedErrorResponse>) {
    let error_response = UnifiedErrorResponse {
        success: false,
        error: ErrorDetails {
            code: status.as_u16(),
            message: message.to_string(),
            reason: reason.to_string(),
        },
    };

    (status, Json(error_response))
}



/// Validate a Bearer token as an API key (SHA-256 hash → DB lookup)
/// Returns OpenIDUserContext with auth_method = "api_key" on success
async fn validate_api_key(
    token: &str,
    app_state: &AppState,
) -> Result<OpenIDUserContext, (StatusCode, Json<UnifiedErrorResponse>)> {
    let repo = ApiKeyRepository::new(*app_state.db_pool);

    let api_key = match repo.validate_key(token).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            return Err(create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid token",
                "authentication_failed",
            ));
        }
        Err(_) => {
            return Err(create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Invalid token",
                "authentication_failed",
            ));
        }
    };

    // Check expiration
    if let Some(exp) = api_key.expires_at {
        if exp < chrono::Utc::now() {
            return Err(create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Token expired",
                "token_expired",
            ));
        }
    }

    // Check status (validate_key filters active, but guard against race)
    if !matches!(api_key.status, crate::domain::developer_portal::ApiKeyStatus::Active) {
        return Err(create_auth_error(
            StatusCode::UNAUTHORIZED,
            "Token revoked",
            "token_revoked",
        ));
    }

    // Merge selected_permissions + plan permissions
    let mut permissions = api_key.selected_permissions.clone();
    for plan in &api_key.permission_plans {
        // Plan permissions are loaded via the plan's features — use plan slug as permission prefix
        // The selected_permissions already contain the actual IAM strings
        let _ = plan; // Plans provide context but permissions are stored in selected_permissions
    }
    permissions.dedup();

    let now = chrono::Utc::now();
    let ctx = OpenIDUserContext {
        sub: api_key.wallet_address.clone(),
        wallet_address: api_key.wallet_address,
        permissions,
        auth_method: "api_key".to_string(),
        jti: api_key.id.to_string(),
        exp: api_key.expires_at.map(|e| e.timestamp()).unwrap_or(i64::MAX),
        iat: api_key.created_at.timestamp(),
        auth_time: now.timestamp(),
    };

    debug!(
        "API key validated for user: {} (key: {}, permissions: {})",
        ctx.wallet_address, api_key.key_prefix, ctx.permissions.len()
    );

    Ok(ctx)
}

/// Optional Bearer Token Middleware (for public/optional auth endpoints)
/// Does not fail if no token is present, but validates if token exists
pub async fn optional_bearer_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract and validate token, but don't fail if missing
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(header) = auth_header {
        if let Some(token) = header.strip_prefix("Bearer ") {
            if !token.is_empty() {
                // Try JWT first, then API key fallback
                match validate_bearer_token(token, &app_state).await {
                    Ok(ctx) => {
                        debug!("Optional auth: JWT validated for user: {}", ctx.wallet_address);
                        request.extensions_mut().insert(ctx);
                    }
                    Err(_) => {
                        // JWT failed — try API key
                        if let Ok(ctx) = validate_api_key(token, &app_state).await {
                            debug!("Optional auth: API key validated for user: {}", ctx.wallet_address);
                            request.extensions_mut().insert(ctx);
                        }
                    }
                }
            }
        }
    }

    next.run(request).await
}

/// Helper to extract user context from request
pub fn extract_user_context(request: &Request) -> Option<&OpenIDUserContext> {
    request.extensions().get::<OpenIDUserContext>()
}

/// Helper to require user context (for use in handlers)
pub fn require_user_context(request: &Request) -> Result<&OpenIDUserContext, (StatusCode, Json<UnifiedErrorResponse>)> {
    extract_user_context(request).ok_or_else(|| {
        create_auth_error(
            StatusCode::UNAUTHORIZED,
            "Authentication required",
            "Valid Bearer token required for this endpoint"
        )
    })
}

/// Helper to check if user has specific permission
/// Uses exact match + wildcard matching (platform:*:* and platform:resource:*)
pub fn check_user_permission(
    user_context: &OpenIDUserContext,
    required_permission: &str,
) -> bool {
    crate::core::permissions::has_permission(&user_context.permissions, required_permission)
}

/// Helper to create permission denied error
pub fn create_permission_denied_error(
    required_permission: &str,
) -> (StatusCode, Json<UnifiedErrorResponse>) {
    create_auth_error(
        StatusCode::FORBIDDEN,
        "Permission denied",
        &format!("Required permission: {}", required_permission)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_user_permission() {
        let user_context = OpenIDUserContext {
            sub: "0x123".to_string(),
            wallet_address: "0x123".to_string(),
            permissions: vec![
                "epsx:analytics:read".to_string(),
                "epsx:export:csv".to_string(),
            ],
            auth_method: "web3_siwe".to_string(),
            jti: "test".to_string(),
            exp: 0,
            iat: 0,
            auth_time: 0,
        };

        assert!(check_user_permission(&user_context, "epsx:analytics:read"));
        assert!(!check_user_permission(&user_context, "admin:users:manage"));
    }

    #[test]
    fn test_admin_wildcard_permission() {
        let admin_context = OpenIDUserContext {
            sub: "0x456".to_string(),
            wallet_address: "0x456".to_string(),
            permissions: vec!["admin:*:*".to_string()],
            auth_method: "web3_siwe".to_string(),
            jti: "test".to_string(),
            exp: 0,
            iat: 0,
            auth_time: 0,
        };

        // admin:*:* grants all admin permissions
        assert!(check_user_permission(&admin_context, "admin:users:manage"));
        assert!(check_user_permission(&admin_context, "admin:permissions:read"));
        // admin:*:* does NOT grant cross-platform permissions
        assert!(!check_user_permission(&admin_context, "epsx:analytics:read"));
    }

    #[test]
    fn test_resource_wildcard_permission() {
        let ctx = OpenIDUserContext {
            sub: "0x789".to_string(),
            wallet_address: "0x789".to_string(),
            permissions: vec!["admin:users:*".to_string()],
            auth_method: "web3_siwe".to_string(),
            jti: "test".to_string(),
            exp: 0,
            iat: 0,
            auth_time: 0,
        };

        assert!(check_user_permission(&ctx, "admin:users:read"));
        assert!(check_user_permission(&ctx, "admin:users:manage"));
        // admin:users:* does NOT grant admin:permissions:read
        assert!(!check_user_permission(&ctx, "admin:permissions:read"));
    }
}