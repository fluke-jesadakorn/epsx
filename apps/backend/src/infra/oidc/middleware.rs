// OIDC Bearer Authentication Middleware
// Replaces all cookie-based authentication with standard Bearer tokens

use std::sync::Arc;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tracing::{info, warn, error};

use super::service::{OIDCService, OIDCClaims};

#[derive(Debug, Clone)]
pub struct BearerAuthState {
    pub user_id: String,
    pub email: Option<String>,
    pub permissions: Vec<String>,
    pub role: String,
    pub claims: OIDCClaims,
}

/// Bearer token authentication middleware
/// Validates Authorization: Bearer <token> headers and injects user context
pub async fn bearer_auth_middleware(
    oidc_service: Arc<OIDCService>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req.headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Validate Bearer token format
    if !auth_header.starts_with("Bearer ") {
        warn!("Invalid Authorization header format - expected Bearer token");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Validate token with OIDC service
    let claims = match oidc_service.validate_bearer_token(token).await {
        Ok(claims) => claims,
        Err(e) => {
            error!("Token validation failed: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Create authentication state
    let auth_state = BearerAuthState {
        user_id: claims.sub.clone(),
        email: claims.email.clone(),
        permissions: claims.permissions.clone().unwrap_or_default(),
        role: claims.role.clone().unwrap_or_else(|| "user".to_string()),
        claims: claims.clone(),
    };

    // Inject auth state into request extensions
    req.extensions_mut().insert(auth_state);

    info!("Authenticated user {} with {} permissions", 
          claims.sub, claims.permissions.as_ref().map_or(0, |p| p.len()));

    Ok(next.run(req).await)
}

/// Optional Bearer authentication (for public endpoints that can work with or without auth)
pub async fn optional_bearer_auth_middleware(
    oidc_service: Arc<OIDCService>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try to extract Authorization header
    if let Some(auth_header) = req.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                
                // Try to validate token, but don't fail if invalid
                if let Ok(claims) = oidc_service.validate_bearer_token(token).await {
                    let user_id = claims.sub.clone();
                    let auth_state = BearerAuthState {
                        user_id: claims.sub.clone(),
                        email: claims.email.clone(),
                        permissions: claims.permissions.clone().unwrap_or_default(),
                        role: claims.role.clone().unwrap_or_else(|| "user".to_string()),
                        claims,
                    };
                    
                    req.extensions_mut().insert(auth_state);
                    info!("Optional authentication successful for user: {}", user_id);
                } else {
                    warn!("Optional authentication failed - continuing without auth");
                }
            }
        }
    }

    Ok(next.run(req).await)
}

/// Permission-based authorization middleware
/// Requires specific permissions to access the endpoint
pub async fn require_permission_middleware(
    required_permission: &'static str,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    let required_permission = required_permission.to_string();
    
    move |req: Request, next: Next| {
        let required_permission = required_permission.clone();
        Box::pin(async move {
            // Get auth state from request extensions
            let auth_state = req.extensions()
                .get::<BearerAuthState>()
                .ok_or_else(|| {
                    warn!("Permission check failed - no authentication state found");
                    StatusCode::UNAUTHORIZED
                })?;

            // Check if user has required permission
            if !has_permission(&auth_state.permissions, &required_permission) {
                warn!("Permission denied: user {} missing permission {}", 
                      auth_state.user_id, required_permission);
                return Err(StatusCode::FORBIDDEN);
            }

            info!("Permission granted: {} for user {}", required_permission, auth_state.user_id);
            Ok(next.run(req).await)
        })
    }
}

/// Check if user has a specific permission
/// Supports wildcards: admin:*:* grants admin:users:view
fn has_permission(user_permissions: &[String], required_permission: &str) -> bool {
    let required_parts: Vec<&str> = required_permission.split(':').collect();
    
    for permission in user_permissions {
        let permission_parts: Vec<&str> = permission.split(':').collect();
        
        // Direct match
        if permission == required_permission {
            return true;
        }
        
        // Wildcard match
        if permission_parts.len() >= required_parts.len() {
            let mut matches = true;
            for (i, &required_part) in required_parts.iter().enumerate() {
                if i >= permission_parts.len() {
                    matches = false;
                    break;
                }
                
                if permission_parts[i] != "*" && permission_parts[i] != required_part {
                    matches = false;
                    break;
                }
            }
            if matches {
                return true;
            }
        }
    }
    
    false
}

/// Admin-only middleware (requires admin:*:* permission)
pub async fn admin_only_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_state = req.extensions()
        .get::<BearerAuthState>()
        .ok_or_else(|| {
            warn!("Admin check failed - no authentication state found");
            StatusCode::UNAUTHORIZED
        })?;

    if !has_permission(&auth_state.permissions, "admin:*:*") {
        warn!("Admin access denied for user: {}", auth_state.user_id);
        return Err(StatusCode::FORBIDDEN);
    }

    info!("Admin access granted for user: {}", auth_state.user_id);
    Ok(next.run(req).await)
}

