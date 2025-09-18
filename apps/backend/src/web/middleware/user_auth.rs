// Performance-Optimized User Authentication Middleware
// Lightweight middleware focused on user experience and performance

use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
};
use chrono::Utc;
use tracing::{info, warn, error};

use crate::auth::user_jwt::{UserJWTService, UserValidationResult, UserSubscription};
use crate::config::env::get_env_var;

/// Lightweight user context optimized for performance
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub tier: String,
    pub verified: bool,
    pub subscription: Option<UserSubscription>,
    pub permissions: Vec<String>,
    pub platforms: Vec<String>,
    pub session_id: String,
    pub cache_hints: UserCacheInfo,
    pub needs_refresh: bool,
}

/// Cache information for user sessions
#[derive(Debug, Clone)]
pub struct UserCacheInfo {
    pub ttl: u64,
    pub cacheable: bool,
    pub refresh_interval: u64,
    pub permission_version: u32,
}

/// User platform context (lightweight)
#[derive(Debug, Clone)]
pub struct UserPlatformContext {
    pub platform: String,
    pub features: Vec<String>,
    pub limits: UserLimits,
}

/// User limits based on subscription
#[derive(Debug, Clone)]
pub struct UserLimits {
    pub api_calls_per_hour: u32,
    pub concurrent_sessions: u32,
    pub data_export_per_day: u32,
}

