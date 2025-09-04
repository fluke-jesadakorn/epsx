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
pub use aggregates::{User, Session};

pub use value_objects::{
    UserId, 
    Email, 
    FirebaseUid, 
    Permission, 
    SessionId
};

pub use events::{
    UserCreatedEvent,
    UserEmailUpdatedEvent,
    UserActivatedEvent,
    UserDeactivatedEvent,
    PermissionGrantedEvent,
    PermissionRevokedEvent,
    UserPermissionsUpdatedEvent,
    SessionCreatedEvent,
    SessionInvalidatedEvent,
    SessionExtendedEvent,
    session_events::SessionInvalidationReason,
};

pub use repository_ports::{
    UserRepositoryPort,
    SessionRepositoryPort,
    user_repository_port::{UserSearchCriteria, UserSearchResult, UserStatistics, UserAnalyticsPort},
    session_repository_port::{SessionSearchCriteria, SessionSearchResult, SessionStatistics, SessionAnalyticsPort}
};

// Domain services will be added here
// pub use domain_services::{UserPermissionService, SessionSecurityService};