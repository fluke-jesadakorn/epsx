// Admin routes configuration

use axum::{ routing::{ get, post, put, delete }, Router };

// Security monitoring handlers
use super::security_monitoring_handlers::{ SecurityMonitoringHandlers };
// Dynamic plan management handlers (simplified)
use super::plan_handlers::{
  create_plan_handler,
  get_plan_handler,
  list_plans_handler,
  update_plan_handler,
  create_subscription_handler,
};
// Promotion management handlers
use super::promotion_handlers::{
  create_promotion_handler,
  get_promotion_handler,
  list_promotions_handler,
  update_promotion_handler,
  delete_promotion_handler,
};
// Consolidated permission module - all permission operations
use super::permissions::{
  // Group CRUD operations
  create_group,
  get_group,
  list_groups,
  update_group,
  delete_group,
  get_group_members,
  // Assignment management
  create_assignment,
  list_assignments,
  remove_assignment,
  get_expiring_assignments,
  get_assignment_history,
  get_wallet_groups,
  // Validation operations
  validate_permission,
  validate_bulk_permissions,
  get_wallet_permissions,
  // Direct permission management
  grant_permission,
  revoke_permission,
  list_wallet_permissions,
  add_permission_to_group,
  remove_permission_from_group,
  // Bulk operations
  bulk_grant,
  bulk_revoke,
  bulk_assign_roles,
  bulk_apply_template,
  bulk_validate,
  // System operations
  get_health,
  get_statistics,
  clear_caches,
  get_route_permissions,
  register_route_permission,
};
// Performance monitoring handlers
use super::performance_handlers::{
  get_auth_cache_performance,
  get_cache_summary,
  clear_auth_cache,
};
// Web3 permission management handlers
use super::auth_handlers::{
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
  get_tiers,
};
// Consolidated user management handlers
use super::wallet_management_handlers::{
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
// Notification management handlers
use super::notification_handlers::{
  send_notification_handler,
  get_all_notifications_handler,
  get_notification_stats_handler,
  acknowledge_notification_handler,
  delete_admin_notification_handler,
};
use crate::web::auth::AppState;

pub fn create_admin_routes() -> Router<AppState> {
  Router::new()
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
    .route("/plans/:plan_id", put(update_plan_handler))
    // Promotion Management routes (require admin:promotions:* permissions)
    .route("/promotions", get(list_promotions_handler))
    .route("/promotions", post(create_promotion_handler))
    .route("/promotions/:id", get(get_promotion_handler))
    .route("/promotions/:id", put(update_promotion_handler))
    .route("/promotions/:id", delete(delete_promotion_handler))
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
    .route("/tiers", get(get_tiers))

    // ============================================================================
    // BACKEND-CENTRIC PERMISSION AUTHORITY SYSTEM (THE SINGLE SOURCE OF TRUTH)
    // These endpoints are THE AUTHORITY for all permission decisions
    // Frontend and admin apps consume these APIs and handle only error responses
    // ============================================================================
    
    // ============================================================================
    // CONSOLIDATED PERMISSION MODULE ROUTES
    // Using new consolidated permission module structure
    // ============================================================================

    // Permission Group Management (CRUD)
    .route("/permissions/groups", get(list_groups).post(create_group))
    .route("/permissions/groups/:group_id", get(get_group).put(update_group).delete(delete_group))
    .route("/permissions/groups/:group_id/members", get(get_group_members))

    // Wallet-Group Assignment Management
    .route("/permissions/assignments", get(list_assignments).post(create_assignment))
    .route("/permissions/assignments/:assignment_id", delete(remove_assignment))
    .route("/permissions/assignments/expiring", get(get_expiring_assignments))
    .route("/permissions/assignments/history/:wallet", get(get_assignment_history))
    .route("/permissions/wallets/:wallet/groups", get(get_wallet_groups))

    // Permission Validation
    .route("/permissions/validate", post(validate_permission))
    .route("/permissions/validate/bulk", post(validate_bulk_permissions))
    .route("/permissions/wallets/:wallet/permissions", get(get_wallet_permissions))

    // Direct Permission Management
    .route("/permissions/direct", post(grant_permission).delete(revoke_permission))
    .route("/permissions/direct/:wallet", get(list_wallet_permissions))
    .route("/permissions/groups/:group_id/permissions", post(add_permission_to_group))
    .route("/permissions/groups/:group_id/permissions/:permission_id", delete(remove_permission_from_group))

    // Bulk Operations
    .route("/permissions/bulk/grant", post(bulk_grant))
    .route("/permissions/bulk/revoke", post(bulk_revoke))
    .route("/permissions/bulk/assign-roles", post(bulk_assign_roles))
    .route("/permissions/bulk/apply-template", post(bulk_apply_template))
    .route("/permissions/bulk/validate", post(bulk_validate))

    // System Operations
    .route("/permissions/system/health", get(get_health))
    .route("/permissions/system/stats", get(get_statistics))
    .route("/permissions/system/cache/clear", post(clear_caches))
    .route("/permissions/system/routes", get(get_route_permissions).post(register_route_permission))
    // ============================================================================
    // CONSOLIDATED WALLET MANAGEMENT SYSTEM
    // Backend-centric wallet operations with comprehensive data and analytics
    // ============================================================================

    // Wallet Management routes (require admin:wallets:* permissions)
    .route("/wallets", get(list_users_handler))
    .route("/wallets/stats", get(get_user_stats_handler))
    .route("/wallets/:wallet_address", get(get_user_handler))
    .route("/wallets/:wallet_address", put(update_user_handler))

    // ============================================================================
    // ANALYTICS AND BUSINESS INTELLIGENCE SYSTEM
    // Data aggregation and insights for administrative decision making
    // ============================================================================
    
    // Analytics routes (require admin:analytics:* permissions)
    .route("/analytics/overview", get(get_platform_overview_handler))
    .route("/analytics/users", get(get_user_analytics_handler))
    .route("/analytics/permissions", get(get_permission_analytics_handler))
    .route("/analytics/revenue", get(get_revenue_analytics_handler))

    // CQRS-based admin analytics endpoints (from analytics module)
    .route("/analytics/metrics", get(crate::web::analytics::system_metrics_handler))
    .route("/analytics/time-series", get(crate::web::analytics::admin_time_series_handler))
    .route("/analytics/modules", get(crate::web::analytics::admin_modules_handler))

    // ============================================================================
    // NOTIFICATION MANAGEMENT SYSTEM
    // Admin notification sending and statistics
    // ============================================================================

    // Notification routes (require admin:notifications:* permissions)
    .route("/notifications/send", post(send_notification_handler))
    .route("/notifications", get(get_all_notifications_handler))
    .route("/notifications/stats", get(get_notification_stats_handler))
    .route("/notifications/:id/acknowledge", put(acknowledge_notification_handler))
    .route("/notifications/:id", delete(delete_admin_notification_handler))

    // TODO: Temporarily disabled due to Axum trait bound issues
    // .layer(axum::middleware::from_fn(crate::web::middleware::web3_auth_middleware))
}

pub fn create_admin_public_routes() -> Router<AppState> {
  Router::new()
    // Public admin authentication routes - handler missing
    // .route("/auth/login", post(login_handler))
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
    .route("/api/permissions/validate", post(validate_permission))

    // ⚡ CRITICAL: Bulk permission validation for performance
    // Used by frontend/admin for batch permission checking
    .route("/api/permissions/validate-bulk", post(validate_bulk_permissions))

    // ⚡ CRITICAL: Wallet's effective permissions - what they can actually do
    // Used by frontend/admin to understand wallet capabilities
    .route("/api/permissions/wallet/:wallet_address", get(get_wallet_permissions))

    // Apply authentication middleware to permission authority routes
    // TODO: Temporarily disabled due to Axum trait bound issues
    // .layer(axum::middleware::from_fn(crate::web::middleware::web3_auth_middleware))
}
