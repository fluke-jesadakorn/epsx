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
    http::{header::{AUTHORIZATION, COOKIE}, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::{
    auth::OpenIDTokenError,
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

    // Try to get token from Authorization header first, then fallback to cookies
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            header.strip_prefix("Bearer ").unwrap_or("").to_string()
        }
        _ => {
            // Fallback: Try to extract token from cookies
            // Cookie names: "epsx.access" (development) or "__Host-epsx.access" (production)
            match extract_token_from_cookie(&request) {
                Some(cookie_token) => {
                    debug!("No Authorization header, using token from cookie");
                    cookie_token
                }
                None => {
                    debug!("No Bearer token found in Authorization header or cookies");
                    return Err(create_auth_error(
                        StatusCode::UNAUTHORIZED,
                        "Bearer token required",
                        "Authorization header or valid session cookie required"
                    ));
                }
            }
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

    // Validate JWT token
    let user_context = match validate_bearer_token(&token, &app_state).await {
        Ok(context) => context,
        Err(err) => {
            warn!("Bearer token validation failed: {}", err);
            return Err(create_auth_error(
                StatusCode::UNAUTHORIZED,
                "Token validation failed",
                &format!("Invalid or expired token: {}", err)
            ));
        }
    };

    debug!(
        "Bearer token validated successfully for user: {}",
        user_context.wallet_address
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

/// Extract access token from cookie header
/// Supports multiple cookie names:
/// - Production: "__Host-epsx.access" (HttpOnly)
/// - Development: "epsx.access" (HttpOnly)  
/// - Client session: "epsx.client_session" or "__Host-epsx.client_session" (JS accessible)
/// - User cookie: "epsx.user" or "__Host-epsx.user" (JSON with `access` field - this is where frontend stores token!)
fn extract_token_from_cookie(request: &Request) -> Option<String> {
    let cookie_header = request.headers().get(COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;
    
    // Log all cookie names for debugging
    let cookie_names: Vec<&str> = cookie_str.split(';')
        .filter_map(|c| c.trim().split('=').next())
        .collect();
    debug!("Available cookies: {:?}", cookie_names);
    
    for cookie in cookie_str.split(';') {
        let cookie = cookie.trim();
        // Try production cookie names first, then development
        // HttpOnly access cookies
        if let Some(token) = cookie.strip_prefix("__Host-epsx.access=") {
            if !token.is_empty() {
                debug!("Found token in __Host-epsx.access cookie");
                return url_decode_token(token);
            }
        }
        if let Some(token) = cookie.strip_prefix("epsx.access=") {
            if !token.is_empty() {
                debug!("Found token in epsx.access cookie");
                return url_decode_token(token);
            }
        }
        // Client session cookies (frontend JavaScript-accessible fallback)
        if let Some(token) = cookie.strip_prefix("__Host-epsx.client_session=") {
            if !token.is_empty() {
                debug!("Found token in __Host-epsx.client_session cookie");
                return url_decode_token(token);
            }
        }
        if let Some(token) = cookie.strip_prefix("epsx.client_session=") {
            if !token.is_empty() {
                debug!("Found token in epsx.client_session cookie");
                return url_decode_token(token);
            }
        }
        // User JSON cookie - THIS IS WHERE FRONTEND ACTUALLY STORES THE TOKEN
        // Format: {"sub":"0x...", "wallet_address":"0x...", "access":"eyJ...JWT..."}
        if let Some(user_json) = cookie.strip_prefix("__Host-epsx.user=") {
            if let Some(token) = extract_token_from_user_json(user_json) {
                debug!("Found token in __Host-epsx.user cookie's access field");
                return Some(token);
            }
        }
        if let Some(user_json) = cookie.strip_prefix("epsx.user=") {
            if let Some(token) = extract_token_from_user_json(user_json) {
                debug!("Found token in epsx.user cookie's access field");
                return Some(token);
            }
        }
    }
    debug!("No access token found in cookies. Checked: epsx.access, __Host-epsx.access, epsx.client_session, __Host-epsx.client_session, epsx.user.access");
    None
}

/// Extract the `access` token from URL-decoded user JSON cookie
fn extract_token_from_user_json(encoded_json: &str) -> Option<String> {
    // URL decode the JSON first
    let decoded = url_decode_token(encoded_json)?;
    
    // Parse as JSON and extract the "access" field
    match serde_json::from_str::<serde_json::Value>(&decoded) {
        Ok(value) => {
            if let Some(access) = value.get("access").and_then(|v| v.as_str()) {
                if !access.is_empty() && access.starts_with("eyJ") {
                    // Looks like a JWT (starts with base64-encoded JSON header)
                    return Some(access.to_string());
                }
            }
            None
        }
        Err(e) => {
            debug!("Failed to parse epsx.user cookie as JSON: {}", e);
            None
        }
    }
}

/// URL decode a cookie token value (frontend may URL-encode the JWT)
/// Note: JWT tokens use base64url encoding which typically doesn't need URL decoding
fn url_decode_token(token: &str) -> Option<String> {
    // JWTs use base64url encoding which only has alphanumeric, hyphen, underscore, and period
    // These don't get percent-encoded, so we can just return the token as-is
    // Only decode if we see percent signs
    if token.contains('%') {
        // Simple percent decoding for common cases
        let mut result = String::with_capacity(token.len());
        let mut chars = token.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            } else {
                result.push(c);
            }
        }
        Some(result)
    } else {
        Some(token.to_string())
    }
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
                // Try to validate token
                match validate_bearer_token(token, &app_state).await {
                    Ok(user_context) => {
                        debug!(
                            "Optional auth: Bearer token validated for user: {}",
                            user_context.wallet_address
                        );
                        request.extensions_mut().insert(user_context);
                    }
                    Err(err) => {
                        debug!("Optional auth: Token validation failed: {}", err);
                        // Don't fail the request, just continue without user context
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
pub fn check_user_permission(
    user_context: &OpenIDUserContext,
    required_permission: &str,
) -> bool {
    // Check exact permission match
    if user_context.permissions.contains(&required_permission.to_string()) {
        return true;
    }

    // Check admin wildcard permissions
    if user_context.permissions.iter().any(|p| p.starts_with("admin:")) {
        return true;
    }

    // Check platform wildcard permissions
    let parts: Vec<&str> = required_permission.split(':').collect();
    if parts.len() >= 2 {
        let wildcard = format!("{}:*", parts[0]);
        if user_context.permissions.contains(&wildcard) {
            return true;
        }
    }

    false
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

        assert!(check_user_permission(&admin_context, "admin:users:manage"));
        assert!(check_user_permission(&admin_context, "epsx:analytics:read"));
    }
}