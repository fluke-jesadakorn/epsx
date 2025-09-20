// Admin module for user management endpoints

pub mod handlers;
pub mod routes;
// casbin_handlers removed - using modern JWT auth
pub mod unified_user_handlers;
pub mod setup_handlers;
// Removed: permission handlers
pub mod analytics_handlers;
// pub mod database_role_management; // Removed - using permissions-based system
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
// Security monitoring handlers
pub mod security_monitoring_handlers;
// Permission hierarchy handlers (DISABLED during refactoring)
// pub mod hierarchy_handlers;
// Dynamic policy handlers
pub mod policy_handlers;
// Dynamic plan management handlers (simplified for compilation)
pub mod plan_management_handlers_simple;
// Performance monitoring handlers for authentication system
pub mod performance_handlers;
// Web3 permission management handlers
pub mod web3_admin_handlers;

pub use routes::{create_admin_routes, create_admin_public_routes};