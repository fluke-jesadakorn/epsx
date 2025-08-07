// Admin routes configuration

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use super::handlers::{
    create_user_handler,
    get_user_handler,
    list_users_handler,
    update_user_handler,
    delete_user_handler,
    get_user_stats_handler,
    get_level_history_handler,
    bulk_update_users_handler,
    bulk_assign_modules_handler,
    assign_permission_profiles_handler,
    list_api_keys_handler,
};
use super::unified_user_handlers::{
    get_unified_user_data_handler,
    update_user_profile_handler,
    update_user_roles_handler,
    update_user_modules_handler,
    update_user_billing_handler,
    get_user_activity_handler,
};
use super::casbin_handlers::{
    get_all_policies_handler,
    add_policy_handler,
    remove_policy_handler,
    add_batch_policies_handler,
    assign_role_handler,
    remove_role_handler,
    get_user_roles_handler,
    get_user_permissions_handler,
    test_policy_handler,
    reload_policies_handler,
    get_cache_stats_handler,
    clear_cache_handler,
};
use crate::web::auth::AppState;

pub fn create_admin_routes() -> Router<AppState> {
    Router::new()
        // Authentication routes for admin (these will be protected by the parent router)
        .route("/auth/logout", post(super::super::auth::handlers::logout_handler))
        .route("/auth/profile", get(super::super::auth::handlers::me_handler))
        
        // Analytics routes
        .route("/analytics/user-statistics", get(get_user_stats_handler))
        
        // User management routes (legacy)
        .route("/users", get(list_users_handler))
        .route("/users", post(create_user_handler))
        .route("/users/:user_id", get(get_user_handler))
        .route("/users/:user_id", put(update_user_handler))
        .route("/users/:user_id", delete(delete_user_handler))
        .route("/users/bulk-update", post(bulk_update_users_handler))
        .route("/users/bulk/assign-modules", post(bulk_assign_modules_handler))
        .route("/users/level-history", get(get_level_history_handler))
        
        // Unified User Management routes (new refactored interface)
        .route("/users/:user_id/unified", get(get_unified_user_data_handler))
        .route("/users/:user_id/profile", put(update_user_profile_handler))
        .route("/users/:user_id/roles", put(update_user_roles_handler))
        .route("/users/:user_id/modules", put(update_user_modules_handler))
        .route("/users/:user_id/billing", put(update_user_billing_handler))
        .route("/users/:user_id/activity", get(get_user_activity_handler))
        
        // Permission profile management routes
        .route("/permission-profiles/assign", post(assign_permission_profiles_handler))
        
        // Casbin policy management routes
        .route("/casbin/policies", get(get_all_policies_handler))
        .route("/casbin/policies", post(add_policy_handler))
        .route("/casbin/policies", delete(remove_policy_handler))
        .route("/casbin/policies/batch", post(add_batch_policies_handler))
        .route("/casbin/policies/test", post(test_policy_handler))
        .route("/casbin/policies/reload", post(reload_policies_handler))
        
        // Casbin role management routes
        .route("/casbin/roles", post(assign_role_handler))
        .route("/casbin/roles", delete(remove_role_handler))
        .route("/casbin/users/:user_id/roles", get(get_user_roles_handler))
        .route("/casbin/users/:user_id/permissions", get(get_user_permissions_handler))
        
        // Casbin cache management routes
        .route("/casbin/cache/stats", get(get_cache_stats_handler))
        .route("/casbin/cache/clear", post(clear_cache_handler))
        
        // API Keys management routes
        .route("/api-keys", get(list_api_keys_handler))
}

pub fn create_admin_public_routes() -> Router<AppState> {
    Router::new()
        // Public admin authentication routes (no auth required)
        .route("/auth/login", post(super::super::auth::handlers::login_handler))
}