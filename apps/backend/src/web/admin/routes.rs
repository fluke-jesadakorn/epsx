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
// Casbin handlers removed - using modern JWT auth system
// use super::casbin_handlers::{...};
use super::permission_profile_handlers::{
    list_permission_profiles_handler,
    get_permission_profile_handler,
    create_permission_profile_handler,
    update_permission_profile_handler,
    delete_permission_profile_handler,
    unassign_permission_profile_handler,
    get_permission_profile_categories_handler,
    get_permission_profile_tiers_handler,
    validate_permission_profile_assignment_handler,
    bulk_validate_permission_profile_assignment_handler,
};
use super::temporary_permission_handlers::{
    create_temporary_permission_handler,
    get_temporary_permission_handler,
    list_temporary_permissions_handler,
    update_temporary_permission_handler,
    revoke_temporary_permission_handler,
    delete_temporary_permission_handler,
    get_user_temporary_permissions_handler,
    cleanup_expired_permissions_handler,
    bulk_create_temporary_permissions_handler,
    bulk_revoke_temporary_permissions_handler,
    bulk_update_temporary_permissions_handler,
};
use super::permission_export_import_handlers::{
    export_user_permissions_handler,
    bulk_export_user_permissions_handler,
    validate_permission_import_handler,
    import_user_permissions_handler,
    generate_audit_report_handler,
    create_system_backup_handler,
    restore_system_backup_handler,
};
use super::analytics_handlers::{
    get_permission_analytics_handler,
    get_permission_recommendations_handler,
    get_performance_metrics_handler,
    get_security_risk_analysis_handler,
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
    Router::new()
        // Authentication routes for admin (these will be protected by the parent router)
        .route("/auth/logout", post(super::super::auth::handlers::logout_handler))
        .route("/auth/profile", get(super::super::auth::handlers::me_handler))
        
        // Analytics routes
        .route("/analytics/user-statistics", get(get_user_stats_handler))
        
        // Firebase User management routes (Firebase-native)
        .route("/firebase/users", get(firebase_list_users))
        .route("/firebase/users", post(firebase_create_user))
        .route("/firebase/users/:uid", get(firebase_get_user))
        .route("/firebase/users/:uid", put(firebase_update_user))
        .route("/firebase/users/:uid", delete(firebase_delete_user))
        .route("/firebase/users/:uid/role", post(firebase_set_user_role))
        
        // Database Role management routes (roles/permissions stored in database)
        .route("/roles/users/:firebase_uid", get(db_get_user_role))
        .route("/roles/users/:firebase_uid/assign", post(db_assign_user_role))
        .route("/roles/users/:firebase_uid/permissions", put(db_update_user_permissions))
        .route("/roles/users/:firebase_uid", delete(db_revoke_user_role))
        .route("/roles/users-by-role", get(db_list_users_by_role))
        .route("/roles/users/:firebase_uid/history", get(db_get_role_assignment_history))
        .route("/roles/cleanup-expired", post(db_cleanup_expired_roles))
        
        // Admin Role Management routes (granular admin modules system)
        .route("/admin-modules", get(get_all_admin_modules))
        .route("/admin-modules/users/:firebase_uid", get(get_user_admin_modules))
        .route("/admin-modules/users/:firebase_uid/details", get(get_user_admin_module_details))
        .route("/admin-modules/assign", post(assign_admin_modules))
        .route("/admin-modules/revoke", post(revoke_admin_modules))
        .route("/admin-modules/users/:firebase_uid/assign-all", post(assign_all_admin_modules))
        .route("/admin-modules/users/:firebase_uid/audit", get(get_admin_role_audit))
        .route("/admin-modules/users/:firebase_uid/access/:module_code", get(check_admin_module_access))
        
        // Current user admin modules routes
        .route("/modules/user", get(get_current_user_admin_modules))
        
        // User management routes (legacy - for backward compatibility)
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
        
        // Permission profile management routes (Full CRUD)
        .route("/permission-profiles", get(list_permission_profiles_handler))
        .route("/permission-profiles", post(create_permission_profile_handler))
        .route("/permission-profiles/:id", get(get_permission_profile_handler))
        .route("/permission-profiles/:id", put(update_permission_profile_handler))
        .route("/permission-profiles/:id", delete(delete_permission_profile_handler))
        .route("/permission-profiles/assign", post(assign_permission_profiles_handler))
        .route("/permission-profiles/unassign", delete(unassign_permission_profile_handler))
        .route("/permission-profiles/validate-assignment", post(validate_permission_profile_assignment_handler))
        .route("/permission-profiles/bulk-validate", post(bulk_validate_permission_profile_assignment_handler))
        .route("/permission-profiles/categories", get(get_permission_profile_categories_handler))
        .route("/permission-profiles/tiers", get(get_permission_profile_tiers_handler))
        
        // Temporary permissions routes
        .route("/temporary-permissions", post(create_temporary_permission_handler))
        .route("/temporary-permissions", get(list_temporary_permissions_handler))
        .route("/temporary-permissions/:id", get(get_temporary_permission_handler))
        .route("/temporary-permissions/:id", put(update_temporary_permission_handler))
        .route("/temporary-permissions/:id", delete(delete_temporary_permission_handler))
        .route("/temporary-permissions/:id/revoke", post(revoke_temporary_permission_handler))
        .route("/users/:user_id/temporary-permissions", get(get_user_temporary_permissions_handler))
        .route("/temporary-permissions/cleanup-expired", post(cleanup_expired_permissions_handler))
        
        // Temporary permissions bulk operations
        .route("/temporary-permissions/bulk-create", post(bulk_create_temporary_permissions_handler))
        .route("/temporary-permissions/bulk-revoke", post(bulk_revoke_temporary_permissions_handler))
        .route("/temporary-permissions/bulk-update", post(bulk_update_temporary_permissions_handler))
        
        // Permission export/import routes
        .route("/users/:user_id/permissions/export", get(export_user_permissions_handler))
        .route("/users/:user_id/permissions/validate-import", post(validate_permission_import_handler))
        .route("/users/:user_id/permissions/import", post(import_user_permissions_handler))
        .route("/permissions/bulk-export", post(bulk_export_user_permissions_handler))
        .route("/permissions/audit-report", post(generate_audit_report_handler))
        .route("/permissions/system-backup", post(create_system_backup_handler))
        .route("/permissions/system-backup/:backup_id/restore", post(restore_system_backup_handler))
        
        // Modern JWT-based permission management routes (replaces Casbin)
        // TODO: Implement modern permission management endpoints
        
        // API Keys management routes
        .route("/api-keys", get(list_api_keys_handler))
        
        // Analytics routes
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