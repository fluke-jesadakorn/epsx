// Resource Management Domain
// Handles usage tracking, billing calculation, and resource optimization

pub mod aggregates;
pub mod events;
pub mod repository_ports;
pub mod services;
pub mod value_objects;

// Re-export aggregates and IDs
pub use aggregates::{
    PlanResourceConfig, PlanResourceConfigId, ResourceUsageId, UserResourceUsage,
};

// Alias for backward compatibility
pub use aggregates::UserResourceUsage as ResourceUsageAggregate;

// Re-export events
pub use events::{
    BillingCalculated, PlanUpgradeRecommended, ResourceUsageExceeded, ResourceUsageWarning,
    UsagePattern, UsagePatternDetected,
};

// Re-export domain services
pub use services::{BillingCalculationService, UsageAnalyticsService};

// Re-export value objects
pub use value_objects::{
    AccessContext as ValueObjectAccessContext, CostCalculation, ResourceCategory, ResourceType,
    UsageAnalytics as ValueObjectUsageAnalytics, UsageMetrics,
    UsagePrediction as ValueObjectUsagePrediction,
};
