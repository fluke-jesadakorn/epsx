// Resource management services
// Domain services for usage tracking, billing, and resource optimization

pub mod resource_tracking_service;
pub mod billing_calculation_service;
pub mod usage_analytics_service;
pub mod rate_limiting_service;

// Re-export service types with specific imports to avoid conflicts
pub use resource_tracking_service::{
    ResourceTrackingService, ResourceUsageEvent,
    BillingSummary as ResourceBillingSummary, IdentifierType as ResourceIdentifierType
};
pub use billing_calculation_service::{
    BillingCalculationService,
    BillingSummary as BillingServiceSummary
};
pub use usage_analytics_service::{
    UsageAnalyticsService
};
pub use rate_limiting_service::{
    RateLimitingService, RateLimitRequest, RateLimitResult,
    IdentifierType as RateLimitIdentifierType, RateLimitConfig
};