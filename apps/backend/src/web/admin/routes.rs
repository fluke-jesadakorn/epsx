// Admin routes configuration

use axum::{ routing::{ get, post, put, delete }, Router };

// Removed missing user handlers - they don't exist in the Web3 migration
// use super::unified_user_handlers::{
//   get_unified_user_data_handler,
//   update_user_profile_handler,
//   update_user_roles_handler,
//   update_user_modules_handler,
//   update_user_billing_handler,
//   get_user_activity_handler,
// }; // Removed - module deleted
// Casbin handlers removed - using Web3 wallet-first auth system
// Removed permission profile handlers - using simple roles
// Removed temporary permission handlers - using simple roles
// Removed permission export/import handlers - using simple roles
// Analytics functionality moved to frontend-only implementation
// use super::search_handlers::{ search_users_handler }; // Removed - module deleted
// Legacy embedded timestamp permission handlers removed for Web3-first migration
// use super::embedded_permission_handlers::{
//   grant_embedded_permission,
//   grant_bulk_embedded_permissions,
//   validate_embedded_permissions,
//   get_permission_expiry_status,
//   revoke_embedded_permission,
//   extend_embedded_permission,
//   cleanup_expired_permissions,
// };
// Legacy bulk permission handlers removed for Web3-first migration
// use super::bulk_permission_handlers::{
//   bulk_grant_permissions,
//   bulk_revoke_permissions,
//   bulk_assign_roles,
// //   bulk_apply_permission_template,
//   bulk_validate_permissions,
// };
// Firebase user management removed - migrated to Web3
// Database role management removed - using permissions-based system
// V1 Granular permission management handlers
// Legacy granular permission handlers removed for Web3-first migration
// use super::granular_permissions::{
//   grant_permission,
//   revoke_permission,
//   list_user_permissions,
//   extend_permission,
//   bulk_grant_permissions as granular_bulk_grant_permissions,
//   get_permission_statistics,
// };
// Legacy admin notification handlers removed for Web3-first migration
// use super::notification_handlers::{
//   admin_send_notification,
//   admin_broadcast_to_topic,
//   admin_get_notification_stats,
//   admin_get_user_notifications,
//   admin_mark_notification_read,
//   admin_delete_notification,
// };
// Security monitoring handlers
use super::security_monitoring_handlers::{ SecurityMonitoringHandlers };
// Permission hierarchy handlers (DISABLED during refactoring)
// use super::hierarchy_handlers::{
//     get_hierarchy_stats,
//     get_hierarchy_tree,
//     create_hierarchy,
//     remove_hierarchy,
//     resolve_user_permissions,
//     invalidate_user_cache,
//     test_hierarchy_resolution,
// };
// Legacy dynamic policy handlers removed for Web3-first migration
// use super::policy_handlers::{
//   list_policies_handler as get_policies,
//   create_policy_handler as create_policy,
//   evaluate_policy_handler as evaluate_policies,
//   delete_policy_handler as delete_policy,
//   get_policy_stats_handler as get_policy_stats,
//   toggle_policy_handler as toggle_policy_status,
// };
// Dynamic plan management handlers (simplified)
use super::plan_handlers::{
  create_plan_handler,
  get_plan_handler,
  list_plans_handler,
  create_subscription_handler,
};
// Unified permission group management handlers (wallet-first system)
use super::permission_group_handlers::{
  create_permission_group_handler,
  get_permission_group_handler,
  list_permission_groups_handler,
  update_permission_group_handler,
  delete_permission_group_handler,
  create_wallet_assignment_handler,
  get_wallet_assignments_handler,
  // NEW: Backend-centric permission validation handlers (THE AUTHORITY)
  validate_permission_handler,
  validate_bulk_permissions_handler,
  get_wallet_permissions_handler,
};
// Performance monitoring handlers
use super::performance_handlers::{
  get_auth_cache_performance,
  get_cache_summary,
  clear_auth_cache,
};
// Web3 permission management handlers
use super::web3_admin_handlers::{
  get_user_permissions,
  grant_manual_permission,
  create_nft_gate,
  create_token_gate,
  create_dao_proposal,
  get_nft_gates,
  get_token_gates,
  get_dao_proposals,
  get_recent_wallets,
  search_wallets,
};
// Consolidated user management handlers
use super::user_management_handlers::{
  list_users_handler,
  get_user_handler,
  update_user_handler,
  get_user_stats_handler,
};
// Analytics and business intelligence handlers
use super::analytics_handlers::{
  get_platform_overview_handler,
  get_user_analytics_handler,
  get_permission_analytics_handler,
  get_revenue_analytics_handler,
};
// Removed admin module management handlers - using simple roles
use crate::web::auth::AppState;

