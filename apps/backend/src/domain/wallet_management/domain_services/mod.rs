// Wallet Management Domain Services
// Domain services contain business logic that doesn't naturally fit within a single aggregate
// or that requires coordination between multiple aggregates

// Web3 wallet-based services
pub mod wallet_permission_service;

// NEW - Web3 wallet permission service (primary)
pub use wallet_permission_service::{
    HasChainAccessSpecification, HasWalletPlatformAccessSpecification, IsWalletAdminSpecification,
    PermissionSyncResult, WalletPermissionService, Web3PermissionContext, Web3ValidationResult,
    Web3ValidationType,
};
