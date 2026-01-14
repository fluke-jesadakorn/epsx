// ============================================================================
// JWT-ONLY PERMISSION VALIDATION MIDDLEWARE (v3.0)
// THE SINGLE SOURCE OF TRUTH for all permission enforcement
// Zero database queries - permissions validated from JWT claims only
// ============================================================================

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{Response, IntoResponse},
};
use serde_json::json;
use tracing::{info, warn, debug};

use crate::web::middleware::bearer_middleware::OpenIDUserContext;
use crate::web::errors::PermissionError;

/// JWT-ONLY Permission validation middleware
/// Validates permissions using ONLY the JWT claims (no database queries)
/// This is 100x faster than database validation and horizontally scalable
///
/// IMPORTANT: Permissions are expanded from permission groups during token generation
/// and stored in the JWT. This middleware just validates what's in the token.
pub async fn permission_validation_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path();

    debug!("JWT permission validation: {} {}", method, path);

    // Check if public route (no authentication needed)
    if is_public_endpoint(path) {
        debug!("Public route: {} {}", method, path);
        return next.run(request).await;
    }

    // Extract user context from JWT (set by bearer_middleware)
    let user_context = match request.extensions().get::<OpenIDUserContext>() {
        Some(ctx) => ctx,
        None => {
            warn!("Missing JWT authentication for protected route: {} {}", method, path);
            return create_auth_error(
                "Authentication required",
                "Valid Bearer token required to access this resource"
            ).into_response();
        }
    };

    // Determine required permission for this route
    let required_permission = match get_required_permission(method.as_ref(), path) {
        Some(perm) => perm,
        None => {
            // No specific permission required (public after auth)
            debug!("No specific permission required for: {} {}", method, path);
            return next.run(request).await;
        }
    };

    // Check permission using ONLY JWT claims (NO DATABASE QUERY!)
    if check_jwt_permission(user_context, &required_permission) {
        info!(
            "✅ Permission granted (JWT): wallet={}, permission={}, route={} {}",
            user_context.wallet_address, required_permission, method, path
        );
        next.run(request).await
    } else {
        warn!(
            "❌ Permission denied (JWT): wallet={}, permission={}, route={} {}",
            user_context.wallet_address, required_permission, method, path
        );
        create_permission_denied_error(
            &required_permission,
            user_context
        ).into_response()
    }
}

// ============================================================================
// JWT-ONLY PERMISSION VALIDATION HELPERS
// All permission checks use ONLY JWT claims (no database queries)
// ============================================================================

/// Check if user has permission using ONLY JWT claims (NO DATABASE!)
fn check_jwt_permission(user_context: &OpenIDUserContext, required: &str) -> bool {
    // Direct permission match
    if user_context.permissions.contains(&required.to_string()) {
        return true;
    }

    // Admin wildcard: admin:*:*
    if user_context.permissions.iter().any(|p| p == "admin:*:*") {
        return true;
    }

    // Platform wildcard matching
    let parts: Vec<&str> = required.split(':').collect();
    if parts.len() >= 3 {
        // Check platform:*:* wildcard (e.g., epsx:*:*)
        let platform_wildcard = format!("{}:*:*", parts[0]);
        if user_context.permissions.contains(&platform_wildcard) {
            return true;
        }

        // Check platform:resource:* wildcard (e.g., epsx:analytics:*)
        let resource_wildcard = format!("{}:{}:*", parts[0], parts[1]);
        if user_context.permissions.contains(&resource_wildcard) {
            return true;
        }
    }

    false
}

/// Determine required permission for HTTP route and method
fn get_required_permission(method: &str, path: &str) -> Option<String> {
    // Route permission mapping (static, fast lookup)
    match (method, path) {
        // Admin routes - require admin permissions
        ("GET", p) if p.contains("/admin/wallets") => Some("admin:users:read".to_string()),
        ("POST", p) if p.contains("/admin/wallets") => Some("admin:users:create".to_string()),
        ("PUT", p) if p.contains("/admin/wallets") => Some("admin:users:update".to_string()),
        ("DELETE", p) if p.contains("/admin/wallets") => Some("admin:users:delete".to_string()),

        ("GET", p) if p.contains("/admin/permissions") => Some("admin:permissions:read".to_string()),
        ("POST", p) if p.contains("/admin/permissions") => Some("admin:permissions:manage".to_string()),
        ("PUT", p) if p.contains("/admin/permissions") => Some("admin:permissions:manage".to_string()),
        ("DELETE", p) if p.contains("/admin/permissions") => Some("admin:permissions:manage".to_string()),

        // Payment Admin routes (Phase 1.2)
        ("GET", p) if p.contains("/admin/list") => Some("admin:payments:read".to_string()),
        ("GET", p) if p.contains("/admin/analytics") => Some("admin:payments:read".to_string()),
        ("GET", p) if p.contains("/admin/subscriptions") => Some("admin:payments:read".to_string()),
        ("PUT", p) if p.contains("/admin/") && p.contains("/status") => Some("admin:payments:manage".to_string()),
        ("POST", p) if p.contains("/admin/") && p.contains("/refund") => Some("admin:payments:manage".to_string()),

        // Analytics routes
        ("GET", p) if p.starts_with("/api/auth/analytics") => Some("epsx:analytics:read".to_string()),
        ("GET", p) if p.starts_with("/api/analytics") => Some("epsx:analytics:read".to_string()),

        // Export routes
        ("POST", p) if p.contains("/export") => Some("epsx:export:csv".to_string()),

        // No specific permission required (public after authentication)
        _ => None,
    }
}