pub fn create_admin_routes() -> Router<AppState> {
  // User management routes removed - only wallet management available
  let user_mgmt_routes = Router::new();

  // System administration routes (require system-configuration module)
  let system_config_routes = Router::new()
    // .route("/api-keys", get(list_api_keys_handler)) // Handler missing
    // Role cleanup removed - using permissions-based system
    // TODO: Temporarily disabled due to Axum trait bound issues
    // .layer(axum::middleware::from_fn(crate::web::middleware::web3_auth_middleware))
    ;

  // Security management routes (require security-management module)
  let security_mgmt_routes = Router::new();
    // TODO: Temporarily disabled due to Axum trait bound issues
    // .layer(axum::middleware::from_fn(crate::web::middleware::web3_auth_middleware));

  Router::new()
    // Public admin auth routes - handlers missing
    // .route("/auth/logout", post(logout_handler))
    // .route("/auth/profile", get(me_handler))
    // Merge protected routes
    .merge(user_mgmt_routes)
    .merge(system_config_routes)
    .merge(security_mgmt_routes)
    // Firebase User management routes removed - migrated to Web3
    // .route("/firebase/users", get(firebase_list_users))
    // .route("/firebase/users", post(firebase_create_user))
    // .route("/firebase/users/:uid", get(firebase_get_user))
    // .route("/firebase/users/:uid", put(firebase_update_user))
    // .route("/firebase/users/:uid", delete(firebase_delete_user))
    // .route("/firebase/users/:uid/role", post(firebase_set_user_role))

    // Database role management routes removed - using permissions-based system

    // Admin module routes removed - using simple role system

    // Bulk operations - handlers missing
    // .route("/users/bulk-update", post(bulk_update_users_handler))
    // .route("/users/level-history", get(get_level_history_handler))

    // Legacy bulk permission handlers removed for Web3-first migration
    // .route("/users/bulk/permissions/grant", post(bulk_grant_permissions))
    // .route("/users/bulk/permissions/revoke", post(bulk_revoke_permissions))
    // .route("/users/bulk/roles/assign", post(bulk_assign_roles))
    // .route("/users/bulk/templates/apply", post(bulk_apply_permission_template))
    // .route("/users/bulk/permissions/validate", post(bulk_validate_permissions))
    // Unified User Management routes (require user-management module)
    // .route("/users/:user_id/unified", get(get_unified_user_data_handler)) // Removed - handler deleted
    // .route("/users/:user_id/profile", put(update_user_profile_handler)) // Removed - handler deleted
    // .route("/users/:user_id/roles", put(update_user_roles_handler)) // Removed - handler deleted
    // .route("/users/:user_id/modules", put(update_user_modules_handler)) // Removed - handler deleted
    // .route("/users/:user_id/billing", put(update_user_billing_handler)) // Removed - handler deleted
    // .route("/users/:user_id/activity", get(get_user_activity_handler)) // Removed - handler deleted
    // Legacy embedded permission handlers removed for Web3-first migration
    // .route(
    //   "/users/:user_id/embedded-permissions",
    //   post(grant_embedded_permission)
    // )
    // .route(
    //   "/users/bulk/embedded-permissions",
    //   post(grant_bulk_embedded_permissions)
    // )
    // .route(
    //   "/users/:user_id/embedded-permissions/validate",
    //   post(validate_embedded_permissions)
    // )
    // .route(
    //   "/users/:user_id/permissions/expiry-status",
    //   get(get_permission_expiry_status)
    // )
    // .route(
    //   "/users/:user_id/embedded-permissions/revoke",
    //   post(revoke_embedded_permission)
    // )
    // .route(
    //   "/users/:user_id/embedded-permissions/extend",
    //   post(extend_embedded_permission)
    // )
    // .route(
    //   "/embedded-permissions/cleanup-expired",
    //   post(cleanup_expired_permissions)
    // )
    // Legacy granular permission handlers removed for Web3-first migration
    // .route("/users/:user_id/granular-permissions/grant", post(grant_permission))
    // .route(
    //   "/users/:user_id/granular-permissions/revoke",
    //   post(revoke_permission)
    // )
    // .route("/users/:user_id/granular-permissions", get(list_user_permissions))
    // .route(
    //   "/users/:user_id/granular-permissions/extend",
    //   post(extend_permission)
    // )
    // .route(
    //   "/granular-permissions/bulk/grant",
    //   post(granular_bulk_grant_permissions)
    // )
    // .route("/granular-permissions/statistics", get(get_permission_statistics))
    // Simple role system: complex permission management routes removed
    // Use basic user role updates through /users/:user_id endpoints

    // Permission Hierarchy Management routes (DISABLED during refactoring)
    // .route("/permissions/hierarchy/stats", get(get_hierarchy_stats))
    // .route("/permissions/hierarchy/tree", get(get_hierarchy_tree))
    // .route("/permissions/hierarchy", post(create_hierarchy))
    // .route("/permissions/hierarchy/:hierarchy_id", delete(remove_hierarchy))
    // .route("/permissions/hierarchy/resolve", post(resolve_user_permissions))
    // .route("/permissions/hierarchy/test", get(test_hierarchy_resolution))
    // .route("/users/:user_id/permissions/cache/invalidate", delete(invalidate_user_cache))

    // Dynamic Policy Management routes (require admin:policies:* module)
    // .route("/policies", get(get_policies))
    // .route("/policies", post(create_policy))
    // .route("/policies/:policy_id", delete(delete_policy))
    // .route("/policies/:policy_id/toggle", put(toggle_policy_status))
    // .route("/policies/evaluate", post(evaluate_policies))
    // .route("/policies/templates", get(get_policy_templates)) // TODO: Implement
    // .route("/policies/stats", get(get_policy_stats))
    // Analytics routes removed - functionality moved to frontend-only implementation
    // Admin notification routes (require admin permissions)
    // .route("/notifications/send", post(admin_send_notification))
    // .route("/notifications/broadcast", post(admin_broadcast_to_topic))
    // .route("/notifications/stats", get(admin_get_notification_stats))
    // .route("/notifications/list", get(admin_get_user_notifications))
    // .route("/notifications/recent", get(admin_get_user_notifications))
    // .route("/notifications/history", get(admin_get_user_notifications))
    // .route("/notifications/unread", get(admin_get_user_notifications))
    // .route("/notifications/:id/read", put(admin_mark_notification_read))
    // .route("/notifications/:id", delete(admin_delete_notification))
    // Security monitoring routes (require admin:security:* permissions)
    .route(
      "/security/events",
      get(SecurityMonitoringHandlers::get_security_events)
    )
    .route(
      "/security/metrics",
      get(SecurityMonitoringHandlers::get_security_metrics)
    )
    .route(
      "/security/user-threat",
      get(SecurityMonitoringHandlers::get_user_threat_assessment)
    )
    // Dynamic Plan Management routes (require admin:plans:* permissions) - Simplified
    .route("/plans", get(list_plans_handler))
    .route("/plans", post(create_plan_handler))
    .route("/plans/:plan_id", get(get_plan_handler))
    // Subscription Management routes (require admin:subscriptions:* permissions) - Simplified
    .route("/subscriptions", post(create_subscription_handler))
    // Performance monitoring routes (require admin:performance:* permissions)
    .route("/performance/auth-cache", get(get_auth_cache_performance))
    .route("/performance/cache-summary", get(get_cache_summary))
    .route("/performance/clear-cache", post(clear_auth_cache))
    // Web3 Permission Management routes (require admin:web3:* permissions)
    .route("/web3/permissions", get(get_user_permissions))
    .route("/web3/permissions/grant", post(grant_manual_permission))
    .route("/web3/nft-gates", get(get_nft_gates))
    .route("/web3/nft-gates", post(create_nft_gate))
    .route("/web3/token-gates", get(get_token_gates))
    .route("/web3/token-gates", post(create_token_gate))
    .route("/web3/dao-proposals", get(get_dao_proposals))
    .route("/web3/dao-proposals", post(create_dao_proposal))
    .route("/web3/recent-wallets", get(get_recent_wallets))
    .route("/wallets/search", get(search_wallets))

    // ============================================================================
    // BACKEND-CENTRIC PERMISSION AUTHORITY SYSTEM (THE SINGLE SOURCE OF TRUTH)
    // These endpoints are THE AUTHORITY for all permission decisions
    // Frontend and admin apps consume these APIs and handle only error responses
    // ============================================================================
    
    // Unified Permission Group Management (wallet-first system)
    .route("/permission-groups", get(list_permission_groups_handler))
    .route("/permission-groups", post(create_permission_group_handler))
    .route("/permission-groups/:group_id", get(get_permission_group_handler))
    .route("/permission-groups/:group_id", put(update_permission_group_handler))
    .route("/permission-groups/:group_id", delete(delete_permission_group_handler))
    
    // Wallet Assignment Management (wallet-first system)
    .route("/wallet-assignments", post(create_wallet_assignment_handler))
    .route("/wallets/:wallet_address/assignments", get(get_wallet_assignments_handler))

    // ============================================================================
    // CONSOLIDATED USER MANAGEMENT SYSTEM
    // Backend-centric user operations with comprehensive data and analytics
    // ============================================================================
    
    // User Management routes (require admin:users:* permissions)
    .route("/users", get(list_users_handler))
    .route("/users/stats", get(get_user_stats_handler))
    .route("/users/:wallet_address", get(get_user_handler))
    .route("/users/:wallet_address", put(update_user_handler))

    // ============================================================================
    // ANALYTICS AND BUSINESS INTELLIGENCE SYSTEM
    // Data aggregation and insights for administrative decision making
    // ============================================================================
    
    // Analytics routes (require admin:analytics:* permissions)
    .route("/analytics/overview", get(get_platform_overview_handler))
    .route("/analytics/users", get(get_user_analytics_handler))
    .route("/analytics/permissions", get(get_permission_analytics_handler))
    .route("/analytics/revenue", get(get_revenue_analytics_handler))

    // TODO: Temporarily disabled due to Axum trait bound issues
    // .layer(axum::middleware::from_fn(crate::web::middleware::web3_auth_middleware))
}

