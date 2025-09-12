// Resource tracking service
// Core domain service for tracking and analyzing resource usage across all access contexts

use std::{sync::Arc, collections::HashMap};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use thiserror::Error;
use serde::{Deserialize, Serialize};

use crate::domain::{
    shared_kernel::domain_error::DomainError,
    resource_management::value_objects::{
        resource_type::{ResourceType, ResourceCategory, TrackingPriority},
        usage_metrics::{
            UsageMetrics, TimePeriod, RealTimeUsageTracker, UsageAnalytics,
            UsageSummary, ConsumerType, TopConsumer,
        },
    },
};

/// Resource tracking service for monitoring usage across all access contexts
pub struct ResourceTrackingService {
    usage_repository: Arc<dyn UsageRepositoryPort>,
    analytics_repository: Arc<dyn AnalyticsRepositoryPort>,
    real_time_cache: Arc<dyn RealTimeCachePort>,
    billing_calculator: Arc<dyn BillingCalculatorPort>,
}

/// Repository port for usage data persistence
#[async_trait::async_trait]
pub trait UsageRepositoryPort: Send + Sync {
    async fn save_usage_metrics(&self, metrics: &UsageMetrics) -> Result<(), DomainError>;
    async fn get_usage_metrics(&self, identifier: &str, time_period: &TimePeriod) -> Result<Option<UsageMetrics>, DomainError>;
    async fn get_usage_history(&self, identifier: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<UsageMetrics>, DomainError>;
    async fn aggregate_usage_by_category(&self, time_period: &TimePeriod) -> Result<HashMap<ResourceCategory, Decimal>, DomainError>;
    async fn get_top_consumers(&self, time_period: &TimePeriod, limit: usize) -> Result<Vec<TopConsumer>, DomainError>;
}

/// Repository port for usage analytics
#[async_trait::async_trait]
pub trait AnalyticsRepositoryPort: Send + Sync {
    async fn save_usage_analytics(&self, analytics: &UsageAnalytics) -> Result<(), DomainError>;
    async fn get_usage_trends(&self, time_period: &TimePeriod) -> Result<Option<UsageAnalytics>, DomainError>;
    async fn calculate_predictions(&self, historical_data: &[UsageMetrics]) -> Result<UsagePrediction, DomainError>;
}

/// Cache port for real-time usage tracking
#[async_trait::async_trait]
pub trait RealTimeCachePort: Send + Sync {
    async fn get_real_time_usage(&self, identifier: &str) -> Result<Option<RealTimeUsageTracker>, DomainError>;
    async fn update_real_time_usage(&self, tracker: &RealTimeUsageTracker) -> Result<(), DomainError>;
    async fn get_current_rate_limit_usage(&self, identifier: &str, time_window: &TimePeriod) -> Result<u64, DomainError>;
    async fn cleanup_expired_trackers(&self) -> Result<u32, DomainError>;
}

/// Billing calculation port
#[async_trait::async_trait]
pub trait BillingCalculatorPort: Send + Sync {
    async fn calculate_cost(&self, resource_type: &ResourceType, quantity: u64) -> Result<Decimal, DomainError>;
    async fn calculate_total_bill(&self, metrics: &UsageMetrics) -> Result<BillingSummary, DomainError>;
    async fn apply_plan_discounts(&self, base_cost: Decimal, plan_id: i32) -> Result<Decimal, DomainError>;
}

/// Resource usage event for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageEvent {
    pub identifier: String,
    pub identifier_type: IdentifierType,
    pub resource_type: ResourceType,
    pub quantity: u64,
    pub metadata: ResourceMetadata,
    pub timestamp: DateTime<Utc>,
    pub context: UsageContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentifierType {
    UserId(String),
    ApiKey(String),
    SessionId(String),
    AdminUserId(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub endpoint: Option<String>,
    pub response_code: Option<u16>,
    pub response_size_bytes: Option<u64>,
    pub processing_time_ms: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageContext {
    pub access_context: String, // "internal", "external", "admin"
    pub plan_id: Option<i32>,
    pub session_id: Option<String>,
    pub is_billable: bool,
    pub requires_audit: bool,
}

/// Billing summary for cost calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingSummary {
    pub base_cost: Decimal,
    pub discounted_cost: Decimal,
    pub tax_amount: Decimal,
    pub total_amount: Decimal,
    pub cost_breakdown: HashMap<ResourceCategory, Decimal>,
    pub discount_applied: Option<String>,
    pub billing_period: TimePeriod,
}

/// Usage prediction for capacity planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePrediction {
    pub predicted_requests: u64,
    pub predicted_cost: Decimal,
    pub confidence_score: f64,
    pub prediction_period: TimePeriod,
    pub factors_considered: Vec<String>,
}

#[derive(Error, Debug)]
pub enum ResourceTrackingError {
    #[error("Invalid resource type")]
    InvalidResourceType,
    
