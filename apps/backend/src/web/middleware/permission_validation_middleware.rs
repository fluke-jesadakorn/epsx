// ============================================================================
// JWT-ONLY PERMISSION VALIDATION MIDDLEWARE (v3.0)
// THE SINGLE SOURCE OF TRUTH for all permission enforcement
// Zero database queries - permissions validated from JWT claims only
// ============================================================================

use axum::{
    extract::{Request, State},
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
/// IMPORTANT: Permissions are expanded from permission plans during token generation
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
            "Permission granted (JWT): wallet={}, permission={}, route={} {}",
            user_context.wallet_address, required_permission, method, path
        );
        next.run(request).await
    } else {
        warn!(
            "Permission denied (JWT): wallet={}, permission={}, route={} {}\nCurrent Permissions: {:?}",
            user_context.wallet_address, required_permission, method, path, user_context.permissions
        );
        create_permission_denied_error(
            &required_permission,
            user_context
        ).into_response()
    }
}

/// Decorator-style permission guard — apply per route group via `from_fn_with_state`.
///
/// Usage:
/// ```ignore
/// Router::new()
///     .route("/items", get(list_items))
///     .layer(from_fn_with_state("admin:items:read", perm_guard))
/// ```
pub async fn perm_guard(
    State(required): State<&'static str>,
    request: Request,
    next: Next,
) -> Response {
    match request.extensions().get::<OpenIDUserContext>() {
        Some(ctx) if crate::core::permissions::has_permission(&ctx.permissions, required) => {
            next.run(request).await
        }
        Some(ctx) => {
            warn!(
                "perm_guard denied: wallet={}, required={}, permissions={:?}",
                ctx.wallet_address, required, ctx.permissions
            );
            PermissionError::PermissionDenied {
                permission: required.to_string(),
                reason: format!("Required: {}", required),
                suggested_actions: vec!["Upgrade your plan".into()],
                upgrade_plan: None,
            }.into_response()
        }
        None => {
            PermissionError::authentication_required("Bearer token required").into_response()
        }
    }
}

// ============================================================================
// JWT-ONLY PERMISSION VALIDATION HELPERS
// All permission checks use ONLY JWT claims (no database queries)
// ============================================================================

