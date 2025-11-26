/**
 * Diesel Database Models Module
 *
 * Centralized database models using Diesel ORM organized by entity type
 * Replaces the scattered models in database_types.rs for better organization
 */
// Re-export all model modules
pub mod wallet_user;
pub mod session;
pub mod permission_group;
pub mod permission;

// Re-export common model structs for convenience
pub use wallet_user::{
    WalletUserDb, NewWalletUserDb, UpdateWalletUserDb, UpdateWalletUserRequest
};

pub use session::{
    SessionDb, NewSessionDb
};

pub use permission_group::{
    PermissionGroupDb, NewPermissionGroupDb, UpdatePermissionGroupDb,
    CreatePermissionGroupRequest, UpdatePermissionGroupRequest
};

pub use permission::{
    PermissionDb, NewPermissionDb, UpdatePermissionDb,
    WalletPermissionsViewDb, CreatePermissionRequest, UpdatePermissionRequest,
    BulkPermissionRequest, PermissionStats, PermissionValidationResult,
    PermissionAssignmentResult, PermissionSearchFilters, PermissionSummary,
    PlatformPermissionStats
};



// Common type aliases for database types
pub type DbTimestamp = chrono::DateTime<chrono::Utc>;
pub type DbPool = &'static diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>;