pub fn create_admin_public_routes() -> Router<AppState> {
  Router::new()
    // Public admin authentication routes - handler missing
    // .route("/auth/login", post(login_handler))
    .route(
      "/health",
      get(|| async { "OK" })
    )
}

// ============================================================================
// PERMISSION AUTHORITY ROUTES (THE SINGLE SOURCE OF TRUTH)
// These routes are accessible to ALL applications (frontend, admin, external APIs)
// and provide THE AUTHORITATIVE permission decisions
// ============================================================================

pub fn create_permission_authority_routes() -> Router<AppState> {
  Router::new()
    // ⚡ CRITICAL: Real-time permission validation - THE AUTHORITY
    // This endpoint is called by frontend/admin for ALL permission checks
    .route("/api/permissions/validate", post(validate_permission_handler))
    
    // ⚡ CRITICAL: Bulk permission validation for performance
    // Used by frontend/admin for batch permission checking
    .route("/api/permissions/validate-bulk", post(validate_bulk_permissions_handler))
    
    // ⚡ CRITICAL: Wallet's effective permissions - what they can actually do
    // Used by frontend/admin to understand wallet capabilities
    .route("/api/permissions/wallet/:wallet_address", get(get_wallet_permissions_handler))
    
    // Apply authentication middleware to permission authority routes
    // TODO: Temporarily disabled due to Axum trait bound issues
    // .layer(axum::middleware::from_fn(crate::web::middleware::web3_auth_middleware))
}
