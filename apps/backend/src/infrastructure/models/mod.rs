/**
 * Diesel Database Models Module
 *
 * Centralized database models using Diesel ORM organized by entity type
 * Replaces the scattered models in database_types.rs for better organization
 */
// Re-export all model modules
pub mod wallet_user;

pub mod plan;
pub mod permission;
pub mod payment;
pub mod credit;
pub mod notification;
pub mod audit;

// Re-export common model structs for convenience
pub use wallet_user::{
    WalletUserDb, NewWalletUserDb, UpdateWalletUserDb, UpdateWalletUserRequest
};


// Primary exports (new names)
pub use plan::{
    PlanDb as GroupDb, NewPlanDb as NewGroupDb, UpdatePlanDb as UpdateGroupDb,
    CreatePlanRequest as CreateGroupRequest, UpdatePlanRequest as UpdateGroupRequest,
    // Unified names
    PlanDb, NewPlanDb, UpdatePlanDb,
    CreatePlanRequest, UpdatePlanRequest,
    // Backward compatibility aliases
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

pub use credit::{
    WalletCreditDb, NewWalletCreditDb, UpdateWalletCreditDb,
    CreditTransactionDb, NewCreditTransactionDb,
    GrantCreditsRequest, RevokeCreditsRequest,
    CreditBalanceResponse, CreditTransactionResponse,
    CreditStatsResponse, CreditTransactionFilters
};



// Common type aliases for database types
pub type DbTimestamp = chrono::DateTime<chrono::Utc>;
pub type DbPool = crate::prelude::TlsPool;