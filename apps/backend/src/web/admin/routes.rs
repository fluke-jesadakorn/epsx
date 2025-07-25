// Admin routes configuration

use axum::{
    routing::{get, post, put},
    Router,
};

use super::handlers::{
    create_user_handler,
    get_user_handler,
    list_users_handler,
    update_user_role_handler,
    bulk_update_levels_handler,
    get_user_stats_handler,
    get_level_history_handler,
    list_permission_profiles_handler,
    assign_permission_profile_directly_handler,
    get_permission_profile_details_handler,
};
use crate::web::AppState;

pub fn create_admin_routes() -> Router<AppState> {
    Router::new()
        // Authentication routes for admin (these will be protected by the parent router)
        .route("/auth/logout", post(super::super::auth::handlers::logout_handler))
        .route("/auth/profile", get(super::super::auth::handlers::me_handler))
        
        // Analytics routes
        .route("/analytics/user-statistics", get(get_user_stats_handler))
        
        // User management routes
        .route("/users", get(list_users_handler))
        .route("/users", post(create_user_handler))
        .route("/users/:user_id", get(get_user_handler))
        .route("/users/:user_id", put(update_user_role_handler))
        .route("/users/batch-update-roles", post(bulk_update_levels_handler))
        .route("/users/:user_id/role-history", get(get_level_history_handler))
        
        // Permission profile management routes
        .route("/permission-profiles", get(list_permission_profiles_handler))
        .route("/permission-profiles/:profile_id", get(get_permission_profile_details_handler))
        .route("/permission-profiles/assign", post(assign_permission_profile_directly_handler))
}

pub fn create_admin_public_routes() -> Router<AppState> {
    Router::new()
        // Public admin authentication routes (no auth required)
        .route("/auth/login", post(super::super::auth::multi_handlers::multi_login_handler))
}