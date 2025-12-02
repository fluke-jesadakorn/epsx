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
pub mod payment;

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
    CreatePermissionRequest, UpdatePermissionRequest,
    BulkPermissionRequest, PermissionStats, PermissionValidationResult,
    PermissionAssignmentResult, PermissionSearchFilters, PermissionSummary,
    PlatformPermissionStats
};

pub use payment::{
    PaymentDb, NewPaymentDb, UpdatePaymentDb,
    SubscriptionDb, NewSubscriptionDb, UpdateSubscriptionDb,
    PaymentAuditLogDb, NewPaymentAuditLogDb,
    CreatePaymentRequest, UpdatePaymentRequest,
    CreateSubscriptionRequest, UpdateSubscriptionRequest,
    PaymentStatsDb, PaymentSummaryDb
};



// Common type aliases for database types
pub type DbTimestamp = chrono::DateTime<chrono::Utc>;
pub type DbPool = &'static diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>;