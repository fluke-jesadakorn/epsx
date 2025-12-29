// Admin module for user management endpoints

pub mod routes;
pub mod setup_handlers;
pub mod security_monitoring_handlers;
pub mod plan_handlers;
pub mod promotion_handlers;
pub mod performance_handlers;
pub mod auth_handlers;
pub mod responses;
pub mod wallet_management_handlers;
pub mod wallet_disable_handlers;
pub mod analytics_handlers;
pub mod notification_handlers;
pub mod notification_query_helper;
pub mod wallet_notification_repository;
pub mod system_settings_handlers;
pub mod developer_portal_handlers;
pub mod payment_link_handlers;

// Consolidated permission module (v3.0) - replaces 5 handler files (3,743 lines)
pub mod permissions;

pub use routes::{create_admin_routes, create_admin_public_routes, create_permission_authority_routes};