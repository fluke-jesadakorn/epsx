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

use crate::auth::{JWT, User, jwt};
use crate::dom::values::UserId;
use crate::config::env::get_env_var;

/// HMAC256 JWT Claims for Admin Frontend (matches admin frontend token format)
#[derive(Debug, Serialize, Deserialize)]
struct AdminFrontendTokenClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub package_tier: String,
    pub firebase_uid: String,
    pub admin_modules: Option<Vec<String>>,
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
    pub role: String,
    pub permissions: Vec<String>,
    pub admin_modules: Vec<String>,
    pub package_tier: String,
}

/**
 * Modern Auth.js v5 JWT middleware
 * Replaces complex Casbin middleware with simple JWT validation
 */
pub async fn modern_jwt_auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path().to_string();
    
    // Skip auth for public endpoints
    if is_public_endpoint(&path) {
        return Ok(next.run(request).await);
    }

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

    // Try HMAC256 validation first (for admin frontend tokens), then RSA256 (for other tokens)
    let user = match validate_hmac_token(token) {
        Ok(user) => {
            info!("HMAC256 token validation successful for admin endpoint: {}", path);
            user
        }
        Err(hmac_error) => {
            // HMAC validation failed, try RSA validation
            match JWT.extract_user(token) {
                Ok(user) => {
                    info!("RSA256 token validation successful for endpoint: {}", path);
                    user
                }
                Err(jwt::Error::Expired) => {
                    warn!("Expired token for endpoint: {}", path);
                    return Err(StatusCode::UNAUTHORIZED.into_response());
                }
                Err(jwt::Error::Invalid(msg)) => {
                    error!("Token validation failed for endpoint {}: HMAC error: {}, RSA error: {}", path, hmac_error, msg);
                    return Err(StatusCode::UNAUTHORIZED.into_response());
                }
                Err(jwt::Error::MissingClaims(msg)) => {
                    warn!("Missing claims in token for endpoint {}: {}", path, msg);
                    return Err(StatusCode::UNAUTHORIZED.into_response());
                }
                Err(jwt::Error::InvalidSignature) => {
                    error!("Invalid token signature for endpoint {}: Both HMAC and RSA validation failed", path);
                    return Err(StatusCode::UNAUTHORIZED.into_response());
                }
                Err(jwt::Error::PermissionDenied) => {
                    warn!("Permission denied for endpoint: {}", path);
                    return Err(StatusCode::FORBIDDEN.into_response());
                }
                Err(jwt::Error::NotYetValid) => {
                    warn!("Token not yet valid for endpoint: {}", path);
                    return Err(StatusCode::UNAUTHORIZED.into_response());
                }
            }
        }
    };

    // Check admin access for admin endpoints
    if path.starts_with("/api/admin") || path.starts_with("/api/v1/admin") {
        if !JWT.validate_admin_endpoint(&user, &path) {
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

    // Check specific permission requirements
    if let Some(required_permission) = get_required_permission(&path) {
        if !JWT.has_permission(&user, &required_permission) {
            warn!(
                "Permission '{}' denied for user {} to endpoint: {}", 
                required_permission, user.email, path
            );
            return Err(StatusCode::FORBIDDEN.into_response());
        }
    }

    // Check package tier requirements
    if let Some(required_tier) = get_required_package_tier(&path) {
        if !JWT.has_package_tier(&user, &required_tier) {
            warn!(
                "Package tier '{}' required for user {} to access endpoint: {}", 
                required_tier, user.email, path
            );
            return Err(StatusCode::FORBIDDEN.into_response());
        }
    }

    // Add user to request extensions for use in handlers
    let user_email = user.email.clone();
    request.extensions_mut().insert(user);

    info!(
        "Authenticated user {} accessing endpoint: {}", 
        user_email, 
        path
    );

    Ok(next.run(request).await)
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

/**
 * Get required permission for specific endpoints
 * Replaces complex Casbin policy rules with simple mapping
 */
fn get_required_permission(path: &str) -> Option<String> {
    let permission_map = [
        ("/api/v1/analytics", "analytics:read"),
        ("/api/v1/stock", "stock:read"),
        ("/api/v1/user/profile", "user:read"),
        ("/api/v1/user/settings", "user:write"),
        ("/api/v1/payment", "payment:read"),
        ("/api/v1/modules", "modules:read"),
    ];

    for (endpoint_prefix, permission) in &permission_map {
        if path.starts_with(endpoint_prefix) {
            return Some(permission.to_string());
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

/// Simple authentication context for realtime handlers
#[derive(Debug, Clone)]
pub struct AuthCtx {
    pub user_id: UserId,
    pub email: String,
    pub package_tier: String,
    pub permissions: Vec<String>,
    pub admin_modules: Vec<String>,
}

impl From<User> for AuthCtx {
    fn from(user: User) -> Self {
        Self {
            user_id: UserId::from(user.id),
            email: user.email,
            package_tier: user.package_tier,
            permissions: user.permissions,
            admin_modules: user.admin_modules,
        }
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

        // Validate JWT and extract user
        match JWT.extract_user(token) {
            Ok(user) => Ok(AuthCtx::from(user)),
            Err(jwt::Error::Expired) => {
                warn!("Expired token for AuthCtx extraction");
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(jwt::Error::Invalid(msg)) => {
                warn!("Invalid token for AuthCtx extraction: {}", msg);
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(jwt::Error::MissingClaims(msg)) => {
                warn!("Missing claims in token for AuthCtx extraction: {}", msg);
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(jwt::Error::InvalidSignature) => {
                warn!("Invalid signature in token for AuthCtx extraction");
                Err(StatusCode::UNAUTHORIZED)
            }
            Err(jwt::Error::PermissionDenied) => {
                warn!("Permission denied for AuthCtx extraction");
                Err(StatusCode::FORBIDDEN)
            }
            Err(jwt::Error::NotYetValid) => {
                warn!("Token not yet valid for AuthCtx extraction");
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
        let sub = claims.sub.clone();
        return Ok(User {
            id: claims.sub,
            email: claims.email,
            name: Some(claims.name),
            permissions: claims.permissions,
            admin_modules: claims.admin_modules.unwrap_or_default(),
            package_tier: claims.package_tier,
            role: claims.role,
            firebase_uid: Some(sub),
        });
    }
    
    // Try OIDC token format if admin format failed
    let token_data = decode::<AccessTokenClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("HMAC JWT validation failed for both formats: {}", e))?;
    
    let claims = token_data.claims;
    let sub = claims.sub.clone();
    Ok(User {
        id: claims.sub,
        email: claims.email,
        name: None,
        permissions: claims.permissions,
        admin_modules: claims.admin_modules,
        package_tier: claims.package_tier,
        role: claims.role,
        firebase_uid: Some(sub),
    })
}