    #[error("Usage limit exceeded")]
    UsageLimitExceeded,
    
    #[error("Billing calculation failed: {0}")]
    BillingCalculationFailed(String),
    
    #[error("Real-time cache error: {0}")]
    RealTimeCacheError(String),
    
    #[error("Analytics calculation failed: {0}")]
    AnalyticsCalculationFailed(String),
    
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
}

impl ResourceTrackingService {
    pub fn new(
        usage_repository: Arc<dyn UsageRepositoryPort>,
        analytics_repository: Arc<dyn AnalyticsRepositoryPort>,
        real_time_cache: Arc<dyn RealTimeCachePort>,
        billing_calculator: Arc<dyn BillingCalculatorPort>,
    ) -> Self {
        Self {
            usage_repository,
            analytics_repository,
            real_time_cache,
            billing_calculator,
        }
    }

    /// Track a resource usage event
    pub async fn track_usage(
        &self,
        event: ResourceUsageEvent,
    ) -> Result<TrackingResult, ResourceTrackingError> {
        // Calculate cost for the resource usage
        let cost = self.billing_calculator
            .calculate_cost(&event.resource_type, event.quantity)
            .await?;

        // Update real-time usage tracker
        let mut tracker = self.real_time_cache
            .get_real_time_usage(&event.identifier)
            .await?
            .unwrap_or_else(|| RealTimeUsageTracker::new(event.identifier.clone()));

        tracker.record_usage(event.resource_type.clone(), cost);

        self.real_time_cache
            .update_real_time_usage(&tracker)
            .await?;

        // Batch persist based on priority
        let should_persist_immediately = matches!(
            event.resource_type.priority(),
            TrackingPriority::High
        );

        if should_persist_immediately {
            self.persist_usage_event(&event, cost).await?;
        } else {
            // Queue for batch processing
            self.queue_for_batch_processing(&event, cost).await?;
        }

        // Generate tracking result
        Ok(TrackingResult {
            tracked: true,
            cost,
            current_usage: tracker.current_day.total_requests,
            estimated_monthly_cost: self.estimate_monthly_cost(&tracker).await?,
            warnings: self.generate_warnings(&tracker).await?,
        })
    }

    /// Get usage summary for an identifier
    pub async fn get_usage_summary(
        &self,
        identifier: &str,
        time_period: TimePeriod,
    ) -> Result<UsageSummary, ResourceTrackingError> {
        let metrics = self.usage_repository
            .get_usage_metrics(identifier, &time_period)
            .await?
            .ok_or_else(|| ResourceTrackingError::RepositoryError("No usage data found".to_string()))?;

        Ok(metrics.get_usage_summary())
    }

    /// Get real-time usage for rate limiting
    pub async fn get_current_usage(
        &self,
        identifier: &str,
        time_window: TimePeriod,
    ) -> Result<u64, ResourceTrackingError> {
        let usage_count = self.real_time_cache
            .get_current_rate_limit_usage(identifier, &time_window)
            .await?;

        Ok(usage_count)
    }

    /// Generate usage analytics
    pub async fn generate_analytics(
        &self,
        time_period: TimePeriod,
    ) -> Result<UsageAnalytics, ResourceTrackingError> {
        // Get category aggregations
        let category_totals = self.usage_repository
            .aggregate_usage_by_category(&time_period)
            .await?;

        // Get top consumers
        let top_consumers = self.usage_repository
            .get_top_consumers(&time_period, 10)
            .await?;

        // Calculate totals
        let _total_cost: Decimal = category_totals.values().sum();
        let total_users = top_consumers.iter()
            .filter(|c| matches!(c.identifier_type, ConsumerType::User))
            .count() as u64;
        let total_api_keys = top_consumers.iter()
            .filter(|c| matches!(c.identifier_type, ConsumerType::ApiKey))
            .count() as u64;

        // Build analytics (simplified for now)
        Ok(UsageAnalytics {
            time_period,
            total_users,
            total_api_keys,
            usage_trends: Default::default(),
            cost_analysis: Default::default(),
            performance_metrics: Default::default(),
            top_consumers,
        })
    }

