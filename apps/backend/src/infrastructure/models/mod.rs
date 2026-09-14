/**
 * Diesel Database Models Module
 *
 * Centralized database models using Diesel ORM organized by entity type
 * Replaces the scattered models in database_types.rs for better organization
 */
// Re-export all model modules
pub mod wallet_user;

pub mod audit;
pub mod chat;
pub mod credit;
pub mod news;
pub mod notification;
pub mod payment;
pub mod permission;
pub mod plan;

// Re-export common model structs for convenience
pub use wallet_user::{NewWalletUserDb, UpdateWalletUserDb, UpdateWalletUserRequest, WalletUserDb};

// Primary exports (new names)
pub use plan::{
    CreatePermissionGroupRequest,
    CreatePlanRequest as CreateGroupRequest,
    CreatePlanRequest,
    NewPermissionGroupDb,
    NewPlanDb as NewGroupDb,
    NewPlanDb,
    // Backward compatibility aliases
    PermissionGroupDb,
    PlanDb as GroupDb,
    // Unified names
    PlanDb,
    UpdatePermissionGroupDb,
    UpdatePermissionGroupRequest,
    UpdatePlanDb as UpdateGroupDb,
    UpdatePlanDb,
    UpdatePlanRequest as UpdateGroupRequest,
    UpdatePlanRequest,
};

pub use permission::{
    BulkPermissionRequest, CreatePermissionRequest, NewPermissionDb, PermissionAssignmentResult,
    PermissionDb, PermissionSearchFilters, PermissionStats, PermissionSummary,
    PermissionValidationResult, PlatformPermissionStats, UpdatePermissionDb,
    UpdatePermissionRequest,
};

pub use payment::{
    CreatePaymentRequest, CreateSubscriptionRequest, NewPaymentAuditLogDb, NewPaymentDb,
    NewSubscriptionDb, PaymentAuditLogDb, PaymentDb, PaymentStatsDb, PaymentSummaryDb,
    SubscriptionDb, UpdatePaymentDb, UpdatePaymentRequest, UpdateSubscriptionDb,
    UpdateSubscriptionRequest,
};

pub use credit::{
    CreditBalanceResponse, CreditStatsResponse, CreditTransactionDb, CreditTransactionFilters,
    CreditTransactionResponse, GrantCreditsRequest, NewCreditTransactionDb, NewWalletCreditDb,
    RevokeCreditsRequest, UpdateWalletCreditDb, WalletCreditDb,
};

// Common type aliases for database types
pub type DbTimestamp = chrono::DateTime<chrono::Utc>;
pub type DbPool = crate::prelude::TlsPool;
