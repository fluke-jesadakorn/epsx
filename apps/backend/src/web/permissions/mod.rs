// Unified Permission Validation API Module
//
// Provides REST API endpoints for the unified permission validation system,
// enabling centralized permission checking, management, and monitoring across
// all EPSX applications with enterprise-grade performance and security.

pub mod handlers;
pub mod routes;
pub mod dto;
pub mod middleware;
pub mod websocket;

// Re-export public types
pub use handlers::*;
pub use routes::*;
pub use dto::*;
pub use middleware::*;
pub use websocket::*;

use axum::{
    Router,
    routing::{get, post, put, delete},
};
use crate::infra::container::AppContainer;

/// Create the permission API router with all endpoints
pub fn create_permission_router(container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Permission validation endpoints
        .route("/validate", post(handlers::validate_permissions))
        .route("/validate/batch", post(handlers::validate_permissions_batch))
        .route("/validate/realtime/:user_id", get(handlers::validate_permissions_realtime))
        
        // User permission management
        .route("/user/:user_id", get(handlers::get_user_permissions))
        .route("/user/:user_id/effective", get(handlers::get_effective_permissions))
        .route("/user/:user_id/grant", post(handlers::grant_user_permission))
        .route("/user/:user_id/revoke", delete(handlers::revoke_user_permission))
        .route("/user/:user_id/elevate", post(handlers::elevate_user_permissions))
        
        // Admin module permissions
        .route("/admin/modules", get(handlers::get_admin_modules))
        .route("/admin/modules/:module", get(handlers::get_module_permissions))
        .route("/admin/modules/:module/assign", post(handlers::assign_module_permission))
        .route("/admin/modules/:module/revoke", delete(handlers::revoke_module_permission))
        .route("/admin/grant", post(handlers::grant_temporary_admin))
        
        // Package tier permissions
        .route("/tiers", get(handlers::get_package_tiers))
        .route("/tiers/:tier", get(handlers::get_tier_permissions))
        .route("/tiers/:tier/features", get(handlers::get_tier_features))
        .route("/tiers/:tier/limits", get(handlers::get_tier_limits))
        .route("/tiers/upgrade", post(handlers::upgrade_user_tier))
        .route("/tiers/downgrade", post(handlers::downgrade_user_tier))
        
        // Permission templates
        .route("/templates", get(handlers::get_permission_templates))
        .route("/templates/:template_id", get(handlers::get_permission_template))
        .route("/templates", post(handlers::create_permission_template))
        .route("/templates/:template_id", put(handlers::update_permission_template))
        .route("/templates/:template_id", delete(handlers::delete_permission_template))
        .route("/templates/:template_id/apply", post(handlers::apply_permission_template))
        
        // Permission auditing
        .route("/audit/events", get(handlers::get_audit_events))
        .route("/audit/events/:event_id", get(handlers::get_audit_event))
        .route("/audit/security-events", get(handlers::get_security_events))
        .route("/audit/user/:user_id", get(handlers::get_user_audit_log))
        .route("/audit/export", post(handlers::export_audit_log))
        
        // System monitoring
        .route("/permissions/health", get(handlers::get_permission_health))
        .route("/metrics", get(handlers::get_permission_metrics))
        .route("/cache/stats", get(handlers::get_cache_statistics))
        .route("/cache/clear", post(handlers::clear_permission_cache))
        .route("/cache/warm", post(handlers::warm_permission_cache))
        
        // Real-time permission updates
        .route("/ws/permissions/:user_id", get(handlers::permission_websocket_handler))
        .route("/sse/permissions/:user_id", get(handlers::permission_sse_handler))
        
        // Bulk operations
        .route("/bulk/validate", post(handlers::bulk_validate_permissions))
        .route("/bulk/grant", post(handlers::bulk_grant_permissions))
        .route("/bulk/revoke", post(handlers::bulk_revoke_permissions))
        .route("/bulk/import", post(handlers::import_permissions))
        .route("/bulk/export", get(handlers::export_permissions))
        
        // Permission policies
        .route("/policies", get(handlers::get_permission_policies))
        .route("/policies/:policy_id", get(handlers::get_permission_policy))
        .route("/policies", post(handlers::create_permission_policy))
        .route("/policies/:policy_id", put(handlers::update_permission_policy))
        .route("/policies/:policy_id", delete(handlers::delete_permission_policy))
        .route("/policies/:policy_id/test", post(handlers::test_permission_policy))
        
        // Advanced features
        .route("/inheritance/check", post(handlers::check_permission_inheritance))
        .route("/delegation/create", post(handlers::create_permission_delegation))
        .route("/delegation/:delegation_id", delete(handlers::revoke_permission_delegation))
        .route("/constraints/evaluate", post(handlers::evaluate_permission_constraints))
        
        // State propagation and synchronization
        .with_state(container.clone())
        
        // Apply permission middleware
        .layer(axum::middleware::from_fn_with_state(container.clone(), middleware::permission_validation_middleware))
        .layer(axum::middleware::from_fn_with_state(container.clone(), middleware::permission_audit_middleware))
        .layer(axum::middleware::from_fn_with_state(container.clone(), middleware::permission_security_middleware))
}

// API versioning support
pub mod v1 {
    use super::*;
    
    pub fn create_v1_router(container: &AppContainer) -> Router<AppContainer> {
        Router::new()
            .nest("/permissions", create_permission_router(container))
            .with_state(container.clone())
    }
}

// Legacy API support for migration
pub mod legacy {
    use super::*;
    
    pub fn create_legacy_router(container: &AppContainer) -> Router {
        Router::new()
            .route("/check-permission", post(handlers::legacy_check_permission))
            .route("/user-permissions/:user_id", get(handlers::legacy_get_user_permissions))
            .route("/admin-check", post(handlers::legacy_admin_check))
            .with_state(container.clone())
    }
}