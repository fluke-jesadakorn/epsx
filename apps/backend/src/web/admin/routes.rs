// Admin routes configuration

use axum::{ routing::{ get, post, put, patch, delete }, Router, middleware::from_fn_with_state };

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
  get_admin_analytics_dashboard_handler,
};

// Notification management handlers
use super::notification_handlers::{
  send_notification_handler,
  get_all_notifications_handler,
  get_notification_stats_handler,
  acknowledge_notification_handler,
  delete_admin_notification_handler,
  upload_notification_image,
};
// System settings handlers
// use super::system_settings_handlers::{
//   get_all_settings_handler,
//   get_settings_by_category_handler,
//   update_settings_handler,
//   reset_settings_handler,
// };
use super::batch_handlers::{
    admin_dashboard_summary_handler,
    admin_notification_overview_handler,
    wallet_access_summary_handler,
};
use crate::web::auth::AppState;

pub fn create_admin_routes() -> Router<AppState> {
  use crate::web::middleware::perm_guard;

  // Dashboard identity check and batch overview
  let dashboard = Router::new()
    .route("/me", get(super::setup_handlers::admin_me_handler))
    .route("/dashboard/summary", get(admin_dashboard_summary_handler))
    .layer(from_fn_with_state("admin:dashboard:view", perm_guard));

  // Security monitoring
  let security = Router::new()
    .route("/security/events", get(SecurityMonitoringHandlers::get_security_events))
    .route("/security/metrics", get(SecurityMonitoringHandlers::get_security_metrics))
    .route("/security/user-threat", get(SecurityMonitoringHandlers::get_user_threat_assessment))
    .layer(from_fn_with_state("admin:security:read", perm_guard));

  // Plan management — read
  let plans_read = Router::new()
    .route("/plans", get(list_plans_handler))
    .route("/plans/{plan_id}", get(get_plan_handler))
    .route("/subscriptions", get(list_subscriptions_handler))
    .route("/plans/user-access/list", get(admin_list_user_access_handler))
    .route("/plans/history", get(get_plan_history))
    .layer(from_fn_with_state("admin:plans:read", perm_guard));

  // Plan management — write
  let plans_write = Router::new()
    .route("/plans", post(create_plan_handler))
    .route("/plans/{plan_id}", put(update_plan_handler).delete(delete_plan_handler))
    .route("/subscriptions", post(create_subscription_handler))
    .route("/plans/seed", post(crate::web::public::seed_plans_handler::seed_subscription_plans))
    .layer(from_fn_with_state("admin:plans:manage", perm_guard));

  // Promotion management
  let promotions = Router::new()
    .route("/promotions", get(list_promotions_handler).post(create_promotion_handler))
    .route("/promotions/{id}", get(get_promotion_handler).put(update_promotion_handler).delete(delete_promotion_handler))
    .layer(from_fn_with_state("admin:promotions:manage", perm_guard));

  // Performance monitoring
  let performance = Router::new()
    .route("/performance/auth-cache", get(get_auth_cache_performance))
    .route("/performance/cache-summary", get(get_cache_summary))
    .route("/performance/clear-cache", post(clear_auth_cache))
    .layer(from_fn_with_state("admin:performance:view", perm_guard));

  // Permission system — read
  let permissions_read = Router::new()
    .route("/permissions", get(get_user_permissions))
    .route("/web3/permissions", get(get_user_permissions))
    .route("/permissions/system/health", get(get_health))
    .route("/permissions/system/stats", get(get_statistics))
    .route("/permissions/system/routes", get(get_route_permissions))
    .route("/permissions/available", get(super::permissions::list_available_permissions))
    .route("/permissions/direct/wallet/{wallet}", get(list_wallet_permissions))
    .route("/web3/nft-gates", get(get_nft_gates))
    .route("/web3/token-gates", get(get_token_gates))
    .route("/web3/dao-proposals", get(get_dao_proposals))
    .route("/web3/recent-wallets", get(get_recent_wallets))
    .route("/wallets/search", get(search_wallets))
    .route("/tiers", get(get_tiers))
    .layer(from_fn_with_state("admin:permissions:read", perm_guard));

  // Permission system — write
  let permissions_write = Router::new()
    .route("/web3/permissions/grant", post(grant_manual_permission))
    .route("/permissions/grant", post(grant_manual_permission))
    .route("/web3/nft-gates", post(create_nft_gate))
    .route("/web3/token-gates", post(create_token_gate))
    .route("/web3/dao-proposals", post(create_dao_proposal))
    .route("/permissions/system/cache/clear", post(clear_caches))
    .route("/permissions/system/routes", post(register_route_permission))
    .route("/permissions/direct/grant", post(grant_permission))
    .route("/permissions/direct/revoke", delete(revoke_permission))
    .route("/permissions/plans/{plan_id}/permissions", post(add_permission_to_plan))
    .route("/permissions/plans/{plan_id}/permissions/{permission_id}", delete(remove_permission_from_plan))
    .route("/permissions/bulk/grant", post(bulk_grant))
    .route("/permissions/bulk/revoke", post(bulk_revoke))
    .route("/permissions/bulk/assign-plans", post(bulk_assign_plans))
    .route("/permissions/bulk/apply-template", post(bulk_apply_template))
    .route("/permissions/bulk/validate", post(bulk_validate))
    .layer(from_fn_with_state("admin:permissions:manage", perm_guard));

  // Wallet management — read
  let wallets_read = Router::new()
    .route("/wallets", get(list_users_handler))
    .route("/wallets/stats", get(get_user_stats_handler))
    .route("/wallets/{wallet_address}", get(get_user_handler))
    .route("/wallets/{wallet_address}/activity", get(get_wallet_activity_handler))
    .route("/wallets/{wallet_address}/access-summary", get(wallet_access_summary_handler))
    .layer(from_fn_with_state("admin:users:read", perm_guard));

  // Wallet management — write
  let wallets_write = Router::new()
    .route("/wallets/{wallet_address}", put(update_user_handler))
    .route("/wallets/{wallet_address}/disable", post(super::wallet_management_handlers::disable_user_handler))
    .route("/wallets/{wallet_address}/enable", post(super::wallet_management_handlers::enable_user_handler))
    .layer(from_fn_with_state("admin:users:update", perm_guard));

  // Analytics and audit
  let analytics = Router::new()
    .route("/analytics/overview", get(get_platform_overview_handler))
    .route("/analytics/users", get(get_user_analytics_handler))
    .route("/analytics/permissions", get(get_permission_analytics_handler))
    .route("/analytics/revenue", get(get_revenue_analytics_handler))
    .route("/analytics/usage", get(get_usage_analytics_handler))
    .route("/analytics/dashboard", get(get_admin_analytics_dashboard_handler))
    .route("/analytics/metrics", get(crate::web::analytics::system_metrics_handler))
    .route("/analytics/time-series", get(crate::web::analytics::admin_time_series_handler))
    .route("/analytics/modules", get(crate::web::analytics::admin_modules_handler))
    .route("/audit-logs", get(super::audit_log_handlers::get_audit_logs_handler))
    .layer(from_fn_with_state("admin:analytics:view", perm_guard));

  // Notification management
  let notifications = Router::new()
    .route("/notifications/send", post(send_notification_handler))
    .route("/notifications", get(get_all_notifications_handler))
    .route("/notifications/stats", get(get_notification_stats_handler))
    .route("/notifications/{id}/acknowledge", put(acknowledge_notification_handler))
    .route("/notifications/{id}", delete(delete_admin_notification_handler))
    .route("/notifications/overview", get(admin_notification_overview_handler))
    .route("/notifications/upload-image", post(upload_notification_image))
    .layer(from_fn_with_state("admin:notifications:manage", perm_guard));

  // Developer portal
  let developer = Router::new()
    .route("/developer-portal/api-keys", get(super::developer_portal_handlers::list_api_keys_handler).post(super::developer_portal_handlers::create_api_key_handler))
    .route("/developer-portal/api-keys/{id}", get(super::developer_portal_handlers::get_api_key_handler))
    .route("/developer-portal/api-keys/{id}/revoke", post(super::developer_portal_handlers::revoke_api_key_handler))
    .route("/developer-portal/api-keys/{id}/expiration", patch(super::developer_portal_handlers::update_expiration_handler))
    .route("/developer-portal/api-keys/expiring", get(super::developer_portal_handlers::list_expiring_keys_handler))
    .route("/developer-portal/modules", get(super::developer_portal_handlers::list_modules_handler).post(super::developer_portal_handlers::create_module_handler))
    .route("/developer-portal/modules/{id}", get(super::developer_portal_handlers::get_module_handler).put(super::developer_portal_handlers::update_module_handler))
    .route("/developer-portal/stats", get(super::developer_portal_handlers::get_stats_handler))
    .layer(from_fn_with_state("admin:developer:manage", perm_guard));

  // Payment links
  //
  // wave11(track-b): the handlers live at
  // `crate::web::payments::payment_link_handlers` now (folded
  // out of `web::admin::payment_link_handlers`). The route
  // mount and the perm-guard layer stay here because the admin
  // scope is what gates the writes; the public slug lookup
  // (`/api/public/payment-links/{slug}`) is mounted separately
  // from `unified_router::create_public_routes` and uses no
  // perm guard.
  let payment_links = Router::new()
    .route("/payment-links", get(crate::web::payments::payment_link_handlers::list_payment_links_handler).post(crate::web::payments::payment_link_handlers::create_payment_link_handler))
    .route("/payment-links/{id}", get(crate::web::payments::payment_link_handlers::get_payment_link_handler).put(crate::web::payments::payment_link_handlers::update_payment_link_handler).delete(crate::web::payments::payment_link_handlers::delete_payment_link_handler))
    .route("/payment-links/{id}/record-usage", post(crate::web::payments::payment_link_handlers::record_payment_usage_handler))
    .layer(from_fn_with_state("admin:payments:manage", perm_guard));

  // Support chat
  let chat = Router::new()
    .route("/chat/topics", get(super::chat_handlers::admin_list_topics))
    .route("/chat/conversations", get(super::chat_handlers::admin_list_conversations))
    .route("/chat/conversations/{id}", get(super::chat_handlers::admin_get_conversation))
    .route("/chat/conversations/{id}/messages", get(super::chat_handlers::admin_list_messages).post(super::chat_handlers::admin_send_reply))
    .route("/chat/conversations/{id}/upload", post(crate::web::user::chat_upload_handlers::admin_upload_attachment))
    .route("/chat/conversations/{id}/typing", post(crate::web::user::chat_upload_handlers::admin_typing))
    .route("/chat/conversations/{id}/assign", put(super::chat_handlers::admin_assign_agent))
    .route("/chat/conversations/{id}/status", put(super::chat_handlers::admin_update_status))
    .route("/chat/conversations/{id}/read", put(super::chat_handlers::admin_mark_read))
    .route("/chat/stats", get(super::chat_handlers::admin_get_stats))
    .route("/chat/overview", get(super::chat_handlers::admin_chat_overview_handler))
    .layer(from_fn_with_state("admin:chat:manage", perm_guard));

  // News / content management
  let news = Router::new()
    .route("/news", get(super::news_handlers::list_news).post(super::news_handlers::create_news))
    .route("/news/upload-image", post(super::news_handlers::upload_news_image))
    .route("/news/{id}", get(super::news_handlers::get_news).put(super::news_handlers::update_news).delete(super::news_handlers::delete_news))
    .route("/news/{id}/publish", put(super::news_handlers::publish_news))
    .route("/news/{id}/unpublish", put(super::news_handlers::unpublish_news))
    .route("/news/{id}/pin", put(super::news_handlers::pin_news))
    .route("/news/{id}/unpin", put(super::news_handlers::unpin_news))
    .layer(from_fn_with_state("admin:content:manage", perm_guard));

  // Media and file management
  let media = Router::new()
    .route("/files/upload", post(super::media_handlers::upload_public_file))
    .route("/files", get(super::media_handlers::list_public_files))
    .route("/files/{key}", delete(super::media_handlers::delete_public_file))
    .route("/media/{bucket}", get(super::media_handlers::list_media))
    .route("/media/{bucket}/{key}", delete(super::media_handlers::delete_media))
    .layer(from_fn_with_state("admin:media:manage", perm_guard));

  dashboard
    .merge(security)
    .merge(plans_read)
    .merge(plans_write)
    .merge(promotions)
    .merge(performance)
    .merge(permissions_read)
    .merge(permissions_write)
    .merge(wallets_read)
    .merge(wallets_write)
    .merge(analytics)
    .merge(notifications)
    .merge(developer)
    .merge(payment_links)
    .merge(chat)
    .merge(news)
    .merge(media)
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
    use crate::web::middleware::perm_guard;

    // Public validation — any authenticated user can validate permissions
    let validate_routes = Router::new()
        .route("/validate", post(validate_permission))
        .route("/validate-bulk", post(validate_bulk_permissions))
        .route("/available", get(super::permissions::list_available_permissions));

    // Admin read — requires admin:permissions:read
    let admin_read = Router::new()
        .route("/wallet/{wallet_address}", get(get_wallet_permissions))
        .route("/plans", get(list_plans))
        .route("/plans/{plan_id}", get(get_plan))
        .route("/plans/{plan_id}/members", get(get_plan_members))
        .route("/plans/{plan_id}/permissions", get(get_plan_permissions))
        .route("/assignments", get(list_assignments))
        .route("/assignments/expiring", get(get_expiring_assignments))
        .route("/assignments/wallet/{wallet}", get(get_assignment_history))
        .route("/assignments/plan/{plan_id}", get(get_plan_assignments))
        .route("/definitions", get(super::permissions::list_permission_definitions))
        .layer(from_fn_with_state("admin:permissions:read", perm_guard));

    // Admin write — requires admin:permissions:manage
    let admin_write = Router::new()
        .route("/plans", post(create_plan))
        .route("/plans/{plan_id}", put(update_plan).delete(delete_plan))
        .route("/assignments", post(create_assignment))
        .route("/assignments/{assignment_id}", delete(remove_assignment))
        .route("/definitions", post(super::permissions::create_permission_definition))
        .route("/definitions/{id}", delete(super::permissions::delete_permission_definition))
        .route("/definitions/by-name/{permission}", delete(super::permissions::delete_permission_by_name))
        .layer(from_fn_with_state("admin:permissions:manage", perm_guard));

    validate_routes.merge(admin_read).merge(admin_write)
}