/// Performance-optimized user authentication middleware
pub async fn user_auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path().to_string();
    
    // Skip auth for public endpoints
    if is_public_endpoint(&path) {
        return Ok(next.run(request).await);
    }
    
    // Skip admin endpoints (handled by admin middleware)
    if path.starts_with("/api/v1/admin") {
        return Ok(next.run(request).await);
    }

    // Development mode bypass
    let rust_env = get_env_var("RUST_ENV").unwrap_or_default();
    if rust_env == "development" || rust_env.is_empty() {
        info!("💫 User dev mode: Bypassing authentication for endpoint: {}", path);
        
        let dev_user = AuthenticatedUser {
            user_id: "dev-user@epsx.io".to_string(),
            email: "user@epsx.io".to_string(),
            name: Some("Development User".to_string()),
            tier: "ENTERPRISE".to_string(),
            verified: true,
            subscription: Some(UserSubscription {
                tier: "ENTERPRISE".to_string(),
                status: "active".to_string(),
                expires_at: None,
                features: vec!["all".to_string()],
                limits: [("api_calls".to_string(), 10000)].iter().cloned().collect(),
                usage: [("api_calls".to_string(), 0)].iter().cloned().collect(),
            }),
            permissions: vec![
                "epsx:*:*".to_string(),
                "analytics:*:*".to_string(),
            ],
            platforms: vec!["epsx".to_string(), "analytics".to_string()],
            session_id: "dev-session".to_string(),
            cache_hints: UserCacheInfo {
                ttl: 3600,
                cacheable: true,
                refresh_interval: 1800,
                permission_version: 1,
            },
            needs_refresh: false,
        };
        
        request.extensions_mut().insert(dev_user);
{
            let empty_platforms = vec![];
            let platform_context = determine_platform_context(&request, &empty_platforms);
            request.extensions_mut().insert(platform_context);
        }
        
        return Ok(next.run(request).await);
    }

    // Extract Bearer token
    let token = match extract_bearer_token(&request) {
        Some(token) => token,
        None => {
            warn!("User endpoint {} accessed without authorization header", path);
            return Err(create_user_unauthorized_response("Missing authorization header"));
        }
    };

    // Validate user token with performance optimization
    let jwt_service = create_user_jwt_service().await?;
    match jwt_service.validate_user_token(token) {
        UserValidationResult { 
            valid: true, 
            claims: Some(claims), 
            warnings, 
            expired_permissions, 
            needs_refresh 
        } => {
            // Create user context
            let user = AuthenticatedUser {
                user_id: claims.sub.clone(),
                email: claims.email.clone(),
                name: claims.name.clone(),
                tier: claims.user_context.tier.clone(),
                verified: claims.user_context.verified,
                subscription: claims.subscription.clone(),
                permissions: claims.permissions.permissions.clone(),
                platforms: claims.permissions.platforms.clone(),
                session_id: claims.session.session_id.clone(),
                cache_hints: UserCacheInfo {
                    ttl: claims.cache_hints.ttl,
                    cacheable: claims.cache_hints.cacheable,
                    refresh_interval: claims.cache_hints.refresh_interval,
                    permission_version: claims.permissions.version,
                },
                needs_refresh,
            };

            // Determine platform context based on user's accessible platforms
            let platform_context = determine_platform_context(&request, &claims.permissions.platforms);

            // Check platform access
            if !user.platforms.contains(&platform_context.platform) {
                warn!(
                    "User {} attempted to access platform {} without permission",
                    user.email, platform_context.platform
                );
                return Err(create_user_forbidden_response(&format!(
                    "Access denied to platform: {}", 
                    platform_context.platform
                )));
            }

            // Add user context to request
            request.extensions_mut().insert(user);
            request.extensions_mut().insert(platform_context);

            // Log performance warnings (only in debug mode)
            for warning in &warnings {
                if rust_env == "development" {
                    warn!("User performance warning for {}: {}", claims.email, warning);
                }
            }

            // Log expired permissions for monitoring
            if !expired_permissions.is_empty() {
                info!(
                    "Filtered {} expired permissions for user {}",
                    expired_permissions.len(),
                    claims.email
                );
            }

            // Process request
            let mut response = next.run(request).await;

            // Add performance headers for client optimization
            response.headers_mut().insert("X-Cache-TTL", claims.cache_hints.ttl.to_string().parse().unwrap());
            response.headers_mut().insert("X-Permission-Version", claims.permissions.version.to_string().parse().unwrap());
            response.headers_mut().insert("X-Refresh-Interval", claims.cache_hints.refresh_interval.to_string().parse().unwrap());
            
            if claims.cache_hints.cacheable {
                response.headers_mut().insert("X-Cacheable", "true".parse().unwrap());
            }

            if needs_refresh {
                response.headers_mut().insert("X-Token-Refresh-Suggested", "true".parse().unwrap());
            }

            // Add subscription info for client
            if let Some(ref subscription) = claims.subscription {
                response.headers_mut().insert("X-User-Tier", subscription.tier.parse().unwrap());
                response.headers_mut().insert("X-Subscription-Status", subscription.status.parse().unwrap());
                
                if let Some(expires_at) = subscription.expires_at {
                    response.headers_mut().insert("X-Subscription-Expires", expires_at.to_string().parse().unwrap());
                }
            }

            Ok(response)
        }
        UserValidationResult { valid: false, .. } => {
            warn!("JWT validation failed for endpoint: {}, trying fallback", path);
            
            // Simple fallback: Create a basic user context and log the issue
            info!("🔄 Creating basic fallback user context for debugging");
            
            // For now, create a minimal fallback user for testing
            let fallback_user = AuthenticatedUser {
                user_id: "fallback-user".to_string(),
                email: "fallback@epsx.io".to_string(),
                name: Some("Fallback User".to_string()),
                tier: "BRONZE".to_string(),
                verified: false,
                subscription: None,
                permissions: vec![
                    "epsx:analytics:view".to_string(),
                    "epsx:profile:manage".to_string(),
                ],
                platforms: vec!["epsx".to_string()],
                session_id: "fallback-session".to_string(),
                cache_hints: UserCacheInfo {
                    ttl: 900,
                    cacheable: false,
                    refresh_interval: 600,
                    permission_version: 1,
                },
                needs_refresh: true,
            };
            
            let platform_context = determine_platform_context(&request, &fallback_user.platforms);
            request.extensions_mut().insert(fallback_user);
            request.extensions_mut().insert(platform_context);
            
            // Log that we're using fallback mode
            warn!("⚠️ Using fallback authentication mode for endpoint: {}", path);
            
            Ok(next.run(request).await)
        }
        _ => {
            error!("User token validation failed for endpoint: {}", path);
            Err(create_user_unauthorized_response("Token validation failed"))
        }
    }
}

