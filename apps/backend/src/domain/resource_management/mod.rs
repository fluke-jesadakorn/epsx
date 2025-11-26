// Resource Management Domain
// Handles usage tracking, billing calculation, and resource optimization

pub mod aggregates;
pub mod services;
pub mod value_objects;
pub mod repository_ports;
pub mod events;

// Re-export aggregates and IDs
pub use aggregates::{
    UserResourceUsage, ResourceUsageId,
    PlanResourceConfig, PlanResourceConfigId
};

// Alias for backward compatibility
pub use aggregates::{UserResourceUsage as ResourceUsageAggregate};

// Re-export events
pub use events::{
    ResourceUsageExceeded, ResourceUsageWarning,
    PlanUpgradeRecommended, BillingCalculated,
    UsagePatternDetected, UsagePattern
};

// Re-export domain services
pub use services::{
    BillingCalculationService,
    UsageAnalyticsService
};

// Re-export value objects
pub use value_objects::{
    AccessContext as ValueObjectAccessContext, CostCalculation, ResourceType,
    UsageMetrics, UsagePrediction as ValueObjectUsagePrediction,
    UsageAnalytics as ValueObjectUsageAnalytics,
    ResourceCategory
};