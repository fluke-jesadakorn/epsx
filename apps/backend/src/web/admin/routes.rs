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
        // User management routes
        .route("/users", get(list_users_handler))
        .route("/users", post(create_user_handler))
        .route("/users/:user_id", get(get_user_handler))
        .route("/users/:user_id", put(update_user_role_handler))
        .route("/users/bulk-update-levels", post(bulk_update_levels_handler))
        .route("/users/:user_id/level-history", get(get_level_history_handler))
        
        // Statistics routes
        .route("/stats", get(get_user_stats_handler))
}