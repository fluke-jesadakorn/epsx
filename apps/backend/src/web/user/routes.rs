// User management routes

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use super::super::auth::routes::AppState;
use super::handlers::*;

/// Create v1 API routes for user operations
pub fn user_routes_v1() -> Router<AppState> {
    Router::new()
        // Admin operations (will be nested under /api/v1/admin)
        .route("/admin/users", get(list_users_handler))
        .route("/admin/users/:id", get(get_user_by_id_handler))
        .route("/admin/users/:id", delete(delete_user_handler))
}

/// Create legacy user routes (backward compatibility)
pub fn user_routes() -> Router<AppState> {
    Router::new()
        // Current user profile operations
        .route("/me", get(get_current_user_handler))
        .route("/me", put(update_user_profile_handler))
        
        // Admin operations
        .route("/users", get(list_users_handler))
        .route("/users/:id", get(get_user_by_id_handler))
        .route("/users/:id", delete(delete_user_handler))
        
        // Auth operations
        .route("/logout", post(logout_handler))
}