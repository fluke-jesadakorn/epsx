// Permission Management Bounded Context
// Handles plans, policies, and permission assignment rules

pub mod aggregates;
pub mod domain_services;
pub mod entities;
pub mod events;
pub mod repository_ports;
pub mod value_objects;

// Re-export key types (new names)
pub use aggregates::{
    CreatePermissionPlanParams,
    CreatePlanParams,
    CreatePlanParams as CreatePermissionGroupParams,
    LoadPermissionPlanParams,
    LoadPlanParams,
    LoadPlanParams as LoadPermissionGroupParams,
    PermissionPlan,
    Plan,
    // Backward compatibility aliases
    Plan as PermissionGroup,
    Policy,
    UpdatePermissionPlanParams,
    UpdatePlanParams,
    UpdatePlanParams as UpdatePermissionGroupParams,
};

pub use value_objects::{
    PermissionString, PlanCategory, PlanGroup, PlanId, PlanSlug, PolicyId, PolicyRule,
};

pub use entities::{PlanAssignment, PlanAssignment as GroupAssignment};

pub use events::{
    PlanCreatedEvent,
    // Backward compatibility aliases
    PlanCreatedEvent as PermissionPlanCreatedEvent,
    PlanDeletedEvent,
    PlanDeletedEvent as PermissionPlanDeletedEvent,
    PlanUpdatedEvent,
    PlanUpdatedEvent as PermissionPlanUpdatedEvent,
    PolicyCreatedEvent,
    PolicyUpdatedEvent,
    WalletAssignedToPlanEvent,
    WalletRemovedFromPlanEvent,
};

pub use repository_ports::{
    PlanAssignmentRepositoryPort, PlanAssignmentRepositoryPort as GroupAssignmentRepositoryPort,
    PlanRepositoryPort, PlanRepositoryPort as PermissionGroupRepositoryPort,
    PlanRepositoryPort as PermissionPlanRepositoryPort, PlanSearchCriteria,
    PlanSearchCriteria as GroupSearchCriteria, PlanStatistics, PlanStatistics as GroupStatistics,
    PolicyRepositoryPort,
};

pub use domain_services::{
    PermissionValidationService, PlanAssignmentService,
    PlanAssignmentService as GroupAssignmentService,
};
