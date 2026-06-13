// Admin module for user management endpoints

pub mod routes;
pub mod setup_handlers;
pub mod security_monitoring_handlers;
pub mod plans;

pub use plans as plan_handlers;
pub mod promotion_handlers;
pub mod performance_handlers;
pub mod auth_handlers;
pub mod responses;
pub mod wallet_management_handlers;
pub mod wallet_disable_handlers;
pub mod analytics;

pub mod notification_handlers;
pub mod notification_query_helper;
pub mod wallet_notification_repository;
pub mod system_settings_handlers;
pub mod developer_portal_handlers;
// wave11(track-b): `payment_link_handlers` moved to
// `crate::web::payments::payment_link_handlers` — see
// `web/payments/payment_link_handlers.rs` for the docstring
// and the audit references. The route is still mounted from
// `web/admin/routes.rs` but the handlers themselves live in
// the payments area.
pub mod audit_log_handlers;
pub mod chat_handlers;
pub mod news_handlers;
pub mod media_handlers;
pub mod batch_handlers;

// Consolidated permission module (v3.0) - replaces 5 handler files (3,743 lines)
pub mod permissions;

pub use routes::{create_admin_routes, create_admin_public_routes, create_permission_authority_routes};