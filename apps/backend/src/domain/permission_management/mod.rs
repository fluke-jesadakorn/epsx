// Permission Management Bounded Context
// Handles groups, policies, and permission assignment rules

pub mod aggregates;
pub mod entities;
pub mod value_objects;
pub mod events;
pub mod repository_ports;
pub mod domain_services;

// Re-export key types (new names)
pub use aggregates::{
    Group, CreateGroupParams, LoadGroupParams, UpdateGroupParams,
    // Backward compatibility aliases
    PermissionGroup, CreatePermissionGroupParams, LoadPermissionGroupParams, UpdatePermissionGroupParams,
    Policy,
};

pub use value_objects::{
    GroupId, GroupSlug, PolicyId, PolicyRule, PermissionString
};

pub use events::{
    GroupCreatedEvent,
    GroupUpdatedEvent,
    GroupDeletedEvent,
    WalletAssignedToGroupEvent,
    WalletRemovedFromGroupEvent,
    // Backward compatibility aliases
    PermissionGroupCreatedEvent,
    PermissionGroupUpdatedEvent,
    PermissionGroupDeletedEvent,
    PolicyCreatedEvent,
    PolicyUpdatedEvent,
};

pub use repository_ports::{
    GroupRepositoryPort,
    PermissionGroupRepositoryPort,
    PolicyRepositoryPort,
    GroupAssignmentRepositoryPort,
    GroupSearchCriteria,
    GroupStatistics,
};

pub use domain_services::{
    PermissionValidationService,
    GroupAssignmentService,
};
