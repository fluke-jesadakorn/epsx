use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
};
// use tower::Service; // Not needed for simple middleware
use tracing::{info, warn, error};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

use crate::auth::{JWT, User};
use crate::auth::jwt::{
    CROSS_PLATFORM_PERMISSION_SERVICE,
    derive_package_tier_from_permissions,
    derive_accessible_platforms_from_permissions,
    derive_primary_platform_from_permissions
};
use crate::dom::values::UserId;
use crate::config::env::get_env_var;

/// HMAC256 JWT Claims for Admin Frontend (matches admin frontend token format)
#[derive(Debug, Serialize, Deserialize)]
struct AdminFrontendTokenClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub iat: i64,
    pub exp: i64,
}

/// HMAC256 JWT Claims for OIDC tokens (matches OIDC token format)
#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaims {
    pub jti: String,
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub scope: String,
    pub email: String,
    pub permissions: Vec<String>,
}

/**
 * Enhanced cross-platform JWT middleware with structured permissions
 * Supports Platform:Resource:Action permission format and platform context
 */
pub async fn cross_platform_auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path().to_string();
    
    // Skip auth for public endpoints
    if is_public_endpoint(&path) {
        return Ok(next.run(request).await);
    }

    // 🚨 DEVELOPMENT MODE BYPASS: Skip all authentication in development
    let rust_env = get_env_var("RUST_ENV").unwrap_or_default();
    if rust_env == "development" || rust_env.is_empty() {
        info!("🚨 Development mode (RUST_ENV='{}'): Bypassing all authentication for endpoint: {}", rust_env, path);
        
        // Create a mock user for development
        let dev_user = User {
            id: "dev-user-admin@epsx.io".to_string(),
            email: "admin@epsx.io".to_string(),
            name: Some("Development Admin".to_string()),
        };
        
        // Add user to request extensions for handlers
        request.extensions_mut().insert(dev_user);
        
        // Add platform context for admin routes
        if path.starts_with("/api/admin") || path.starts_with("/api/v1/admin") {
            request.extensions_mut().insert(PlatformContext { 
                platform: "admin".to_string() 
            });
        }
        
        return Ok(next.run(request).await);
    }

    // Extract platform context from request
    let platform_context = extract_platform_context(&request);

    // Extract JWT token from Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    let token = match auth_header {
        Some(token) => token,
        None => {
            warn!("No authorization header found for protected endpoint: {}", path);
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }
    };

    // Try RSA256 validation first (gets user and permissions together)
    let (user, permissions) = match JWT.decode_with_permissions(token).await {
        Ok((user, permissions)) => {
            info!("RSA256 token validation successful for endpoint: {}", path);
            (user, permissions)
        }
        Err(crate::auth::jwt::Error::Expired) => {
            warn!("Expired token for endpoint: {}", path);
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }
        Err(crate::auth::jwt::Error::Invalid(_)) => {
            // RSA validation failed, try HMAC validation (for admin frontend tokens)
            match validate_hmac_token(token) {
                Ok(user) => {
                    info!("HMAC256 token validation successful for endpoint: {}", path);
                    // For HMAC tokens, we need to get permissions from the token claims directly
                    let claims = JWT.verify(token).await.map_err(|_| StatusCode::UNAUTHORIZED.into_response())?;
                    (user, claims.permissions)
                }
                Err(hmac_error) => {
                    error!("Token validation failed for endpoint {}: Both RSA and HMAC validation failed: {}", path, hmac_error);
                    return Err(StatusCode::UNAUTHORIZED.into_response());
                }
            }
        }
        Err(crate::auth::jwt::Error::MissingClaims(msg)) => {
            warn!("Missing claims in token for endpoint {}: {}", path, msg);
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }
        Err(crate::auth::jwt::Error::InvalidSignature) => {
            error!("Invalid token signature for endpoint: {}", path);
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }
        Err(crate::auth::jwt::Error::PermissionDenied) => {
            warn!("Permission denied for endpoint: {}", path);
            return Err(StatusCode::FORBIDDEN.into_response());
        }
        Err(crate::auth::jwt::Error::NotYetValid) => {
            warn!("Token not yet valid for endpoint: {}", path);
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }
        Err(crate::auth::jwt::Error::Revoked) => {
            warn!("Revoked token used for endpoint: {}", path);
            return Err(StatusCode::UNAUTHORIZED.into_response());
        }
    };

    // 🔥 PLATFORM ACCESS VALIDATION
    if let Some(platform) = &platform_context {
        if !CROSS_PLATFORM_PERMISSION_SERVICE.can_access_platform_with_permissions(&permissions, platform) {
            warn!(
                "Platform access denied for user {} to platform '{}' at endpoint: {}", 
                user.email, platform, path
            );
            return Err((StatusCode::FORBIDDEN, 
                format!("Access denied to platform: {}", platform)).into_response());
        }
        
        info!(
            "Platform access granted for user {} to platform '{}' at endpoint: {}", 
            user.email, platform, path
        );
    }

    // Check admin access for admin endpoints
    if path.starts_with("/api/admin") || path.starts_with("/api/v1/admin") {
        // Development bypass: Allow admin access in development mode
        if get_env_var("RUST_ENV").unwrap_or_default() == "development" {
            info!(
                "Development mode: Bypassing admin endpoint validation for user {} accessing: {}", 
                user.email, path
            );
        } else if !JWT.validate_admin_endpoint(&user, &path) {
            warn!(
                "Admin access denied for user {} to endpoint: {}", 
                user.email, path
            );
            return Err(StatusCode::FORBIDDEN.into_response());
        }
        
        info!(
            "Admin access granted to user {} for endpoint: {}", 
            user.email, path
        );
    }

    // 🔥 STRUCTURED PERMISSION VALIDATION (Legacy support removed)
    if let Some(structured_permission) = get_structured_permission(&path, &platform_context) {
        if let Some((platform, resource, action)) = CROSS_PLATFORM_PERMISSION_SERVICE.parse_permission(&structured_permission) {
            // Development bypass: Allow structured permissions in development mode
            if get_env_var("RUST_ENV").unwrap_or_default() == "development" {
                info!(
                    "Development mode: Bypassing structured permission '{}' validation for user {} at endpoint: {}", 
                    structured_permission, user.email, path
                );
            } else if !CROSS_PLATFORM_PERMISSION_SERVICE.validate_platform_permission(&user, &platform, &resource, &action) {
                warn!(
                    "Structured permission '{}' denied for user {} to endpoint: {}", 
                    structured_permission, user.email, path
                );
                return Err((StatusCode::FORBIDDEN, 
                    format!("Permission denied: {}", structured_permission)).into_response());
            }
            
            info!(
                "Structured permission '{}' granted for user {} at endpoint: {}", 
                structured_permission, user.email, path
            );
        }
    }

    // Check package tier requirements
    if let Some(required_tier) = get_required_package_tier(&path) {
        // Development bypass: Allow package tier access in development mode
        if get_env_var("RUST_ENV").unwrap_or_default() == "development" {
            info!(
                "Development mode: Bypassing package tier '{}' validation for user {} at endpoint: {}", 
                required_tier, user.email, path
            );
        } else if !JWT.has_package_tier_with_permissions(&permissions, &required_tier) {
            warn!(
                "Package tier '{}' required for user {} to access endpoint: {}", 
                required_tier, user.email, path
            );
            return Err(StatusCode::FORBIDDEN.into_response());
        }
    }

    // Add platform context to request extensions
    if let Some(platform) = platform_context {
        request.extensions_mut().insert(PlatformContext { platform });
    }

    // Add user to request extensions for use in handlers
    let user_email = user.email.clone();
    request.extensions_mut().insert(user);

    info!(
        "Cross-platform authentication successful for user {} accessing endpoint: {}", 
        user_email, 
        path
    );

    Ok(next.run(request).await)
}

