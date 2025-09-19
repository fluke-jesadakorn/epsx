// Unified Authentication Middleware
// Consolidates all authentication approaches into a single, performant middleware

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, warn, error, info};
use serde::{Serialize, Deserialize};
use chrono::Utc;

use crate::auth::jwt::{Service as JWTService, Claims, User};
use crate::auth::performance_optimizations::OptimizedPermissionService;
use crate::web::middleware::rate_limiter::{UnifiedRateLimiter, ClientId, RateLimitConfig};
use crate::infrastructure::cache::Cache;

/// Unified authentication result
#[derive(Debug, Clone)]
pub struct AuthenticationResult {
    pub user: User,
    pub claims: Claims,
    pub permissions: Vec<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub authentication_time: chrono::DateTime<chrono::Utc>,
}

impl AuthenticationResult {
    /// Check if user has specific permission
    pub fn has_permission(&self, required_permission: &str) -> bool {
        crate::auth::permissions::check_permission_access(&self.permissions, required_permission)
    }
    
    /// Check if user has any of the specified permissions
    pub fn has_any_permission(&self, required_permissions: &[String]) -> bool {
        crate::auth::permissions::check_any_permission(&self.permissions, required_permissions)
    }
    
    /// Check if user has admin access
    pub fn is_admin(&self) -> bool {
        crate::auth::permissions::has_admin_access(&self.permissions)
    }
    
    /// Get user's effective ranking limit
    pub fn ranking_limit(&self) -> i32 {
        crate::auth::permissions::extract_ranking_limit(&self.permissions)
    }
}

/// Authentication middleware configuration
#[derive(Clone)]
pub struct AuthConfig {
    pub require_auth: bool,
    pub require_permissions: Vec<String>,
    pub rate_limit_config: Option<RateLimitConfig>,
    pub bypass_rate_limit: bool,
    pub log_auth_events: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            require_auth: true,
            require_permissions: Vec::new(),
            rate_limit_config: Some(RateLimitConfig::default()),
            bypass_rate_limit: false,
            log_auth_events: true,
        }
    }
}

