// Admin routes configuration

use axum::{ routing::{ get, post, put, patch, delete }, Router };

// Security monitoring handlers
use super::security_monitoring_handlers::{ SecurityMonitoringHandlers };
// Dynamic plan management handlers (simplified)
use super::plan_handlers::{
  create_plan_handler,
  get_plan_handler,
  list_plans_handler,
  update_plan_handler,
  delete_plan_handler,
  create_subscription_handler,
  admin_list_user_access_handler,
  list_subscriptions_handler,
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
  // Plan CRUD operations
  create_plan,
  get_plan,
  list_plans,
  update_plan,
  delete_plan,
  get_plan_members,
  // Assignment management
  create_assignment,
  list_assignments,
  remove_assignment,
  get_expiring_assignments,
  get_assignment_history,
  get_plan_history,
  // Validation operations
  validate_permission,
  validate_bulk_permissions,
  get_wallet_permissions,
  // Direct permission management
  grant_permission,
  revoke_permission,
  list_wallet_permissions,
  add_permission_to_plan,
  remove_permission_from_plan,
  // Bulk operations
  bulk_grant,
  bulk_revoke,
  bulk_assign_plans,
  bulk_apply_template,
  bulk_validate,
  // System operations
  get_health,
  get_statistics,
  clear_caches,
  get_route_permissions,
  register_route_permission,
  // Plan permission and assignment queries
  get_plan_permissions,
  get_plan_assignments,
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
// Wallet disable/enable and activity handlers
use super::wallet_disable_handlers::{
  get_wallet_activity_handler,
};
// Analytics and business intelligence handlers
use super::analytics::{
  get_platform_overview_handler,
  get_user_analytics_handler,
  get_permission_analytics_handler,
  get_revenue_analytics_handler,
  get_usage_analytics_handler,
};

// Notification management handlers
use super::notification_handlers::{
  send_notification_handler,
  get_all_notifications_handler,
  get_notification_stats_handler,
  acknowledge_notification_handler,
  delete_admin_notification_handler,
};
// System settings handlers
// use super::system_settings_handlers::{
//   get_all_settings_handler,
//   get_settings_by_category_handler,
//   update_settings_handler,
//   reset_settings_handler,
// };
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
    .route("/plans/{plan_id}", get(get_plan_handler).put(update_plan_handler).delete(delete_plan_handler))
    // Promotion Management routes (require admin:promotions:* permissions)
    .route("/promotions", get(list_promotions_handler))
    .route("/promotions", post(create_promotion_handler))
    .route("/promotions/{id}", get(get_promotion_handler))
    .route("/promotions/{id}", put(update_promotion_handler))
    .route("/promotions/{id}", delete(delete_promotion_handler))
    // Subscription Management routes (require admin:subscriptions:* permissions) - Simplified
    .route("/subscriptions", get(list_subscriptions_handler).post(create_subscription_handler))
    // Direct Payment Model: User Access List (replaces subscription-based model)
    .route("/plans/user-access/list", get(admin_list_user_access_handler))
    // Performance monitoring routes (require admin:performance:* permissions)
    .route("/performance/auth-cache", get(get_auth_cache_performance))
    .route("/performance/cache-summary", get(get_cache_summary))
    .route("/performance/clear-cache", post(clear_auth_cache))
    // Permission Management routes (require admin:web3:* permissions)
    .route("/web3/permissions", get(get_user_permissions))
    .route("/web3/permissions/grant", post(grant_manual_permission))
    // New simplified routes (aliased to above)
    .route("/permissions", get(get_user_permissions))
    .route("/permissions/grant", post(grant_manual_permission))
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
    // ADMIN-SPECIFIC PERMISSION OPERATIONS
    // Core permission operations moved to /api/permissions/* (accessible by all apps)
    // ============================================================================

    // Admin-only permission system operations
    .route("/permissions/system/health", get(get_health))
    .route("/permissions/system/stats", get(get_statistics))
    .route("/permissions/system/cache/clear", post(clear_caches))
    .route("/permissions/system/routes", get(get_route_permissions).post(register_route_permission))

    // List all available unique permission strings (matches frontend call)
    .route("/permissions/available", get(super::permissions::list_available_permissions))

    // Plan Assignment History (Audit Log)
    .route("/plans/history", get(get_plan_history))

    // Admin-only direct permission management (elevated privileges)
    .route("/permissions/direct/grant", post(grant_permission))
    .route("/permissions/direct/revoke", delete(revoke_permission))
    .route("/permissions/direct/wallet/{wallet}", get(list_wallet_permissions))
    .route("/permissions/plans/{plan_id}/permissions", post(add_permission_to_plan))
    .route("/permissions/plans/{plan_id}/permissions/{permission_id}", delete(remove_permission_from_plan))

    // Admin-only bulk operations
    .route("/permissions/bulk/grant", post(bulk_grant))
    .route("/permissions/bulk/revoke", post(bulk_revoke))
    .route("/permissions/bulk/assign-plans", post(bulk_assign_plans))
    .route("/permissions/bulk/apply-template", post(bulk_apply_template))
    .route("/permissions/bulk/validate", post(bulk_validate))

    // Admin-specific permission analytics - see /analytics/permissions endpoint
    // ============================================================================
    // CONSOLIDATED WALLET MANAGEMENT SYSTEM
    // Backend-centric wallet operations with comprehensive data and analytics
    // ============================================================================

    // Wallet Management routes (require admin:wallets:* permissions)
    .route("/wallets", get(list_users_handler))
    .route("/wallets/stats", get(get_user_stats_handler))
    .route("/wallets/{wallet_address}", get(get_user_handler))
    .route("/wallets/{wallet_address}", put(update_user_handler))
    // Wallet disable/enable operations
    .route("/wallets/{wallet_address}/disable", post(super::wallet_management_handlers::disable_user_handler))
    .route("/wallets/{wallet_address}/enable", post(super::wallet_management_handlers::enable_user_handler))
    .route("/wallets/{wallet_address}/activity", get(get_wallet_activity_handler))

    // ============================================================================
    // ANALYTICS AND BUSINESS INTELLIGENCE SYSTEM
    // Data aggregation and insights for administrative decision making
    // ============================================================================
    
    // Analytics routes (require admin:analytics:* permissions)
    .route("/analytics/overview", get(get_platform_overview_handler))
    .route("/analytics/users", get(get_user_analytics_handler))
    .route("/analytics/permissions", get(get_permission_analytics_handler))
    .route("/analytics/revenue", get(get_revenue_analytics_handler))
    .route("/analytics/usage", get(get_usage_analytics_handler))
    .route("/audit-logs", get(super::audit_log_handlers::get_audit_logs_handler))

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
    .route("/notifications/{id}/acknowledge", put(acknowledge_notification_handler))
    .route("/notifications/{id}", delete(delete_admin_notification_handler))

    // ============================================================================
    // SYSTEM SETTINGS MANAGEMENT
    // Global admin console settings (NOT tied to any wallet)
    // ============================================================================

    // Settings routes (require admin:settings:* permissions)
    // MOVED to unified_router.rs as public routes for development
    /*
    .route("/settings", get(get_all_settings_handler).put(update_settings_handler))
    .route("/settings/reset", post(reset_settings_handler))
    .route("/settings/{category}", get(get_settings_by_category_handler))
    */

    // ============================================================================
    // DEVELOPER PORTAL MANAGEMENT
    // API key and module management for third-party integrations
    // ============================================================================

    // Developer Portal routes (require admin:developer:* permissions)
    .route("/developer-portal/api-keys", get(super::developer_portal_handlers::list_api_keys_handler).post(super::developer_portal_handlers::create_api_key_handler))
    .route("/developer-portal/api-keys/{id}", get(super::developer_portal_handlers::get_api_key_handler))
    .route("/developer-portal/api-keys/{id}/revoke", post(super::developer_portal_handlers::revoke_api_key_handler))
    .route("/developer-portal/api-keys/{id}/expiration", patch(super::developer_portal_handlers::update_expiration_handler))
    .route("/developer-portal/api-keys/expiring", get(super::developer_portal_handlers::list_expiring_keys_handler))
    .route("/developer-portal/modules", get(super::developer_portal_handlers::list_modules_handler).post(super::developer_portal_handlers::create_module_handler))
    .route("/developer-portal/modules/{id}", get(super::developer_portal_handlers::get_module_handler).put(super::developer_portal_handlers::update_module_handler))
    .route("/developer-portal/stats", get(super::developer_portal_handlers::get_stats_handler))

    // ============================================================================
    // PAYMENT LINK MANAGEMENT (V2 Dynamic Payments)
    // Dynamic payment links for plans, plans, products, campaigns, and custom
    // ============================================================================

    // Payment Link routes (require admin:payments:* permissions)
    .route("/payment-links", get(super::payment_link_handlers::list_payment_links_handler).post(super::payment_link_handlers::create_payment_link_handler))
    .route("/payment-links/{id}", get(super::payment_link_handlers::get_payment_link_handler).put(super::payment_link_handlers::update_payment_link_handler).delete(super::payment_link_handlers::delete_payment_link_handler))
    .route("/payment-links/{id}/record-usage", post(super::payment_link_handlers::record_payment_usage_handler))

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
    // CRITICAL: Real-time permission validation - THE AUTHORITY
    // This endpoint is called by frontend/admin for ALL permission checks
    // Route: /api/permissions/validate
    .route("/validate", post(validate_permission))

    // CRITICAL: Bulk permission validation for performance
    // Used by frontend/admin for batch permission checking
    // Route: /api/permissions/validate-bulk
    .route("/validate-bulk", post(validate_bulk_permissions))

    // List all available unique permission strings
    // Route: /api/permissions/available
    .route("/available", get(super::permissions::list_available_permissions))

    // Permission Definitions Management (CRUD for custom permissions)
    // Route: /api/permissions/definitions
    .route("/definitions", get(super::permissions::list_permission_definitions).post(super::permissions::create_permission_definition))
    .route("/definitions/{id}", delete(super::permissions::delete_permission_definition))
    .route("/definitions/by-name/{permission}", delete(super::permissions::delete_permission_by_name))

    // CRITICAL: Wallet's effective permissions - what they can actually do
    // Used by frontend/admin to understand wallet capabilities
    // Route: /api/permissions/wallet/{wallet_address}
    .route("/wallet/{wallet_address}", get(get_wallet_permissions))

    // Permission Plan Management (accessible by all apps)
    // Route: /api/permissions/plans
    .route("/plans", get(list_plans).post(create_plan))
    .route("/plans/{plan_id}", get(get_plan).put(update_plan).delete(delete_plan))
    .route("/plans/{plan_id}/members", get(get_plan_members))
    .route("/plans/{plan_id}/permissions", get(get_plan_permissions))

    // Assignment Management (accessible by all apps)
    // Route: /api/permissions/assignments
    .route("/assignments", get(list_assignments).post(create_assignment))
    .route("/assignments/{assignment_id}", delete(remove_assignment))
    .route("/assignments/expiring", get(get_expiring_assignments))
    .route("/assignments/wallet/{wallet}", get(get_assignment_history))
    .route("/assignments/plan/{plan_id}", get(get_plan_assignments))

    // Apply authentication middleware to permission authority routes
    }
