// Permission Management Bounded Context
// Handles permission groups, policies, and permission assignment rules

pub mod aggregates;
pub mod entities;
pub mod value_objects;
pub mod events;
pub mod repository_ports;
pub mod domain_services;

// Re-export key types
pub use aggregates::{
    PermissionGroup, Policy,
    CreatePermissionGroupParams, LoadPermissionGroupParams, UpdatePermissionGroupParams,
};

pub use value_objects::{
    GroupId, GroupSlug, PolicyId, PolicyRule, PermissionString
};

pub use events::{
    PermissionGroupCreatedEvent,
    PermissionGroupUpdatedEvent,
    PermissionGroupDeletedEvent,
    WalletAssignedToGroupEvent,
    WalletRemovedFromGroupEvent,
    PolicyCreatedEvent,
    PolicyUpdatedEvent,
};

pub use repository_ports::{
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
