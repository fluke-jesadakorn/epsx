// User management routes

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::web::auth::AppState;
use super::handlers::{
    get_profile_handler,
    update_profile_handler,
    delete_user_handler,
    logout_handler,
    list_users_handler,
    // User-driven expiration handlers
    get_expiration_status_handler,
    request_expiration_check_handler,
    get_notifications_handler,
    mark_notifications_read_handler,
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
        .route("/api/v1/users/me/expiration", get(get_expiration_status_handler))
        .route("/api/v1/users/me/notifications", get(get_notifications_handler))
        .route("/api/v1/users/me/notifications/mark-read", post(mark_notifications_read_handler))
        .route("/api/v1/users/me/permissions", get(get_user_permissions))
        .route("/api/v1/users/me/permissions/check", get(check_user_permission));
    
    // Premium features (user role and above - simple role system)
    let premium_routes = Router::new()
        .route("/api/v1/users/me/expiration/checks", post(request_expiration_check_handler));
    
    // Admin features (require admin role - simple role system)
    let admin_routes = Router::new()
        .route("/api/v1/admin/users", get(list_users_handler))
        .route("/api/v1/admin/users/:id", delete(delete_user_handler));

    // Legacy routes for backward compatibility
    let legacy_routes = Router::new()
        .route("/users/profile", get(get_profile_handler))
        .route("/users/profile", put(update_profile_handler))
        .route("/users/expiration-status", get(get_expiration_status_handler))
        .route("/users/notifications", get(get_notifications_handler))
        .route("/users/notifications/mark-read", post(mark_notifications_read_handler))
        .route("/users/request-expiration-check", post(request_expiration_check_handler))
        .route("/users", get(list_users_handler))
        .route("/users/:id", delete(delete_user_handler))
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
        
        // Admin operations
        .route("/users", get(list_users_handler))
        .route("/users/:id", delete(delete_user_handler))
        
        // Auth operations  
        .route("/logout", post(logout_handler))
        
        // User-driven expiration management
        .route("/me/expiration-status", get(get_expiration_status_handler))
        .route("/me/request-expiration-check", post(request_expiration_check_handler))
        
        // User notifications (deprecated - use /api/v1/notifications instead)
        .route("/me/notifications", get(get_notifications_handler))
        .route("/me/notifications/mark-read", post(mark_notifications_read_handler))
        
}