// Wallet Management Bounded Context
// This context handles wallet registration, authentication, permissions, and session management
// It encapsulates all business rules related to wallet lifecycle and access control

pub mod aggregates;
pub mod domain_services;
pub mod entities;
pub mod events;
pub mod repository_ports;
pub mod value_objects;

// Re-export key types for easy access
// Web3 wallet user types (primary)
pub use aggregates::{WalletMetadata, WalletUser};

pub use value_objects::{
    Permission,    // Enhanced for Web3 permission system
    WalletAddress, // Primary Web3 wallet identity
};

// Re-export shared kernel value objects
pub use epsx_contracts::value_objects::{SessionId, UserId};

pub use events::{
    WalletPermissionsUpdatedEvent,
    WalletUserActivatedEvent,
    // Web3 wallet events
    WalletUserCreatedEvent,
    WalletUserDeactivatedEvent,
};

pub use repository_ports::{
    WalletUserAnalyticsPort,
    // Web3 wallet repository ports
    WalletUserRepositoryPort,
    WalletUserSearchCriteria,
    WalletUserSearchResult,
    WalletUserStatistics,
    Web3Analytics,
    // Session repository ports
};

// Web3 wallet permission services
pub use domain_services::{
    HasChainAccessSpecification, HasWalletPlatformAccessSpecification, IsWalletAdminSpecification,
    PermissionSyncResult, WalletPermissionService, Web3PermissionContext, Web3ValidationResult,
    Web3ValidationType,
};
