// User Management Bounded Context
// This context handles user registration, authentication, permissions, and session management
// It encapsulates all business rules related to user lifecycle and access control

pub mod aggregates;
pub mod entities;
pub mod value_objects;
pub mod events;
pub mod repository_ports;
pub mod domain_services;

// Re-export key types for easy access
// Web3 wallet user types (primary)
pub use aggregates::{WalletUser, WalletMetadata, Session};

pub use value_objects::{
    WalletAddress, // Primary Web3 wallet identity
    Permission,    // Enhanced for Web3 permission system
};

// Re-export shared kernel value objects
pub use crate::domain::shared_kernel::value_objects::{UserId, SessionId};

pub use events::{
    // Web3 wallet user events
    WalletUserCreatedEvent,
    WalletUserActivatedEvent,
    WalletUserDeactivatedEvent,
    WalletPermissionsUpdatedEvent,
    // Session events
    SessionCreatedEvent,
    SessionInvalidatedEvent,
    SessionExtendedEvent,
    session_events::SessionInvalidationReason,
};

pub use repository_ports::{
    // Web3 wallet user repository ports
    WalletUserRepositoryPort,
    WalletUserAnalyticsPort,
    WalletUserSearchCriteria,
    WalletUserSearchResult,
    WalletUserStatistics,
    Web3Analytics,
    // Session repository ports
    SessionRepositoryPort,
    SessionSearchCriteria,
    SessionSearchResult,
    SessionStatistics,
    SessionAnalyticsPort
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
    SessionSecurityService,
};