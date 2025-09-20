// User management routes

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::web::auth::routes::AppState;
use super::handlers::{
    get_profile_handler,
    update_profile_handler,
    get_permissions_handler,
    verify_ownership_handler,
    get_holdings_handler,
    delegate_permission_handler,
};
use super::permissions::{
    get_user_permissions,
    check_user_permission,
};

/// Create v1 API routes for user operations with RESTful patterns
pub fn user_routes_v1() -> Router<AppState> {
    // RESTful user routes (available to all authenticated users)
    let user_routes = Router::new()
        .route("/api/v1/users/me", get(get_profile_handler))
        .route("/api/v1/users/me", put(update_profile_handler))
        // .route("/api/v1/users/me/expiration", get(get_expiration_status_handler)) // Handler missing
        // .route("/api/v1/users/me/notifications", get(get_notifications_handler)) // Handler missing
        // .route("/api/v1/users/me/notifications/mark-read", post(mark_notifications_read_handler)) // Handler missing
        .route("/api/v1/users/me/permissions", get(get_user_permissions))
        .route("/api/v1/users/me/permissions/check", get(check_user_permission));
    
    // Premium features (user role and above - simple role system)
    let premium_routes = Router::new()
        // .route("/api/v1/users/me/expiration/checks", post(request_expiration_check_handler)) // Handler missing
        .route("/api/v1/health", get(|| async { "OK" }));
    
    // Admin features (require admin role - simple role system)
    let admin_routes = Router::new()
        // .route("/api/v1/admin/users", get(list_users_handler)) // Handler missing
        // .route("/api/v1/admin/users/:id", delete(delete_user_handler)) // Handler missing
        .route("/api/v1/admin/health", get(|| async { "OK" }));

    // Legacy routes for backward compatibility
    let legacy_routes = Router::new()
        .route("/users/profile", get(get_profile_handler))
        .route("/users/profile", put(update_profile_handler))
        // .route("/users/expiration-status", get(get_expiration_status_handler)) // Handler missing
        // .route("/users/notifications", get(get_notifications_handler)) // Handler missing
        // .route("/users/notifications/mark-read", post(mark_notifications_read_handler)) // Handler missing
        // .route("/users/request-expiration-check", post(request_expiration_check_handler)) // Handler missing
        // .route("/users", get(list_users_handler)) // Handler missing
        // .route("/users/:id", delete(delete_user_handler)) // Handler missing
;
    
    Router::new()
        .merge(user_routes)
        .merge(premium_routes)
        .merge(admin_routes)
        .merge(legacy_routes)
}

/// Create legacy user routes (backward compatibility)
pub fn user_routes() -> Router<AppState> {
    Router::new()
        // Current user profile operations
        .route("/me", get(get_profile_handler))
        .route("/me", put(update_profile_handler))
        
        // Admin operations - handlers missing
        // .route("/users", get(list_users_handler)) // Handler missing
        // .route("/users/:id", delete(delete_user_handler)) // Handler missing
        
        // Auth operations - handler missing
        // .route("/logout", post(logout_handler)) // Handler missing
        
        // User-driven expiration management - handlers missing
        // .route("/me/expiration-status", get(get_expiration_status_handler)) // Handler missing
        // .route("/me/request-expiration-check", post(request_expiration_check_handler)) // Handler missing
        
        // User notifications (deprecated - use /api/v1/notifications instead) - handlers missing
        // .route("/me/notifications", get(get_notifications_handler)) // Handler missing
        // .route("/me/notifications/mark-read", post(mark_notifications_read_handler)) // Handler missing
        
}