// Wallet Management Bounded Context
// This context handles wallet registration, authentication, permissions, and session management
// It encapsulates all business rules related to wallet lifecycle and access control

pub mod aggregates;
pub mod entities;
pub mod value_objects;
pub mod events;
pub mod repository_ports;
pub mod domain_services;

// Re-export key types for easy access
// Web3 wallet user types (primary)
pub use aggregates::{WalletUser, WalletMetadata};

pub use value_objects::{
    WalletAddress, // Primary Web3 wallet identity
    Permission,    // Enhanced for Web3 permission system
};

// Re-export shared kernel value objects
pub use epsx_contracts::value_objects::{UserId, SessionId};

pub use events::{
    // Web3 wallet events
    WalletUserCreatedEvent,
    WalletUserActivatedEvent,
    WalletUserDeactivatedEvent,
    WalletPermissionsUpdatedEvent,

};

pub use repository_ports::{
    // Web3 wallet repository ports
    WalletUserRepositoryPort,
    WalletUserAnalyticsPort,
    WalletUserSearchCriteria,
    WalletUserSearchResult,
    WalletUserStatistics,
    Web3Analytics,
    // Session repository ports

};

// Web3 wallet permission services
pub use domain_services::{
    WalletPermissionService,
    Web3PermissionContext,
    Web3ValidationResult,
    Web3ValidationType,
    PermissionSyncResult,
    IsWalletAdminSpecification,
    HasWalletPlatformAccessSpecification,
    HasChainAccessSpecification,

};