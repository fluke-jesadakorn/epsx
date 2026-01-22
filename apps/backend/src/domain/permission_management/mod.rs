// Permission Management Bounded Context
// Handles plans, policies, and permission assignment rules

pub mod aggregates;
pub mod entities;
pub mod value_objects;
pub mod events;
pub mod repository_ports;
pub mod domain_services;

// Re-export key types (new names)
pub use aggregates::{
    Plan, CreatePlanParams, LoadPlanParams, UpdatePlanParams,
    // Backward compatibility aliases
    Plan as PermissionGroup, CreatePlanParams as CreatePermissionGroupParams, 
    LoadPlanParams as LoadPermissionGroupParams, UpdatePlanParams as UpdatePermissionGroupParams,
    PermissionPlan, CreatePermissionPlanParams, LoadPermissionPlanParams, UpdatePermissionPlanParams,
    Policy,
};

pub use value_objects::{
    PlanId, PlanSlug, PolicyId, PolicyRule, PermissionString
};

pub use entities::{
    PlanAssignment,
    PlanAssignment as GroupAssignment,
};

pub use events::{
    PlanCreatedEvent,
    PlanUpdatedEvent,
    PlanDeletedEvent,
    WalletAssignedToPlanEvent,
    WalletRemovedFromPlanEvent,
    // Backward compatibility aliases
    PlanCreatedEvent as PermissionPlanCreatedEvent,
    PlanUpdatedEvent as PermissionPlanUpdatedEvent,
    PlanDeletedEvent as PermissionPlanDeletedEvent,
    PolicyCreatedEvent,
    PolicyUpdatedEvent,
};

pub use repository_ports::{
    PlanRepositoryPort,
    PlanRepositoryPort as PermissionGroupRepositoryPort,
    PlanRepositoryPort as PermissionPlanRepositoryPort,
    PolicyRepositoryPort,
    PlanAssignmentRepositoryPort,
    PlanAssignmentRepositoryPort as GroupAssignmentRepositoryPort,
    PlanSearchCriteria,
    PlanSearchCriteria as GroupSearchCriteria,
    PlanStatistics,
    PlanStatistics as GroupStatistics,
};

pub use domain_services::{
    PermissionValidationService,
    PlanAssignmentService,
    PlanAssignmentService as GroupAssignmentService,
};