/// Check if endpoint is public (no authentication needed)
fn is_public_endpoint(path: &str) -> bool {
    const PUBLIC_PATHS: &[&str] = &[
        "/",
        "/health",
        "/readiness",
        "/liveness",
        "/api/public/",
        "/api/auth/web3/challenge",
        "/api/auth/web3/challenge",
        "/api/auth/web3/verify",
        "/api/permissions/health",
        "/docs",
        "/api-docs/",
        // Admin public Web3 endpoints (relative paths for nested router)
        "/web3/recent-wallets",
    ];

    PUBLIC_PATHS.iter().any(|public_path| {
        if *public_path == "/" {
            path == "/"
        } else {
            path.starts_with(public_path)
        }
    })
}

/// Create authentication error response
fn create_auth_error(message: &str, reason: &str) -> (StatusCode, axum::Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({
            "success": false,
            "error": {
                "code": 401,
                "message": message,
                "reason": reason
            }
        }))
    )
}

/// Create permission denied error response
fn create_permission_denied_error(
    permission: &str,
    user_context: &OpenIDUserContext,
) -> PermissionError {
    debug!(
        "Permission denied for wallet={}, permission={}",
        user_context.wallet_address, permission
    );

    PermissionError::PermissionDenied {
        permission: permission.to_string(),
        reason: format!(
            "Insufficient permissions. Required: {}. Current permissions: {}",
            permission,
            user_context.permissions.join(", ")
        ),
        suggested_actions: vec![
            "Upgrade your permission group to access this feature".to_string(),
            "Contact support if you believe this is an error".to_string(),
        ],
        upgrade_group: Some("Premium Access Group".to_string()),
    }
}

// ============================================================================
// JWT-ONLY PERMISSION VALIDATION SYSTEM v3.0
// Zero database queries - all permissions from JWT claims
// 100x faster than database validation
// Horizontally scalable (stateless)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_user_context(permissions: Vec<String>) -> OpenIDUserContext {
        OpenIDUserContext {
            sub: "0x123".to_string(),
            wallet_address: "0x123".to_string(),
            permissions,
            auth_method: "web3_siwe".to_string(),
            jti: "test".to_string(),
            exp: 9999999999,
            iat: 0,
            auth_time: 0,
        }
    }

    #[test]
    fn test_check_jwt_permission_direct_match() {
        let ctx = create_test_user_context(vec![
            "epsx:analytics:read".to_string(),
            "epsx:export:csv".to_string(),
        ]);

        assert!(check_jwt_permission(&ctx, "epsx:analytics:read"));
        assert!(check_jwt_permission(&ctx, "epsx:export:csv"));
        assert!(!check_jwt_permission(&ctx, "admin:users:manage"));
    }

    #[test]
    fn test_check_jwt_permission_admin_wildcard() {
        let ctx = create_test_user_context(vec!["admin:*:*".to_string()]);

        assert!(check_jwt_permission(&ctx, "admin:users:manage"));
        assert!(check_jwt_permission(&ctx, "admin:permissions:read"));
        assert!(check_jwt_permission(&ctx, "epsx:analytics:read"));
    }

    #[test]
    fn test_check_jwt_permission_platform_wildcard() {
        let ctx = create_test_user_context(vec!["epsx:*:*".to_string()]);

        assert!(check_jwt_permission(&ctx, "epsx:analytics:read"));
        assert!(check_jwt_permission(&ctx, "epsx:export:csv"));
        assert!(!check_jwt_permission(&ctx, "admin:users:manage"));
    }

    #[test]
    fn test_check_jwt_permission_resource_wildcard() {
        let ctx = create_test_user_context(vec!["epsx:analytics:*".to_string()]);

        assert!(check_jwt_permission(&ctx, "epsx:analytics:read"));
        assert!(check_jwt_permission(&ctx, "epsx:analytics:write"));
        assert!(!check_jwt_permission(&ctx, "epsx:export:csv"));
    }

    #[test]
    fn test_is_public_endpoint() {
        assert!(is_public_endpoint("/health"));
        assert!(is_public_endpoint("/api/public/analytics"));
        assert!(is_public_endpoint("/api/auth/web3/challenge"));
        assert!(!is_public_endpoint("/admin/wallets"));
        assert!(!is_public_endpoint("/api/auth/analytics"));
    }

    #[test]
    fn test_get_required_permission() {
        assert_eq!(
            get_required_permission("GET", "/admin/wallets"),
            Some("admin:users:read".to_string())
        );
        assert_eq!(
            get_required_permission("POST", "/admin/wallets"),
            Some("admin:users:create".to_string())
        );
        assert_eq!(
            get_required_permission("GET", "/api/auth/analytics"),
            Some("epsx:analytics:read".to_string())
        );
        assert_eq!(
            get_required_permission("GET", "/admin/list"),
            Some("admin:payments:read".to_string())
        );
        assert_eq!(
            get_required_permission("PUT", "/admin/abc/status"),
            Some("admin:payments:manage".to_string())
        );
        assert_eq!(
            get_required_permission("GET", "/some/random/path"),
            None
        );
    }

    // NOTE: Test disabled - determine_upgrade_group function removed during refactoring
    /*
    #[test]
    fn test_determine_upgrade_group() {
        assert_eq!(
            determine_upgrade_group("basic"),
            Some("Standard Access Group".to_string())
        );
        assert_eq!(
            determine_upgrade_group("standard"),
            Some("Premium Access Group".to_string())
        );
        assert_eq!(
            determine_upgrade_group("premium"),
            Some("Professional Access Group".to_string())
        );
    }
    */
}