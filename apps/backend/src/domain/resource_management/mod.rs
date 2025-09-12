// Resource Management Domain
// Handles usage tracking, billing calculation, and resource optimization

pub mod aggregates;
pub mod services;
pub mod value_objects;
pub mod repository_ports;
pub mod events;

// Re-export domain concepts with explicit imports to avoid conflicts
pub use services::{
    BillingCalculationService, RateLimitingService, ResourceTrackingService, 
    UsageAnalyticsService
};
pub use value_objects::{
    AccessContext as ValueObjectAccessContext, CostCalculation, ResourceType,
    UsageMetrics, UsagePrediction as ValueObjectUsagePrediction,
    UsageAnalytics as ValueObjectUsageAnalytics
};
pub use aggregates::{UserResourceUsage};
// Alias for backward compatibility
pub use aggregates::{UserResourceUsage as ResourceUsageAggregate};
// Additional commonly needed exports
pub use value_objects::{ResourceCategory};