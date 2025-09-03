// Admin module for user management endpoints

pub mod handlers;
pub mod routes;
// casbin_handlers removed - using modern JWT auth
pub mod unified_user_handlers;
pub mod setup_handlers;
// Removed: permission handlers
pub mod analytics_handlers;
pub mod firebase_user_management;
pub mod database_role_management;
// Removed admin_role_management - using simple roles
pub mod search_handlers;
// Embedded timestamp permission management
pub mod embedded_permission_handlers;
// Bulk permission management
pub mod bulk_permission_handlers;
// V1 Granular permission management API
pub mod granular_permissions;
// Admin notification handlers
pub mod notification_handlers;

pub use routes::{create_admin_routes, create_admin_public_routes};