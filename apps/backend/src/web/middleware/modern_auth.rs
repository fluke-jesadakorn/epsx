use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
// use tower::Service; // Not needed for simple middleware
use tracing::{info, warn, error};

use crate::auth::{JWT_SERVICE, AuthenticatedUser, JWTError};

/**
 * Modern Auth.js v5 JWT middleware
 * Replaces complex Casbin middleware with simple JWT validation
 */
pub async fn modern_jwt_auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
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
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Validate JWT and extract user
    let user = match JWT_SERVICE.extract_user(token) {
        Ok(user) => user,
        Err(JWTError::Expired) => {
            warn!("Expired token for endpoint: {}", path);
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(JWTError::InvalidToken(msg)) => {
            warn!("Invalid token for endpoint {}: {}", path, msg);
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(JWTError::InvalidSignature) => {
            error!("Invalid token signature for endpoint: {}", path);
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(err) => {
            error!("JWT validation error for endpoint {}: {}", path, err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check admin access for admin endpoints
    if path.starts_with("/api/admin") || path.starts_with("/api/v1/admin") {
        if !JWT_SERVICE.validate_admin_endpoint(&user, &path) {
            warn!(
                "Admin access denied for user {} to endpoint: {}", 
                user.email, path
            );
            return Err(StatusCode::FORBIDDEN);
        }
        
        info!(
            "Admin access granted to user {} for endpoint: {}", 
            user.email, path
        );
    }

    // Check specific permission requirements
    if let Some(required_permission) = get_required_permission(&path) {
        if !JWT_SERVICE.has_permission(&user, &required_permission) {
            warn!(
                "Permission '{}' denied for user {} to endpoint: {}", 
                required_permission, user.email, path
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Check package tier requirements
    if let Some(required_tier) = get_required_package_tier(&path) {
        if !JWT_SERVICE.has_package_tier(&user, &required_tier) {
            warn!(
                "Package tier '{}' required for user {} to access endpoint: {}", 
                required_tier, user.email, path
            );
            return Err(StatusCode::FORBIDDEN);
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
) -> Result<Response, StatusCode> {
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
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let user_email = request
        .extensions()
        .get::<AuthenticatedUser>()
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