// Admin module for user management endpoints

pub mod routes;
pub mod setup_handlers;
pub mod security_monitoring_handlers;
pub mod plan_handlers;
pub mod permission_group_handlers;
pub mod performance_handlers;
pub mod web3_admin_handlers;
pub mod responses;
pub mod user_management_handlers;
pub mod analytics_handlers;

// Centralized permission system handlers (v2.0)
pub mod centralized_permission_handlers;

pub use routes::{create_admin_routes, create_admin_public_routes, create_permission_authority_routes};