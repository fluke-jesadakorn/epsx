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
// Casbin handlers removed - using modern JWT auth system
// use super::casbin_handlers::{...};
// Removed permission profile handlers - using simple roles
// use super::permission_profile_handlers::{...};

// Removed temporary permission handlers - using simple roles
// use super::temporary_permission_handlers::{...};

// Removed permission export/import handlers - using simple roles  
// use super::permission_export_import_handlers::{...};
use super::analytics_handlers::{
    get_permission_analytics_handler,
    get_permission_recommendations_handler,
    get_performance_metrics_handler,
    get_security_risk_analysis_handler,
};
use super::search_handlers::{
    search_users_handler,
};
use super::firebase_user_management::{
    create_user as firebase_create_user,
    get_user as firebase_get_user,
    update_user as firebase_update_user,
    delete_user as firebase_delete_user,
    list_users as firebase_list_users,
    set_user_role as firebase_set_user_role,
};
use super::database_role_management::{
    get_user_role as db_get_user_role,
    assign_user_role as db_assign_user_role,
    update_user_permissions as db_update_user_permissions,
    revoke_user_role as db_revoke_user_role,
    list_users_by_role as db_list_users_by_role,
    get_role_assignment_history as db_get_role_assignment_history,
    cleanup_expired_roles as db_cleanup_expired_roles,
};
use super::admin_role_management::{
    get_all_admin_modules,
    get_user_admin_modules,
    assign_admin_modules,
    revoke_admin_modules,
    assign_all_admin_modules,
    get_admin_role_audit,
    check_admin_module_access,
    get_user_admin_module_details,
    get_current_user_admin_modules,
};
use crate::web::auth::AppState;

pub fn create_admin_routes() -> Router<AppState> {
    // Basic admin routes (require user-management module)
    let user_mgmt_routes = Router::new()
        .route("/analytics/user-statistics", get(get_user_stats_handler))
        .route("/users", get(list_users_handler))
        .route("/users", post(create_user_handler))
        .route("/users/:user_id", get(get_user_handler))
        .route("/users/:user_id", put(update_user_handler))
        .route("/users/:user_id", delete(delete_user_handler))
        .route("/users/search", get(search_users_handler))
        .layer(axum::middleware::from_fn(
            crate::web::middleware::modern_jwt_auth_middleware
        ));
        
    // System administration routes (require system-configuration module)
    let system_config_routes = Router::new()
        .route("/api-keys", get(list_api_keys_handler))
        .route("/roles/cleanup-expired", post(db_cleanup_expired_roles))
        .layer(axum::middleware::from_fn(
            crate::web::middleware::modern_jwt_auth_middleware
        ));
        
    // Security management routes (require security-management module)
    let security_mgmt_routes = Router::new()
        .route("/admin-modules", get(get_all_admin_modules))
        .route("/admin-modules/users/:firebase_uid", get(get_user_admin_modules))
        .route("/admin-modules/assign", post(assign_admin_modules))
        .route("/admin-modules/revoke", post(revoke_admin_modules))
        .layer(axum::middleware::from_fn(
            crate::web::middleware::modern_jwt_auth_middleware
        ));
        
    Router::new()
        // Public admin auth routes
        .route("/auth/logout", post(super::super::auth::handlers::logout_handler))
        .route("/auth/profile", get(super::super::auth::handlers::me_handler))
        // Merge protected routes
        .merge(user_mgmt_routes)
        .merge(system_config_routes)
        .merge(security_mgmt_routes)
        
        // Firebase User management routes (require user-management module)
        .route("/firebase/users", get(firebase_list_users))
        .route("/firebase/users", post(firebase_create_user))
        .route("/firebase/users/:uid", get(firebase_get_user))
        .route("/firebase/users/:uid", put(firebase_update_user))
        .route("/firebase/users/:uid", delete(firebase_delete_user))
        .route("/firebase/users/:uid/role", post(firebase_set_user_role))
        
        // Database Role management routes
        .route("/roles/users/:firebase_uid", get(db_get_user_role))
        .route("/roles/users/:firebase_uid/assign", post(db_assign_user_role))
        .route("/roles/users/:firebase_uid/permissions", put(db_update_user_permissions))
        .route("/roles/users/:firebase_uid", delete(db_revoke_user_role))
        .route("/roles/users-by-role", get(db_list_users_by_role))
        .route("/roles/users/:firebase_uid/history", get(db_get_role_assignment_history))
        
        // Admin module details routes
        .route("/admin-modules/users/:firebase_uid/details", get(get_user_admin_module_details))
        .route("/admin-modules/users/:firebase_uid/assign-all", post(assign_all_admin_modules))
        .route("/admin-modules/users/:firebase_uid/audit", get(get_admin_role_audit))
        .route("/admin-modules/users/:firebase_uid/access/:module_code", get(check_admin_module_access))
        .route("/modules/user", get(get_current_user_admin_modules))
        
        // Bulk operations (require user-management module)
        .route("/users/bulk-update", post(bulk_update_users_handler))
        .route("/users/bulk/assign-modules", post(bulk_assign_modules_handler))
        .route("/users/level-history", get(get_level_history_handler))
        
        // Unified User Management routes (require user-management module)
        .route("/users/:user_id/unified", get(get_unified_user_data_handler))
        .route("/users/:user_id/profile", put(update_user_profile_handler))
        .route("/users/:user_id/roles", put(update_user_roles_handler))
        .route("/users/:user_id/modules", put(update_user_modules_handler))
        .route("/users/:user_id/billing", put(update_user_billing_handler))
        .route("/users/:user_id/activity", get(get_user_activity_handler))
        
        // Simple role system: complex permission management routes removed
        // Use basic user role updates through /users/:user_id endpoints
        
        // Analytics routes (require analytics-access module)
        .route("/analytics/permissions", get(get_permission_analytics_handler))
        .route("/analytics/recommendations", get(get_permission_recommendations_handler))
        .route("/analytics/performance", get(get_performance_metrics_handler))
        .route("/analytics/security-risks", get(get_security_risk_analysis_handler))
}

pub fn create_admin_public_routes() -> Router<AppState> {
    Router::new()
        // Public admin authentication routes (no auth required)
        .route("/auth/login", post(super::super::auth::handlers::login_handler))
}