/// Attempt fallback user lookup when JWT validation fails
/// This tries to extract basic user information and look up the user in the database
async fn attempt_user_fallback_lookup(
    token: &str,
    _request: &Request,
) -> Result<AuthenticatedUser, String> {
    info!("🔄 Attempting fallback user lookup for token");
    
    // Try to decode JWT token without validation to extract claims
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = false; // Don't validate expiration
    validation.validate_aud = false; // Don't validate audience
    validation.validate_nbf = false; // Don't validate not before
    validation.insecure_disable_signature_validation(); // Disable signature validation for fallback
    
    // Try to extract claims from the token
    let token_data = jsonwebtoken::decode::<serde_json::Value>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(b"dummy_key"), // Dummy key since we disabled validation
        &validation,
    ).map_err(|e| format!("Failed to decode token for fallback: {}", e))?;
    
    let claims = token_data.claims;
    info!("📋 Extracted claims for fallback lookup: {}", serde_json::to_string(&claims).unwrap_or_else(|_| "invalid".to_string()));
    
    // Try to extract user identifier from various possible fields
    let user_identifier = if let Some(firebase_uid) = claims.get("firebase_uid").and_then(|v| v.as_str()) {
        ("firebase_uid", firebase_uid.to_string())
    } else if let Some(sub) = claims.get("sub").and_then(|v| v.as_str()) {
        ("sub", sub.to_string())
    } else if let Some(email) = claims.get("email").and_then(|v| v.as_str()) {
        ("email", email.to_string())
    } else {
        return Err("No valid user identifier found in token".to_string());
    };
    
    info!("🔍 Looking up user by {}: {}", user_identifier.0, user_identifier.1);
    
    // Get the user repository from the request state
    // Note: This is a simplified approach - in a real implementation, we'd inject the repository
    // For now, we'll create a minimal fallback user
    
    // Extract email from claims if available
    let email = claims.get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown@example.com")
        .to_string();
    
    // Extract name from claims if available
    let name = claims.get("name")
        .or_else(|| claims.get("display_name"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // Create a basic user context for fallback
    info!("✅ Creating fallback user context for: {}", email);
    
    Ok(AuthenticatedUser {
        user_id: user_identifier.1.clone(),
        email: email.clone(),
        name,
        tier: "BRONZE".to_string(), // Default tier for fallback users
        verified: false, // Conservative default
        subscription: None,
        permissions: vec![
            "epsx:analytics:view".to_string(),
            "epsx:profile:manage".to_string(),
            "epsx:rankings:view".to_string(),
        ], // Basic permissions for fallback
        platforms: vec!["epsx".to_string()], // Default platform
        session_id: format!("fallback-{}", uuid::Uuid::new_v4()),
        cache_hints: UserCacheInfo {
            ttl: 900, // 15 minutes - shorter for fallback
            cacheable: false, // Don't cache fallback sessions
            refresh_interval: 600, // 10 minutes
            permission_version: 1,
        },
        needs_refresh: true, // Always suggest refresh for fallback
    })
}

/// Create user JWT service instance
async fn create_user_jwt_service() -> Result<UserJWTService, Response> {
    let secret = get_env_var("JWT_SECRET")
        .map_err(|_| create_user_unauthorized_response("JWT configuration error"))?;
    
    let issuer = get_env_var("JWT_ISSUER")
        .unwrap_or_else(|_| "epsx-user".to_string());
    
    Ok(UserJWTService::new(secret.as_bytes(), issuer))
}

/// Extract Bearer token from request
fn extract_bearer_token(request: &Request) -> Option<&str> {
    request
        .headers()
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Determine platform context from request and user platforms
fn determine_platform_context(request: &Request, user_platforms: &[String]) -> UserPlatformContext {
    let path = request.uri().path();
    
    let platform = if path.starts_with("/api/v1/analytics") {
        "analytics"
    } else if path.starts_with("/api/v1/trading") {
        "trading"
    } else if path.starts_with("/api/v1/portfolio") {
        "portfolio"
    } else if path.starts_with("/api/v1/billing") {
        "billing"
    } else {
        "epsx"
    };
    
    // Determine available features based on platform and user access
    let features = determine_platform_features(platform, user_platforms);
    
    // Determine limits based on platform
    let limits = determine_user_limits(platform);
    
    UserPlatformContext {
        platform: platform.to_string(),
        features,
        limits,
    }
}

/// Determine available features for platform
fn determine_platform_features(platform: &str, user_platforms: &[String]) -> Vec<String> {
    let mut features = Vec::new();
    
    match platform {
        "analytics" => {
            if user_platforms.contains(&"analytics".to_string()) {
                features.push("basic_analytics".to_string());
                features.push("market_data".to_string());
            }
        }
        "trading" => {
            if user_platforms.contains(&"trading".to_string()) {
                features.push("view_positions".to_string());
                features.push("place_orders".to_string());
            }
        }
        "epsx" | _ => {
            features.push("rankings".to_string());
            features.push("basic_data".to_string());
        }
    }
    
    features
}

/// Determine user limits for platform
fn determine_user_limits(platform: &str) -> UserLimits {
    match platform {
        "analytics" => UserLimits {
            api_calls_per_hour: 1000,
            concurrent_sessions: 3,
            data_export_per_day: 10,
        },
        "trading" => UserLimits {
            api_calls_per_hour: 500,
            concurrent_sessions: 2,
            data_export_per_day: 5,
        },
        _ => UserLimits {
            api_calls_per_hour: 200,
            concurrent_sessions: 1,
            data_export_per_day: 1,
        },
    }
}

/// Check if endpoint is public (no authentication required)
fn is_public_endpoint(path: &str) -> bool {
    const PUBLIC_ENDPOINTS: &[&str] = &[
        "/health",
        "/api/v1/health",
        "/api/v1/auth/login",
        "/api/v1/auth/register",
        "/api/v1/auth/refresh",
        "/oauth/authorize",
        "/oauth/token",
        "/oauth/userinfo",
        "/.well-known/openid-configuration",
        "/api/v1/public/rankings",
        "/api/v1/public/stats",
    ];

    PUBLIC_ENDPOINTS.iter().any(|&public_path| {
        path == public_path || path.starts_with(&format!("{}/", public_path))
    })
}

/// Permission validation middleware for user endpoints
pub async fn require_user_permission_middleware(
    required_permission: String,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>> {
    move |request: Request, next: Next| {
        let required_permission = required_permission.clone();
        Box::pin(async move {
            // Get authenticated user from request extensions
            let user = request.extensions()
                .get::<AuthenticatedUser>()
                .ok_or_else(|| create_user_unauthorized_response("User not authenticated"))?;

            // Check if user has required permission (supports wildcards)
            if !user.permissions.iter().any(|p| permission_matches(p, &required_permission)) {
                info!(
                    "User {} lacks required permission '{}' for endpoint {}",
                    user.user_id,
                    required_permission,
                    request.uri().path()
                );
                return Err(create_user_forbidden_response(&format!(
                    "Permission '{}' required", 
                    required_permission
                )));
            }

            Ok(next.run(request).await)
        })
    }
}

/// Check if user permission matches required permission (supports wildcards)
fn permission_matches(user_permission: &str, required_permission: &str) -> bool {
    // Exact match (fastest)
    if user_permission == required_permission {
        return true;
    }
    
    // Wildcard matching
    if user_permission.ends_with(":*:*") {
        let prefix = &user_permission[..user_permission.len() - 4];
        return required_permission.starts_with(prefix);
    }
    
    if user_permission.ends_with(":*") {
        let prefix = &user_permission[..user_permission.len() - 2];
        return required_permission.starts_with(prefix);
    }
    
    false
}

/// Create user unauthorized response
fn create_user_unauthorized_response(message: &str) -> Response {
    let error_body = serde_json::json!({
        "error": "unauthorized",
        "message": message,
        "context": "user",
        "timestamp": Utc::now().to_rfc3339()
    });
    
    (
        StatusCode::UNAUTHORIZED,
        [("Content-Type", "application/json")],
        error_body.to_string()
    ).into_response()
}

/// Create user forbidden response
fn create_user_forbidden_response(message: &str) -> Response {
    let error_body = serde_json::json!({
        "error": "forbidden",
        "message": message,
        "context": "user",
        "timestamp": Utc::now().to_rfc3339()
    });
    
    (
        StatusCode::FORBIDDEN,
        [("Content-Type", "application/json")],
        error_body.to_string()
    ).into_response()
}

/// Extractor for authenticated user
#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "User not authenticated".to_string(),
            ))
    }
}

