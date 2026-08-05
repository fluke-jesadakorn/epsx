// Subscription Management Bounded Context
// Handles plans, subscriptions, billing cycles, and subscription lifecycle

pub mod aggregates;
pub mod domain_services;
pub mod entities;
pub mod events;
pub mod promotion;
pub mod repository_ports;
pub mod value_objects;

// Re-export key types
// Re-export key types
pub use aggregates::{CreatePlanParams, Plan};

pub use value_objects::{BillingCycle, PlanFeatures, PlanId, Price};

pub use events::{PlanCreatedEvent, PlanDeletedEvent, PlanUpdatedEvent};

pub use repository_ports::{PlanRepositoryPort, PlanSearchCriteria};

pub use domain_services::PricingService;

pub use promotion::{Promotion, PromotionStatus, PromotionType};
