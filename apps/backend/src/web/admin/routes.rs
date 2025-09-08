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
    list_api_keys_handler,
};
use crate::web::user::handlers::{login_handler, logout_handler, me_handler};
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
// Embedded timestamp permission handlers
use super::embedded_permission_handlers::{
    grant_embedded_permission,
    grant_bulk_embedded_permissions,
    validate_embedded_permissions,
    get_permission_expiry_status,
    revoke_embedded_permission,
    extend_embedded_permission,
    cleanup_expired_permissions,
};
use super::bulk_permission_handlers::{
    bulk_grant_permissions,
    bulk_revoke_permissions,
    bulk_assign_roles,
    bulk_apply_permission_template,
    bulk_validate_permissions,
};
use super::firebase_user_management::{
    create_user as firebase_create_user,
    get_user as firebase_get_user,
    update_user as firebase_update_user,
    delete_user as firebase_delete_user,
    list_users as firebase_list_users,
    set_user_role as firebase_set_user_role,
};
// Database role management removed - using permissions-based system
// V1 Granular permission management handlers
use super::granular_permissions::{
    grant_permission,
    revoke_permission,
    list_user_permissions,
    extend_permission,
    bulk_grant_permissions as granular_bulk_grant_permissions,
    get_permission_statistics,
};
// Admin notification handlers
use super::notification_handlers::{
    admin_send_notification,
    admin_broadcast_to_topic,
    admin_get_notification_stats,
    admin_get_user_notifications,
    admin_mark_notification_read,
    admin_delete_notification,
};
// Removed admin module management handlers - using simple roles
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
            crate::web::middleware::clean_auth_middleware
        ));
        
    // System administration routes (require system-configuration module)
    let system_config_routes = Router::new()
        .route("/api-keys", get(list_api_keys_handler))
        // Role cleanup removed - using permissions-based system
        .layer(axum::middleware::from_fn(
            crate::web::middleware::clean_auth_middleware
        ));
        
    // Security management routes (require security-management module)
    let security_mgmt_routes = Router::new()
        .layer(axum::middleware::from_fn(
            crate::web::middleware::clean_auth_middleware
        ));
        
    Router::new()
        // Public admin auth routes
        .route("/auth/logout", post(logout_handler))
        .route("/auth/profile", get(me_handler))
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
        
        // Database role management routes removed - using permissions-based system
        
        // Admin module routes removed - using simple role system
        
        // Bulk operations (require user-management module)
        .route("/users/bulk-update", post(bulk_update_users_handler))
        .route("/users/level-history", get(get_level_history_handler))
        
        // Bulk Permission Management routes (require user-management module)
        .route("/users/bulk/permissions/grant", post(bulk_grant_permissions))
        .route("/users/bulk/permissions/revoke", post(bulk_revoke_permissions))
        .route("/users/bulk/roles/assign", post(bulk_assign_roles))
        .route("/users/bulk/templates/apply", post(bulk_apply_permission_template))
        .route("/users/bulk/permissions/validate", post(bulk_validate_permissions))
        
        // Unified User Management routes (require user-management module)
        .route("/users/:user_id/unified", get(get_unified_user_data_handler))
        .route("/users/:user_id/profile", put(update_user_profile_handler))
        .route("/users/:user_id/roles", put(update_user_roles_handler))
        .route("/users/:user_id/modules", put(update_user_modules_handler))
        .route("/users/:user_id/billing", put(update_user_billing_handler))
        .route("/users/:user_id/activity", get(get_user_activity_handler))
        
        // Embedded Timestamp Permission Management routes (require user-management module)
        .route("/users/:user_id/embedded-permissions", post(grant_embedded_permission))
        .route("/users/bulk/embedded-permissions", post(grant_bulk_embedded_permissions))
        .route("/users/:user_id/embedded-permissions/validate", post(validate_embedded_permissions))
        .route("/users/:user_id/permissions/expiry-status", get(get_permission_expiry_status))
        .route("/users/:user_id/embedded-permissions/revoke", post(revoke_embedded_permission))
        .route("/users/:user_id/embedded-permissions/extend", post(extend_embedded_permission))
        .route("/embedded-permissions/cleanup-expired", post(cleanup_expired_permissions))
        
        // V1 Granular Permission Management API (require user-management module)
        .route("/users/:user_id/granular-permissions/grant", post(grant_permission))
        .route("/users/:user_id/granular-permissions/revoke", post(revoke_permission))
        .route("/users/:user_id/granular-permissions", get(list_user_permissions))
        .route("/users/:user_id/granular-permissions/extend", post(extend_permission))
        .route("/granular-permissions/bulk/grant", post(granular_bulk_grant_permissions))
        .route("/granular-permissions/statistics", get(get_permission_statistics))
        
        // Simple role system: complex permission management routes removed
        // Use basic user role updates through /users/:user_id endpoints
        
        // Analytics routes (require analytics-access module)
        .route("/analytics/permissions", get(get_permission_analytics_handler))
        .route("/analytics/recommendations", get(get_permission_recommendations_handler))
        .route("/analytics/performance", get(get_performance_metrics_handler))
        .route("/analytics/security-risks", get(get_security_risk_analysis_handler))
        
        // Admin notification routes (require admin permissions)
        .route("/notifications/send", post(admin_send_notification))
        .route("/notifications/broadcast", post(admin_broadcast_to_topic))
        .route("/notifications/stats", get(admin_get_notification_stats))
        .route("/notifications/list", get(admin_get_user_notifications))
        .route("/notifications/:id/read", put(admin_mark_notification_read))
        .route("/notifications/:id", delete(admin_delete_notification))
        .layer(axum::middleware::from_fn(
            crate::web::middleware::clean_auth_middleware
        ))
}

pub fn create_admin_public_routes() -> Router<AppState> {
    Router::new()
        // Public admin authentication routes (no auth required)
        .route("/auth/login", post(login_handler))
}