/**
 * Legacy Modern Auth.js v5 JWT middleware (kept for backward compatibility)
 * Use cross_platform_auth_middleware for new implementations
 */
pub async fn modern_jwt_auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Delegate to cross-platform middleware for backward compatibility
    cross_platform_auth_middleware(request, next).await
}

/**
 * Check if endpoint is public (no auth required)
 */
fn is_public_endpoint(path: &str) -> bool {
    let public_endpoints = [
        "/health",
        "/api/health",
        "/api/v1/health",
        "/api/auth/upsert-user",
        "/api/v1/auth/user-claims",
        "/api/public",
        "/docs",
        "/swagger",
        "/favicon.ico",
        "/robots.txt",
        "/.well-known",
        "/oauth/jwks",                    // JWKS endpoint (public keys)
        "/oauth/authorize",               // OAuth authorization endpoint 
        "/oauth/token",                   // OAuth token endpoint
        "/oauth/userinfo",                // OAuth userinfo endpoint (needs Bearer token but handled internally)
        "/oauth/revoke",                  // OAuth token revocation endpoint
        "/oauth/introspect",              // OAuth token introspection endpoint
        "/oauth/.well-known",             // OAuth discovery documents
        "/firebase-auth",                 // Firebase auth page
    ];

    public_endpoints.iter().any(|&endpoint| path.starts_with(endpoint))
}

