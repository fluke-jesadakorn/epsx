// Admin module for user management endpoints

pub mod handlers;
pub mod routes;
pub mod casbin_handlers;
pub mod unified_user_handlers;
pub mod setup_handlers;
pub mod permission_profile_handlers;
pub mod temporary_permission_handlers;
pub mod permission_export_import_handlers;
pub mod analytics_handlers;

pub use routes::{create_admin_routes, create_admin_public_routes};