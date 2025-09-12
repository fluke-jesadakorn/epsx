// Repository Ports for Resource Management
// Define interfaces for data persistence

use crate::domain::resource_management::aggregates::{UserResourceUsage, PlanResourceConfig};
use async_trait::async_trait;
use std::collections::HashMap;

/// Repository for user resource usage data
#[async_trait]
pub trait UserResourceUsageRepository {
    type Error;

    /// Get user's current resource usage
    async fn get_user_usage(&self, user_id: &str, access_context: &str) -> Result<Option<UserResourceUsage>, Self::Error>;

    /// Save user resource usage
    async fn save_user_usage(&self, usage: &UserResourceUsage) -> Result<(), Self::Error>;

    /// Update user's resource usage
    async fn update_user_usage(&self, usage: &UserResourceUsage) -> Result<(), Self::Error>;

    /// Get historical usage data
    async fn get_usage_history(&self, user_id: &str, days: u32) -> Result<Vec<UserResourceUsage>, Self::Error>;

    /// Get all users with active usage
    async fn get_active_users(&self) -> Result<Vec<String>, Self::Error>;

    /// Delete expired usage records
    async fn cleanup_expired_usage(&self, days_old: u32) -> Result<u64, Self::Error>;
}

/// Repository for plan resource configurations
#[async_trait]
pub trait PlanResourceConfigRepository {
    type Error;

    /// Get plan resource configuration
    async fn get_plan_config(&self, plan_id: i32) -> Result<Option<PlanResourceConfig>, Self::Error>;

    /// Save plan resource configuration
    async fn save_plan_config(&self, config: &PlanResourceConfig) -> Result<(), Self::Error>;

    /// Update plan resource configuration
    async fn update_plan_config(&self, config: &PlanResourceConfig) -> Result<(), Self::Error>;

    /// Get all active plan configurations
    async fn get_active_plan_configs(&self) -> Result<Vec<PlanResourceConfig>, Self::Error>;

    /// Get plan configs by access context
    async fn get_plans_by_context(&self, access_context: &str) -> Result<Vec<PlanResourceConfig>, Self::Error>;
}

/// Repository for resource metrics and analytics
#[async_trait]
pub trait ResourceMetricsRepository {
    type Error;

    /// Record resource usage event
    async fn record_usage_event(
        &self,
        user_id: &str,
        resource_type: &str,
        amount: i64,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Self::Error>;

    /// Get usage metrics for a time period
    async fn get_usage_metrics(
        &self,
        user_id: &str,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<HashMap<String, i64>, Self::Error>;

    /// Get peak usage times
    async fn get_peak_usage_times(
        &self,
        user_id: &str,
        days: u32,
    ) -> Result<Vec<(chrono::DateTime<chrono::Utc>, i64)>, Self::Error>;

    /// Get system-wide usage statistics
    async fn get_system_usage_stats(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<HashMap<String, i64>, Self::Error>;
}