/// Extract platform context from request
fn extract_platform_context(request: &Request) -> Option<String> {
    // Method 1: X-Platform header
    if let Some(platform) = request.headers().get("X-Platform") {
        if let Ok(platform_str) = platform.to_str() {
            return Some(platform_str.to_string());
        }
    }
    
    // Method 2: Subdomain detection  
    if let Some(host) = request.headers().get("HOST") {
        if let Ok(host_str) = host.to_str() {
            if host_str.starts_with("pay.") || host_str.contains("epsx-pay") {
                return Some("epsx-pay".to_string());
            }
            if host_str.starts_with("token.") || host_str.contains("epsx-token") {
                return Some("epsx-token".to_string());
            }
        }
    }
    
    // Method 3: Path-based detection
    let path = request.uri().path();
    if path.starts_with("/api/pay/") || path.starts_with("/api/v1/pay/") {
        return Some("epsx-pay".to_string());
    }
    if path.starts_with("/api/token/") || path.starts_with("/api/v1/token/") {
        return Some("epsx-token".to_string());
    }
    
    // Default to EPSX platform
    Some("epsx".to_string())
}


/**
 * Get structured permission based on platform context and path
 */
fn get_structured_permission(path: &str, platform_context: &Option<String>) -> Option<String> {
    let platform = platform_context.as_deref().unwrap_or("epsx");
    
    let structured_permission_map = [
        // EPSX Platform
        ("/api/v1/analytics", ("epsx", "analytics", "read")),
        ("/api/v1/users", ("epsx", "users", "read")), 
        ("/api/admin/users", ("epsx", "users", "manage")),
        
        // EPSX Pay Platform
        ("/api/pay/transactions", ("epsx-pay", "transactions", "read")),
        ("/api/pay/wallets", ("epsx-pay", "wallets", "read")),
        ("/api/pay/admin/transactions", ("epsx-pay", "transactions", "manage")),
        ("/api/v1/pay/transactions", ("epsx-pay", "transactions", "read")),
        ("/api/v1/pay/wallets", ("epsx-pay", "wallets", "read")),
        
        // EPSX Token Platform  
        ("/api/token/governance", ("epsx-token", "governance", "vote")),
        ("/api/token/treasury", ("epsx-token", "treasury", "view")),
        ("/api/token/admin/proposals", ("epsx-token", "governance", "propose")),
        ("/api/v1/token/governance", ("epsx-token", "governance", "vote")),
        ("/api/v1/token/treasury", ("epsx-token", "treasury", "view")),
    ];
    
    for (path_pattern, (default_platform, resource, action)) in &structured_permission_map {
        if path.starts_with(path_pattern) {
            // Use detected platform or default platform from mapping
            let resolved_platform = if platform == "epsx" && *default_platform != "epsx" {
                *default_platform
            } else {
                platform
            };
            
            return Some(format!("{}:{}:{}", resolved_platform, resource, action));
        }
    }
    
    None
}

