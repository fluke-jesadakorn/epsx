// Admin module for user management endpoints

pub mod routes;
pub mod setup_handlers;
pub mod security_monitoring_handlers;
pub mod plan_management_handlers_simple;
pub mod permission_group_handlers;
pub mod performance_handlers;
pub mod web3_admin_handlers;

pub use routes::{create_admin_routes, create_admin_public_routes, create_permission_authority_routes};