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

/// Create v1 API routes for user operations
pub fn user_routes_v1() -> Router<AppState> {
    Router::new()
        // User profile operations (will be nested under /api/v1/users)
        .route("/users/profile", get(get_profile_handler))
        .route("/users/profile", put(update_profile_handler))
        .route("/users", get(list_users_handler))
        .route("/users/:id", delete(delete_user_handler))
        
        // User-driven expiration management
        .route("/users/expiration-status", get(get_expiration_status_handler))
        .route("/users/request-expiration-check", post(request_expiration_check_handler))
        
        // User notifications
        .route("/users/notifications", get(get_notifications_handler))
        .route("/users/notifications/mark-read", post(mark_notifications_read_handler))
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
        
        // User notifications
        .route("/me/notifications", get(get_notifications_handler))
        .route("/me/notifications/mark-read", post(mark_notifications_read_handler))
}