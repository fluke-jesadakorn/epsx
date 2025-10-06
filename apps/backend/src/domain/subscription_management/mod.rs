// Subscription Management Bounded Context
// Handles plans, subscriptions, billing cycles, and subscription lifecycle

pub mod aggregates;
pub mod entities;
pub mod value_objects;
pub mod events;
pub mod repository_ports;
pub mod domain_services;

// Re-export key types
pub use aggregates::{Plan, Subscription, CreatePlanParams, CreateSubscriptionParams};

pub use value_objects::{
    PlanId, SubscriptionId, Price, BillingCycle, PlanFeatures
};

pub use events::{
    PlanCreatedEvent,
    PlanUpdatedEvent,
    PlanDeletedEvent,
    SubscriptionStartedEvent,
    SubscriptionRenewedEvent,
    SubscriptionCancelledEvent,
    SubscriptionExpiredEvent,
};

pub use repository_ports::{
    PlanRepositoryPort,
    SubscriptionRepositoryPort,
    PlanSearchCriteria,
    SubscriptionSearchCriteria,
};

pub use domain_services::{
    PricingService,
    SubscriptionLifecycleService,
};
