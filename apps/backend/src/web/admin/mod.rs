// Admin module for user management endpoints

pub mod handlers;
pub mod routes;
// casbin_handlers removed - using modern JWT auth
pub mod unified_user_handlers;
pub mod setup_handlers;
pub mod permission_profile_handlers;
pub mod temporary_permission_handlers;
pub mod permission_export_import_handlers;
pub mod analytics_handlers;
pub mod firebase_user_management;
pub mod database_role_management;
pub mod admin_role_management;
pub mod search_handlers;

pub use routes::{create_admin_routes, create_admin_public_routes};