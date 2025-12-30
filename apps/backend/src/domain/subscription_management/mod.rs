// Subscription Management Bounded Context
// Handles plans, subscriptions, billing cycles, and subscription lifecycle

pub mod aggregates;
pub mod entities;
pub mod value_objects;
pub mod events;
pub mod repository_ports;
pub mod domain_services;
pub mod promotion;

// Re-export key types
// Re-export key types
pub use aggregates::{Plan, CreatePlanParams};

pub use value_objects::{
    PlanId, Price, BillingCycle, PlanFeatures
};

pub use events::{
    PlanCreatedEvent,
    PlanUpdatedEvent,
    PlanDeletedEvent,
};

pub use repository_ports::{
    PlanRepositoryPort,
    PlanSearchCriteria,
};

pub use domain_services::{
    PricingService,
};

pub use promotion::{
    Promotion,
    PromotionType,
    PromotionStatus,
};