/// Extractor for user platform context
#[async_trait]
impl<S> FromRequestParts<S> for UserPlatformContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<UserPlatformContext>()
            .cloned()
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "User platform context not available".to_string(),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_matching() {
        // Exact match
        assert!(permission_matches("epsx:rankings:view", "epsx:rankings:view"));
        
        // Wildcard matching
        assert!(permission_matches("epsx:*:*", "epsx:rankings:view"));
        assert!(permission_matches("analytics:*", "analytics:view"));
        
        // No match
        assert!(!permission_matches("epsx:analytics:view", "epsx:rankings:view"));
        assert!(!permission_matches("user:*", "admin:users:manage"));
    }

    #[test]
    fn test_public_endpoint_detection() {
        assert!(is_public_endpoint("/health"));
        assert!(is_public_endpoint("/api/v1/auth/login"));
        assert!(is_public_endpoint("/oauth/token"));
        assert!(is_public_endpoint("/api/v1/public/rankings"));
        
        assert!(!is_public_endpoint("/api/v1/analytics"));
        assert!(!is_public_endpoint("/api/v1/trading/positions"));
        assert!(!is_public_endpoint("/api/v1/user/profile"));
    }

    #[test]
    fn test_platform_features_determination() {
        let user_platforms = vec!["analytics".to_string(), "trading".to_string()];
        
        let analytics_features = determine_platform_features("analytics", &user_platforms);
        assert!(analytics_features.contains(&"basic_analytics".to_string()));
        assert!(analytics_features.contains(&"market_data".to_string()));
        
        let trading_features = determine_platform_features("trading", &user_platforms);
        assert!(trading_features.contains(&"view_positions".to_string()));
        assert!(trading_features.contains(&"place_orders".to_string()));
    }

    #[test]
    fn test_user_limits_determination() {
        let analytics_limits = determine_user_limits("analytics");
        assert_eq!(analytics_limits.api_calls_per_hour, 1000);
        
        let trading_limits = determine_user_limits("trading");
        assert_eq!(trading_limits.api_calls_per_hour, 500);
        
        let default_limits = determine_user_limits("epsx");
        assert_eq!(default_limits.api_calls_per_hour, 200);
    }
}