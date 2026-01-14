// Wallet Management Application Layer
// This module contains the application logic for wallet management operations
// following CQRS and hexagonal architecture patterns

pub mod commands;
pub mod queries;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos; // Request/Response DTOs
pub mod wallet_management_repository; // Repository for wallet query operations

// Re-export command and query models for easy access
pub use commands::{
    UpdateWalletCommand,
    UpdateWalletResponse,
    DeleteWalletCommand,
    DeleteWalletResponse,
    GrantPermissionCommand,
    GrantPermissionResponse,

};

pub use queries::{
    GetWalletQuery,
    GetWalletResponse,
    SearchWalletsQuery,
    SearchWalletsResponse,
    ListWalletsQuery,
    ListWalletsResponse,
    WalletSummary,
    GetWalletPermissionsQuery,
    GetWalletPermissionsResponse,


};

// Re-export command handlers
pub use commands::{
    UpdateWalletCommandHandler,
    DeleteWalletCommandHandler,
    GrantPermissionCommandHandler,

};

// Re-export query handlers
pub use queries::{
    GetWalletQueryHandler,
    ListWalletsQueryHandler,
    SearchWalletsQueryHandler,
    GetWalletPermissionsQueryHandler,


};

// Tests module
#[cfg(test)]
mod tests;