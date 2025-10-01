// Settings routes configuration

use axum::{
    routing::{get, post, put},
    Router,
};

use super::settings_handlers::{
    get_system_config_handler,
    update_system_config_handler,
    get_environment_config_handler,
    get_feature_flags_handler,
    update_feature_flags_handler,
    get_settings_by_category_handler,
    update_settings_by_category_handler,
    get_all_settings_handler,
    reset_settings_handler,
    backup_settings_handler,
    restore_settings_handler,
};
use crate::web::auth::AppState;

pub fn create_settings_router() -> Router<AppState> {
    Router::new()
        // System configuration endpoints
        .route("/system", get(get_system_config_handler))
        .route("/system", put(update_system_config_handler))
        
        // Environment configuration endpoints
        .route("/environment", get(get_environment_config_handler))
        
        // Feature flags endpoints
        .route("/feature-flags", get(get_feature_flags_handler))
        .route("/feature-flags", put(update_feature_flags_handler))
        
        // Category-based settings endpoints
        .route("/category/:category", get(get_settings_by_category_handler))
        .route("/category/:category", put(update_settings_by_category_handler))
        
        // All settings endpoints
        .route("/all", get(get_all_settings_handler))
        
        // Settings management endpoints
        .route("/reset", post(reset_settings_handler))
        .route("/backup", post(backup_settings_handler))
        .route("/backup/:backup_id/restore", post(restore_settings_handler))
}