/// Check if user has permission using ONLY JWT claims (NO DATABASE!)
fn check_jwt_permission(user_context: &OpenIDUserContext, required: &str) -> bool {
    crate::core::permissions::has_permission(&user_context.permissions, required)
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
        ("GET", p) if p.contains("/admin/list") => Some("admin:payments:view".to_string()),
        ("GET", p) if p.contains("/admin/subscriptions") => Some("admin:payments:view".to_string()),
        ("PUT", p) if p.contains("/admin/") && p.contains("/status") => Some("admin:payments:manage".to_string()),
        ("POST", p) if p.contains("/admin/") && p.contains("/refund") => Some("admin:payments:manage".to_string()),

        // Security monitoring
        (_, p) if p.contains("/admin/security") => Some("admin:security:read".to_string()),

        // Plan management
        ("GET", p) if p.contains("/admin/plans") => Some("admin:plans:read".to_string()),
        ("POST" | "PUT" | "DELETE", p) if p.contains("/admin/plans") => Some("admin:plans:manage".to_string()),

        // Promotions
        (_, p) if p.contains("/admin/promotions") => Some("admin:promotions:manage".to_string()),

        // Notifications (admin)
        (_, p) if p.contains("/admin/notifications") => Some("admin:notifications:manage".to_string()),

        // Developer portal (admin)
        (_, p) if p.contains("/admin/developer-portal") => Some("admin:developer:manage".to_string()),

        // Payment links
        (_, p) if p.contains("/admin/payment-links") => Some("admin:payments:manage".to_string()),

        // Analytics (admin)
        ("GET", p) if p.contains("/admin/analytics") => Some("admin:analytics:view".to_string()),

        // Audit logs
        ("GET", p) if p.contains("/admin/audit") => Some("admin:audit:read".to_string()),

        // Performance
        ("GET", p) if p.contains("/admin/performance") => Some("admin:performance:view".to_string()),

        // Credits (admin)
        (_, p) if p.contains("/admin/credits") => Some("admin:payments:manage".to_string()),

        // Settings (admin)
        (_, p) if p.contains("/admin/settings") => Some("admin:settings:manage".to_string()),

        // Bulk/direct permission ops (sensitive)
        (_, p) if p.contains("/admin/permissions/bulk") || p.contains("/admin/permissions/direct") => Some("admin:permissions:manage".to_string()),

        // Admin chat management
        (_, p) if p.contains("/admin/chat") => Some("admin:chat:manage".to_string()),

        // CATCH-ALL: Any unmatched admin route requires base admin permission
        (_, p) if p.contains("/admin/") => Some("admin:dashboard:view".to_string()),

        // Analytics routes (user-facing)
        ("GET", p) if p.starts_with("/api/auth/analytics") => None,
        ("GET", p) if p.starts_with("/api/analytics") => None,

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
            "Upgrade your permission plan to access this feature".to_string(),
            "Contact support if you believe this is an error".to_string(),
        ],
        upgrade_plan: Some("Premium Access Plan".to_string()),
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
        // admin:*:* should NOT grant epsx:analytics:read (different platform)
        assert!(!check_jwt_permission(&ctx, "epsx:analytics:read"));
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
        // Wallet admin routes
        assert_eq!(
            get_required_permission("GET", "/admin/wallets"),
            Some("admin:users:read".to_string())
        );
        assert_eq!(
            get_required_permission("POST", "/admin/wallets"),
            Some("admin:users:create".to_string())
        );

        // Analytics (user-facing)
        assert_eq!(
            get_required_permission("GET", "/api/auth/analytics"),
            None
        );

        // Payment admin routes
        assert_eq!(
            get_required_permission("GET", "/admin/list"),
            Some("admin:payments:view".to_string())
        );
        assert_eq!(
            get_required_permission("PUT", "/admin/abc/status"),
            Some("admin:payments:manage".to_string())
        );

        // New admin route mappings
        assert_eq!(
            get_required_permission("GET", "/admin/plans"),
            Some("admin:plans:read".to_string())
        );
        assert_eq!(
            get_required_permission("POST", "/admin/plans"),
            Some("admin:plans:manage".to_string())
        );
        assert_eq!(
            get_required_permission("GET", "/admin/audit/logs"),
            Some("admin:audit:read".to_string())
        );
        assert_eq!(
            get_required_permission("GET", "/admin/performance/metrics"),
            Some("admin:performance:view".to_string())
        );
        assert_eq!(
            get_required_permission("GET", "/admin/settings"),
            Some("admin:settings:manage".to_string())
        );

        // Catch-all: any unknown admin route requires dashboard:view
        assert_eq!(
            get_required_permission("GET", "/admin/some-unknown-route/"),
            Some("admin:dashboard:view".to_string())
        );

        // Non-admin routes
        assert_eq!(
            get_required_permission("GET", "/some/random/path"),
            None
        );
    }

    #[test]
    fn test_admin_chat_permission_mapping() {
        assert_eq!(
            get_required_permission("GET", "/admin/chat/conversations"),
            Some("admin:chat:manage".to_string())
        );
        assert_eq!(
            get_required_permission("POST", "/admin/chat/conversations/123/messages"),
            Some("admin:chat:manage".to_string())
        );
    }

    #[tokio::test]
    async fn test_perm_guard_no_context_returns_401() {
        use axum::{body::Body, routing::get, Router, middleware::from_fn_with_state};
        use tower::ServiceExt;

        async fn ok_handler() -> &'static str { "ok" }

        let app = Router::new()
            .route("/test", get(ok_handler))
            .layer(from_fn_with_state("admin:users:read", perm_guard));

        let req = axum::http::Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_perm_guard_wrong_permission_returns_403() {
        use axum::{body::Body, routing::get, Router, Extension, middleware::from_fn_with_state};
        use tower::ServiceExt;

        async fn ok_handler() -> &'static str { "ok" }

        let ctx = create_test_user_context(vec!["epsx:analytics:read".to_string()]);

        let app = Router::new()
            .route("/test", get(ok_handler))
            .layer(from_fn_with_state("admin:users:read", perm_guard));

        let mut req = axum::http::Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(ctx);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_perm_guard_correct_permission_passes() {
        use axum::{body::Body, routing::get, Router, middleware::from_fn_with_state};
        use tower::ServiceExt;

        async fn ok_handler() -> &'static str { "ok" }

        let ctx = create_test_user_context(vec!["admin:users:read".to_string()]);

        let app = Router::new()
            .route("/test", get(ok_handler))
            .layer(from_fn_with_state("admin:users:read", perm_guard));

        let mut req = axum::http::Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(ctx);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_perm_guard_wildcard_permission_passes() {
        use axum::{body::Body, routing::get, Router, middleware::from_fn_with_state};
        use tower::ServiceExt;

        async fn ok_handler() -> &'static str { "ok" }

        let ctx = create_test_user_context(vec!["admin:*:*".to_string()]);

        let app = Router::new()
            .route("/test", get(ok_handler))
            .layer(from_fn_with_state("admin:permissions:manage", perm_guard));

        let mut req = axum::http::Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(ctx);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}