/**
 * Get required package tier for specific endpoints
 */
fn get_required_package_tier(path: &str) -> Option<String> {
    let tier_map = [
        ("/api/v1/analytics/premium", "BRONZE"),
        ("/api/v1/analytics/advanced", "SILVER"),
        ("/api/v1/enterprise", "ENTERPRISE"),
        ("/api/v1/modules/premium", "BRONZE"),
    ];

    for (endpoint_prefix, tier) in &tier_map {
        if path.starts_with(endpoint_prefix) {
            return Some(tier.to_string());
        }
    }

    None
}

/**
 * Optional: CORS middleware for Auth.js integration
 */
pub async fn cors_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let mut response = next.run(request).await;

    // Add CORS headers for Auth.js frontend integration
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert(
        "Access-Control-Allow-Methods", 
        "GET, POST, PUT, DELETE, OPTIONS, PATCH".parse().unwrap()
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization, X-Provider-Hint".parse().unwrap()
    );
    headers.insert("Access-Control-Max-Age", "86400".parse().unwrap());

    Ok(response)
}

/**
 * Request logging middleware
 */
pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let user_email = request
        .extensions()
        .get::<User>()
        .map(|user| user.email.clone())
        .unwrap_or_else(|| "anonymous".to_string());

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let elapsed = start.elapsed();

    info!(
        "{} {} - {} - {}ms - User: {}",
        method,
        path,
        response.status(),
        elapsed.as_millis(),
        user_email
    );

    Ok(response)
}

/// Platform context for cross-platform operations
#[derive(Debug, Clone)]
pub struct PlatformContext {
    pub platform: String,
}

/// Simple authentication context for realtime handlers with cross-platform support
#[derive(Debug, Clone)]
pub struct AuthCtx {
    pub user_id: UserId,
    pub email: String,
    pub package_tier: String,
    pub permissions: Vec<String>,
    // Cross-platform fields
    pub platforms: Vec<String>,
    pub primary_platform: String,
}

impl AuthCtx {
    /// Create AuthCtx from User with default permissions (permissions need to be fetched separately)
    pub fn from_user_without_permissions(user: User, permissions: &[String]) -> Self {
        Self {
            user_id: UserId::from(user.id),
            email: user.email,
            package_tier: derive_package_tier_from_permissions(permissions),
            permissions: permissions.to_vec(),
            
            // Cross-platform fields - derived from permissions
            platforms: derive_accessible_platforms_from_permissions(permissions),
            primary_platform: derive_primary_platform_from_permissions(permissions),
        }
    }
    
    /// Create AuthCtx from JWT token with full permission fetching
    pub async fn from_jwt_token(token: &str, permission_service: &crate::app::services::PermissionApplicationService) -> Result<Self, StatusCode> {
        use crate::auth::jwt::JWT;
        
        // Extract user from JWT
        let user = match JWT.extract_user(token).await {
            Ok(user) => user,
            Err(_) => return Err(StatusCode::UNAUTHORIZED),
        };
        
        // Fetch permissions from separate table using user.id (same as firebase_uid)
        let permissions = permission_service
            .get_user_permissions(&user.id)
            .await
            .unwrap_or_default();
            
        // Apply timestamp validation to filter expired permissions
        use crate::auth::permissions::filter_valid_permissions;
        let valid_permissions = filter_valid_permissions(&permissions);
        
        Ok(Self {
            user_id: UserId::from(user.id),
            email: user.email,
            package_tier: derive_package_tier_from_permissions(&valid_permissions),
            permissions: valid_permissions.clone(),
            
            // Cross-platform fields - derived from permissions
            platforms: derive_accessible_platforms_from_permissions(&valid_permissions),
            primary_platform: derive_primary_platform_from_permissions(&valid_permissions),
        })
    }
}