/// Unified authentication error
#[derive(Debug, thiserror::Error)]
pub enum UnifiedAuthError {
    #[error("Authentication required")]
    AuthenticationRequired,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl UnifiedAuthError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            UnifiedAuthError::AuthenticationRequired => StatusCode::UNAUTHORIZED,
            UnifiedAuthError::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            UnifiedAuthError::PermissionDenied(_) => StatusCode::FORBIDDEN,
            UnifiedAuthError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            UnifiedAuthError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Unified authentication middleware state
#[derive(Clone)]
pub struct UnifiedAuthState {
    pub jwt_service: Arc<JWTService>,
    pub permission_service: Arc<OptimizedPermissionService>,
    pub rate_limiter: Arc<UnifiedRateLimiter>,
    pub cache: Arc<dyn Cache>,
}

/// Unified authentication middleware with comprehensive security and performance features
pub async fn unified_auth_middleware(
    State(auth_state): State<UnifiedAuthState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let request_path = request.uri().path().to_string();
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    
    // Default configuration (can be overridden by request extensions)
    let auth_config = request.extensions()
        .get::<AuthConfig>()
        .cloned()
        .unwrap_or_default();
    
    debug!(
        path = %request_path,
        client_ip = ?client_ip,
        require_auth = auth_config.require_auth,
        "Processing authentication request"
    );
    
    // Step 1: Rate limiting (if enabled)
    if !auth_config.bypass_rate_limit {
        if let Err(rate_limit_error) = check_rate_limits(
            &auth_state.rate_limiter,
            &client_ip,
            &request_path,
            &auth_config,
        ).await {
            warn!(
                path = %request_path,
                client_ip = ?client_ip,
                error = %rate_limit_error,
                "Rate limit exceeded"
            );
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }
    
    // Step 2: Authentication (if required)
    let auth_result = if auth_config.require_auth {
        match perform_authentication(&auth_state, &headers, &client_ip, &user_agent).await {
            Ok(result) => Some(result),
            Err(auth_error) => {
                warn!(
                    path = %request_path,
                    client_ip = ?client_ip,
                    error = %auth_error,
                    "Authentication failed"
                );
                return Err(auth_error.status_code());
            }
        }
    } else {
        None
    };
    
    // Step 3: Authorization (if permissions required)
    if let Some(ref auth_result) = auth_result {
        if !auth_config.require_permissions.is_empty() {
            if !auth_result.has_any_permission(&auth_config.require_permissions) {
                let error_msg = format!(
                    "Missing required permissions: {:?}",
                    auth_config.require_permissions
                );
                warn!(
                    path = %request_path,
                    user_id = %auth_result.user.id,
                    user_permissions = ?auth_result.permissions,
                    required_permissions = ?auth_config.require_permissions,
                    "Authorization failed"
                );
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }
    
    // Step 4: Add authentication result to request extensions
    if let Some(auth_result) = auth_result {
        if auth_config.log_auth_events {
            info!(
                path = %request_path,
                user_id = %auth_result.user.id,
                user_email = %auth_result.user.email,
                permissions_count = auth_result.permissions.len(),
                client_ip = ?client_ip,
                authentication_time_ms = start_time.elapsed().as_millis(),
                "Authentication successful"
            );
        }
        
        request.extensions_mut().insert(auth_result);
    }
    
    // Continue to next middleware/handler
    let response = next.run(request).await;
    
    // Log authentication metrics
    let total_time = start_time.elapsed();
    debug!(
        path = %request_path,
        processing_time_ms = total_time.as_millis(),
        "Authentication middleware completed"
    );
    
    Ok(response)
}

/// Perform comprehensive authentication
async fn perform_authentication(
    auth_state: &UnifiedAuthState,
    headers: &HeaderMap,
    client_ip: &Option<String>,
    user_agent: &Option<String>,
) -> Result<AuthenticationResult, UnifiedAuthError> {
    // Extract Bearer token
    let token = extract_bearer_token(headers)
        .ok_or_else(|| UnifiedAuthError::AuthenticationRequired)?;
    
    // Validate JWT token
    let claims = auth_state.jwt_service.verify(&token).await
        .map_err(|e| UnifiedAuthError::InvalidToken(e.to_string()))?;
    
    // Extract user information
    let user = User {
        id: claims.sub.clone(),
        email: claims.email.clone(),
        name: claims.name.clone(),
    };
    
    // Get fresh permissions from database/cache
    let user_id = uuid::Uuid::parse_str(&user.id)
        .map_err(|e| UnifiedAuthError::InternalError(format!("Invalid user ID format: {}", e)))?;
    
    let permissions = auth_state.permission_service.get_user_permissions(user_id).await
        .map_err(|e| UnifiedAuthError::InternalError(format!("Failed to fetch permissions: {}", e)))?;
    
    // Validate permissions are current (not expired)
    let valid_permissions = crate::auth::permissions::filter_valid_permissions(&permissions);
    
    Ok(AuthenticationResult {
        user,
        claims,
        permissions: valid_permissions,
        client_ip: client_ip.clone(),
        user_agent: user_agent.clone(),
        authentication_time: Utc::now(),
    })
}

/// Check rate limits for the request
async fn check_rate_limits(
    rate_limiter: &UnifiedRateLimiter,
    client_ip: &Option<String>,
    request_path: &str,
    auth_config: &AuthConfig,
) -> Result<(), UnifiedAuthError> {
    // IP-based rate limiting
    if let Some(ip) = client_ip {
        if let Err(_) = rate_limiter.check_ip_rate_limit(ip, request_path).await {
            return Err(UnifiedAuthError::RateLimitExceeded);
        }
    }
    
    // Custom rate limit configuration
    if let Some(ref rate_config) = auth_config.rate_limit_config {
        if let Some(ip) = client_ip {
            let client_id = ClientId::IpAddress(ip.clone());
            let result = rate_limiter.check_client_rate_limit(
                &client_id,
                request_path,
                "ANY",
                rate_config,
            ).await.map_err(|_| UnifiedAuthError::RateLimitExceeded)?;
            
            if !result.allowed {
                return Err(UnifiedAuthError::RateLimitExceeded);
            }
        }
    }
    
    Ok(())
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get(AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;
    
    if !auth_str.starts_with("Bearer ") {
        return None;
    }
    
    let token = auth_str.strip_prefix("Bearer ")?;
    if token.is_empty() {
        return None;
    }
    
    Some(token.to_string())
}

/// Extract client IP from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    let ip_headers = [
        "x-forwarded-for",
        "cf-connecting-ip", 
        "x-real-ip",
        "x-client-ip",
    ];
    
    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                let ip = ip_str.split(',').next().unwrap_or(ip_str).trim();
                if !ip.is_empty() {
                    return Some(ip.to_string());
                }
            }
        }
    }
    
