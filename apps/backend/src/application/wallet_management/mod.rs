// Wallet Management Application Layer
// This module contains the application logic for wallet management operations
// following CQRS and hexagonal architecture patterns

pub mod commands;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos; // Request/Response DTOs
pub mod queries;
pub mod sqlx_wallet_management_repository;
pub mod wallet_management_repository; // Repository for wallet query operations

// Re-export command and query models for easy access
pub use commands::{
    DeleteWalletCommand, DeleteWalletResponse, GrantPermissionCommand, GrantPermissionResponse,
    UpdateWalletCommand, UpdateWalletResponse,
};

pub use queries::{
    GetWalletPermissionsQuery, GetWalletPermissionsResponse, GetWalletQuery, GetWalletResponse,
    ListWalletsQuery, ListWalletsResponse, SearchWalletsQuery, SearchWalletsResponse,
    WalletSummary,
};

// Re-export command handlers
pub use commands::{
    DeleteWalletCommandHandler, GrantPermissionCommandHandler, UpdateWalletCommandHandler,
};

// Re-export query handlers
pub use queries::{
    GetWalletPermissionsQueryHandler, GetWalletQueryHandler, ListWalletsQueryHandler,
    SearchWalletsQueryHandler,
};

// Tests module