impl From<User> for AuthCtx {
    fn from(user: User) -> Self {
        // Default to empty permissions - should be populated separately
        Self::from_user_without_permissions(user, &[])
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthCtx
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract JWT token from Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .and_then(|header| {
                if header.starts_with("Bearer ") {
                    Some(&header[7..])
                } else {
                    None
                }
            });

        let token = match auth_header {
            Some(token) => token,
            None => {
                warn!("No authorization header found for AuthCtx extraction");
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

        // Validate JWT and extract user with permissions
        use crate::auth::jwt::JWT;
        
        match JWT.decode_with_permissions(token).await {
            Ok((user, permissions)) => {
                let package_tier = derive_package_tier_from_permissions(&permissions);
                let platforms = derive_accessible_platforms_from_permissions(&permissions);
                let primary_platform = derive_primary_platform_from_permissions(&permissions);
                
                Ok(AuthCtx {
                    user_id: UserId::from(user.id),
                    email: user.email,
                    package_tier,
                    permissions,
                    platforms,
                    primary_platform,
                        })
            },
            Err(crate::auth::jwt::Error::Expired) => {
                warn!("Expired token for AuthCtx extraction");
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(crate::auth::jwt::Error::Invalid(msg)) => {
                warn!("Invalid token for AuthCtx extraction: {}", msg);
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(crate::auth::jwt::Error::MissingClaims(msg)) => {
                warn!("Missing claims in token for AuthCtx extraction: {}", msg);
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(crate::auth::jwt::Error::InvalidSignature) => {
                warn!("Invalid signature in token for AuthCtx extraction");
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(crate::auth::jwt::Error::PermissionDenied) => {
                warn!("Permission denied for AuthCtx extraction");
                Err(StatusCode::FORBIDDEN)
            }
            Err(crate::auth::jwt::Error::NotYetValid) => {
                warn!("Token not yet valid for AuthCtx extraction");
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(crate::auth::jwt::Error::Revoked) => {
                warn!("Revoked token for AuthCtx extraction");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

/// Validate HMAC256 token (try both admin frontend and OIDC formats)
fn validate_hmac_token(token: &str) -> Result<User, Box<dyn std::error::Error + Send + Sync>> {
    let jwt_secret = get_env_var("NEXTAUTH_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    
    // Set validation parameters (be permissive for admin tokens)
    validation.validate_exp = true;
    validation.validate_aud = false;
    validation.required_spec_claims.remove("iss"); // Don't require issuer
    validation.required_spec_claims.remove("aud"); // Don't require audience
    
    // Try admin frontend token format first
    if let Ok(token_data) = decode::<AdminFrontendTokenClaims>(token, &decoding_key, &validation) {
        let claims = token_data.claims;
        let _sub = claims.sub.clone();
        return Ok(User {
            id: claims.sub,
            email: claims.email,
            name: Some(claims.name),
            // package_tier, firebase_uid, platforms, primary_platform removed - derived from permissions
            
        });
    }
    
    // Try OIDC token format if admin format failed
    let token_data = decode::<AccessTokenClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("HMAC JWT validation failed for both formats: {}", e))?;
    
    let claims = token_data.claims;
    let _sub = claims.sub.clone();
    Ok(User {
        id: claims.sub,
        email: claims.email,
        name: None,
        // package_tier, firebase_uid, platforms, primary_platform removed - derived from permissions
    })
}