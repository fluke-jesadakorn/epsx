// Admin module for user management endpoints

pub mod handlers;
pub mod routes;
// casbin_handlers removed - using modern JWT auth
// pub mod unified_user_handlers; // Removed - method call errors on placeholder types
pub mod setup_handlers;
// Removed: permission handlers
// Legacy analytics handlers removed for Web3-first migration
// pub mod analytics_handlers;
// pub mod database_role_management; // Removed - using permissions-based system
// Removed admin_role_management - using simple roles
// pub mod search_handlers; // Removed - method call errors on placeholder types
// Legacy permission handlers removed for Web3-first migration
// pub mod embedded_permission_handlers;
// pub mod bulk_permission_handlers;
// pub mod granular_permissions;
// pub mod notification_handlers;
// Security monitoring handlers
pub mod security_monitoring_handlers;
// Permission hierarchy handlers (DISABLED during refactoring)
// pub mod hierarchy_handlers;
// Legacy dynamic policy handlers removed for Web3-first migration
// pub mod policy_handlers;
// Dynamic plan management handlers (simplified for compilation)
pub mod plan_management_handlers_simple;
// Tier group management handlers (unified permission system)
pub mod tier_group_handlers;
// Performance monitoring handlers for authentication system
pub mod performance_handlers;
// Web3 permission management handlers
pub mod web3_admin_handlers;

// Pure Web3 wallet-first handlers and routes
// pub mod wallet_admin_handlers; // Removed - references non-existent services
// pub mod pure_web3_routes; // Removed - compilation issues

pub use routes::{create_admin_routes, create_admin_public_routes};

// Export pure Web3 routes
// pub use pure_web3_routes::{
//     create_pure_web3_admin_routes, 
//     create_pure_web3_admin_public_routes
// }; // Removed - compilation issues