    /// Calculate billing for a specific identifier and time period
    pub async fn calculate_billing(
        &self,
        identifier: &str,
        time_period: TimePeriod,
        plan_id: Option<i32>,
    ) -> Result<BillingSummary, ResourceTrackingError> {
        let metrics = self.usage_repository
            .get_usage_metrics(identifier, &time_period)
            .await?
            .ok_or_else(|| ResourceTrackingError::RepositoryError("No usage data found".to_string()))?;

        let mut billing_summary = self.billing_calculator
            .calculate_total_bill(&metrics)
            .await?;

        // Apply plan discounts if applicable
        if let Some(plan_id) = plan_id {
            billing_summary.discounted_cost = self.billing_calculator
                .apply_plan_discounts(billing_summary.base_cost, plan_id)
                .await?;
        }

        Ok(billing_summary)
    }

    /// Cleanup expired real-time trackers (background task)
    pub async fn cleanup_expired_trackers(&self) -> Result<u32, ResourceTrackingError> {
        let cleaned_count = self.real_time_cache
            .cleanup_expired_trackers()
            .await?;

        tracing::info!(
            target: "resource_tracking_service",
            cleaned_trackers = cleaned_count,
            "Cleaned up expired usage trackers"
        );

        Ok(cleaned_count)
    }

    // Private helper methods

    async fn persist_usage_event(
        &self,
        event: &ResourceUsageEvent,
        cost: Decimal,
    ) -> Result<(), ResourceTrackingError> {
        // Create or update usage metrics for the current time period
        let time_period = TimePeriod::current_day();
        
        let mut metrics = self.usage_repository
            .get_usage_metrics(&event.identifier, &time_period)
            .await?
            .unwrap_or_else(|| {
                match &event.identifier_type {
                    IdentifierType::UserId(user_id) => UsageMetrics::new(
                        Some(user_id.clone()),
                        None,
                        None,
                        time_period.clone(),
                    ),
                    IdentifierType::ApiKey(api_key) => UsageMetrics::new(
                        None,
                        Some(api_key.clone()),
                        None,
                        time_period.clone(),
                    ),
                    IdentifierType::SessionId(session_id) => UsageMetrics::new(
                        None,
                        None,
                        Some(session_id.clone()),
                        time_period.clone(),
                    ),
                    IdentifierType::AdminUserId(admin_id) => UsageMetrics::new(
                        Some(admin_id.clone()),
                        None,
                        None,
                        time_period.clone(),
                    ),
                }
            });

        // Add the usage event
        metrics.add_resource_usage(
            event.resource_type.clone(),
            cost,
            event.metadata.response_size_bytes,
            event.metadata.processing_time_ms,
        );

        // Save updated metrics
        self.usage_repository
            .save_usage_metrics(&metrics)
            .await?;

        Ok(())
    }

    async fn queue_for_batch_processing(
        &self,
        _event: &ResourceUsageEvent,
        _cost: Decimal,
    ) -> Result<(), ResourceTrackingError> {
        // TODO: Implement batch processing queue
        // For now, just persist immediately
        Ok(())
    }

    async fn estimate_monthly_cost(
        &self,
        tracker: &RealTimeUsageTracker,
    ) -> Result<Decimal, ResourceTrackingError> {
        let daily_cost = tracker.current_day.total_cost;
        let days_in_month = 30;
        Ok(daily_cost * Decimal::from(days_in_month))
    }

    async fn generate_warnings(
        &self,
        _tracker: &RealTimeUsageTracker,
    ) -> Result<Vec<String>, ResourceTrackingError> {
        // TODO: Implement warning generation based on usage patterns
        Ok(vec![])
    }
}

/// Result of tracking a resource usage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingResult {
    pub tracked: bool,
    pub cost: Decimal,
    pub current_usage: u64,
    pub estimated_monthly_cost: Decimal,
    pub warnings: Vec<String>,
}

// Default implementations for simplified analytics types
use crate::domain::resource_management::value_objects::usage_metrics::{
    UsageTrends, CostAnalysis, PerformanceMetrics,
};

impl Default for UsageTrends {
    fn default() -> Self {
        Self {
            growth_rate_percentage: 0.0,
            seasonal_patterns: vec![],
            peak_usage_hours: vec![],
            prediction_next_period: Default::default(),
        }
    }
}

impl Default for CostAnalysis {
    fn default() -> Self {
        Self {
            total_infrastructure_cost: Decimal::ZERO,
            total_billable_revenue: Decimal::ZERO,
            profit_margin_percentage: 0.0,
            cost_per_user: Decimal::ZERO,
            revenue_per_api_key: Decimal::ZERO,
            most_expensive_resources: vec![],
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            average_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            error_rate_percentage: 0.0,
            cache_hit_rate_percentage: 0.0,
            resource_efficiency_score: 1.0,
        }
    }
}

impl Default for UsagePrediction {
    fn default() -> Self {
        Self {
            predicted_requests: 0,
            predicted_cost: Decimal::ZERO,
            confidence_score: 0.0,
            prediction_period: TimePeriod::current_day(),
            factors_considered: vec![],
        }
    }
}