/// Rate limiting based on user permissions
pub async fn permission_based_rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get auth state to determine rate limits
    let auth_state = req.extensions().get::<BearerAuthState>();
    
    let rate_limit = if let Some(state) = auth_state {
        match state.role.as_str() {
            "admin" => 1000, // Admin users get higher limits
            "premium" => 500, // Premium users
            _ => 100, // Regular users
        }
    } else {
        10 // Anonymous users get very low limits
    };

    // Add rate limit info to response headers
    let mut response = next.run(req).await;
    response.headers_mut().insert("X-RateLimit-Limit", rate_limit.to_string().parse().unwrap());
    
    Ok(response)
}

/// Extract authenticated user from request
pub fn get_authenticated_user(req: &Request) -> Option<&BearerAuthState> {
    req.extensions().get::<BearerAuthState>()
}

/// Create standardized error response for authentication failures
pub fn auth_error_response(status: StatusCode, message: &str) -> Response {
    let error_json = serde_json::json!({
        "error": "authentication_failed",
        "message": message,
        "status": status.as_u16()
    });

    let mut response = Response::new(error_json.to_string().into());
    *response.status_mut() = status;
    response.headers_mut().insert(
        "content-type", 
        "application/json".parse().unwrap()
    );
    response.headers_mut().insert(
        "www-authenticate",
        "Bearer realm=\"EPSX API\"".parse().unwrap()
    );
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_matching() {
        let permissions = vec![
            "admin:*:*".to_string(),
            "epsx:analytics:view".to_string(),
            "epsx:users:read".to_string(),
        ];

        // Direct match
        assert!(has_permission(&permissions, "epsx:analytics:view"));
        
        // Wildcard match
        assert!(has_permission(&permissions, "admin:users:manage"));
        assert!(has_permission(&permissions, "admin:system:config"));
        
        // No match
        assert!(!has_permission(&permissions, "other:system:admin"));
        assert!(!has_permission(&permissions, "epsx:analytics:write"));
    }

    #[test]
    fn test_permission_specificity() {
        let permissions = vec![
            "epsx:users:read".to_string(),
            "epsx:users:write".to_string(),
        ];

        assert!(has_permission(&permissions, "epsx:users:read"));
        assert!(has_permission(&permissions, "epsx:users:write"));
        assert!(!has_permission(&permissions, "epsx:users:delete"));
        assert!(!has_permission(&permissions, "epsx:admin:*"));
    }

    #[test]
    fn test_admin_permissions() {
        let admin_permissions = vec!["admin:*:*".to_string()];
        
        assert!(has_permission(&admin_permissions, "admin:users:manage"));
        assert!(has_permission(&admin_permissions, "admin:system:configure"));
        assert!(has_permission(&admin_permissions, "admin:analytics:export"));
        
        // Admin permissions don't grant epsx permissions automatically
        assert!(!has_permission(&admin_permissions, "epsx:analytics:view"));
    }
}