    None
}

/// Extract User-Agent header
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers.get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Middleware factory for common authentication patterns
pub struct AuthMiddlewareFactory;

impl AuthMiddlewareFactory {
    /// Create middleware that requires authentication
    pub fn require_auth() -> AuthConfig {
        AuthConfig {
            require_auth: true,
            ..Default::default()
        }
    }
    
    /// Create middleware that requires specific permissions
    pub fn require_permissions(permissions: Vec<String>) -> AuthConfig {
        AuthConfig {
            require_auth: true,
            require_permissions: permissions,
            ..Default::default()
        }
    }
    
    /// Create middleware that requires admin access
    pub fn require_admin() -> AuthConfig {
        AuthConfig {
            require_auth: true,
            require_permissions: vec!["admin:*:*".to_string()],
            ..Default::default()
        }
    }
    
    /// Create middleware with custom rate limiting
    pub fn with_rate_limit(config: RateLimitConfig) -> AuthConfig {
        AuthConfig {
            require_auth: true,
            rate_limit_config: Some(config),
            ..Default::default()
        }
    }
    
    /// Create middleware for public endpoints (no auth required)
    pub fn public() -> AuthConfig {
        AuthConfig {
            require_auth: false,
            require_permissions: Vec::new(),
            rate_limit_config: Some(RateLimitConfig {
                requests_per_minute: Some(100),
                requests_per_hour: Some(1000),
                requests_per_day: Some(10000),
            }),
            ..Default::default()
        }
    }
}

/// Helper macros for extracting authentication results in handlers
#[macro_export]
macro_rules! require_unified_auth {
    ($request:expr) => {
        match $request.extensions().get::<crate::web::middleware::unified_auth::AuthenticationResult>() {
            Some(auth_result) => auth_result,
            None => return Err(axum::http::StatusCode::UNAUTHORIZED),
        }
    };
}

#[macro_export]
macro_rules! require_unified_permission {
    ($request:expr, $permission:expr) => {
        {
            let auth_result = $crate::require_unified_auth!($request);
            if !auth_result.has_permission($permission) {
                tracing::warn!(
                    user_id = %auth_result.user.id,
                    required_permission = $permission,
                    "Permission denied in handler"
                );
                return Err(axum::http::StatusCode::FORBIDDEN);
            }
            auth_result
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, Method};
    
    #[test]
    fn test_bearer_token_extraction() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer test_token_123"));
        
        let token = extract_bearer_token(&headers);
        assert_eq!(token, Some("test_token_123".to_string()));
        
        // Test invalid formats
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Basic invalid"));
        assert!(extract_bearer_token(&headers).is_none());
        
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer "));
        assert!(extract_bearer_token(&headers).is_none());
    }
    
    #[test]
    fn test_client_ip_extraction() {
        let mut headers = HeaderMap::new();
        
        // Test X-Forwarded-For
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.1, 70.41.3.18"));
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("203.0.113.1".to_string()));
        
        // Test CF-Connecting-IP
        headers.clear();
        headers.insert("cf-connecting-ip", HeaderValue::from_static("198.51.100.1"));
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("198.51.100.1".to_string()));
        
        // Test no headers
        headers.clear();
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, None);
    }
    
    #[test]
    fn test_auth_config_factory() {
        let admin_config = AuthMiddlewareFactory::require_admin();
        assert!(admin_config.require_auth);
        assert!(admin_config.require_permissions.contains(&"admin:*:*".to_string()));
        
        let public_config = AuthMiddlewareFactory::public();
        assert!(!public_config.require_auth);
        assert!(public_config.rate_limit_config.is_some());
        
        let custom_permissions = vec!["epsx:analytics:view".to_string()];
        let perm_config = AuthMiddlewareFactory::require_permissions(custom_permissions.clone());
        assert!(perm_config.require_auth);
        assert_eq!(perm_config.require_permissions, custom_permissions);
    }
    
    #[test]
    fn test_unified_auth_error_status_codes() {
        assert_eq!(
            UnifiedAuthError::AuthenticationRequired.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            UnifiedAuthError::InvalidToken("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            UnifiedAuthError::PermissionDenied("test".to_string()).status_code(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            UnifiedAuthError::RateLimitExceeded.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }
}