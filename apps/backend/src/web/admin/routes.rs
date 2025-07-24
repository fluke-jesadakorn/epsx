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
};
use crate::web::AppState;

pub fn create_admin_routes() -> Router<AppState> {
    Router::new()
        // Authentication routes for admin
        .route("/authentication/login", post(super::super::auth::enhanced_handlers::enhanced_login_handler))
        .route("/authentication/logout", post(super::super::auth::handlers::logout_handler))
        .route("/authentication/profile", get(super::super::auth::handlers::me_handler))
        
        // Analytics routes
        .route("/analytics/statistics", get(get_user_stats_handler))
        
        // User management routes
        .route("/user-management/users", get(list_users_handler))
        .route("/user-management/users", post(create_user_handler))
        .route("/user-management/users/:user_id", get(get_user_handler))
        .route("/user-management/users/:user_id", put(update_user_role_handler))
        .route("/user-management/users/bulk-update-levels", post(bulk_update_levels_handler))
        .route("/user-management/users/:user_id/level-history", get(get_level_history_handler))
}