// External authentication service for API developers
// Domain service for API key-based authentication with plan-based access control

use std::{sync::Arc, collections::HashMap};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use rust_decimal::Decimal;

use crate::domain::{
    shared_kernel::{
        domain_error::DomainError,
        value_objects::UserId,
    },
};

/// External authentication service for API developers
/// Handles API key validation, plan-based access control, and usage tracking
pub struct ExternalAuthService {
    api_key_repository: Arc<dyn ApiKeyRepositoryPort>,
    plan_repository: Arc<dyn PlanRepositoryPort>,
    usage_tracker: Arc<dyn UsageTrackerPort>,
}

/// API key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub user_id: UserId,
    pub plan_id: i32,
    pub name: String,
    pub status: ApiKeyStatus,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_limits: UsageLimits,
    pub current_usage: UsageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiKeyStatus {
    Active,
    Suspended,
    Revoked,
    Expired,
}

/// Usage limits based on plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimits {
    pub requests_per_minute: Option<u32>,
    pub requests_per_hour: Option<u32>,
    pub requests_per_day: Option<u32>,
    pub requests_per_month: Option<u32>,
    pub data_transfer_gb: Option<u32>,
    pub webhook_calls: Option<u32>,
    pub concurrent_connections: Option<u32>,
}

/// Current usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub requests_today: u32,
    pub requests_this_hour: u32,
    pub requests_this_minute: u32,
    pub requests_this_month: u32,
    pub data_transfer_mb: u32,
    pub webhook_calls_used: u32,
    pub active_connections: u32,
    pub reset_date: DateTime<Utc>,
}

/// Plan-based feature access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFeatures {
    pub plan_id: i32,
    pub plan_name: String,
    pub plan_tier: PlanTier,
    pub features: HashMap<String, FeatureAccess>,
    pub rate_limits: UsageLimits,
    pub cost_per_request: Decimal,
    pub cost_per_gb: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanTier {
    Starter,
    Professional, 
    Enterprise,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAccess {
    pub enabled: bool,
    pub quota: Option<u32>,
    pub rate_limit: Option<u32>,
    pub requires_approval: bool,
}

/// Authentication context for external API users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAuthContext {
    pub api_key: String,
    pub user_id: UserId,
    pub plan_id: i32,
    pub plan_features: PlanFeatures,
    pub usage_limits: UsageLimits,
    pub current_usage: UsageStats,
    pub client_ip: Option<String>,
    pub last_activity: DateTime<Utc>,
    pub is_quota_exceeded: bool,
    pub estimated_cost: Decimal,
}

/// Authentication request for external API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAuthRequest {
    pub api_key: String,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub requested_endpoint: String,
    pub request_method: String,
}

/// Authentication response for external API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAuthResponse {
    pub auth_context: ExternalAuthContext,
    pub access_granted: bool,
    pub quota_remaining: u32,
    pub rate_limit_remaining: u32,
    pub estimated_cost: Decimal,
    pub warnings: Vec<String>,
}

/// Repository ports
#[async_trait::async_trait]
pub trait ApiKeyRepositoryPort: Send + Sync {
    async fn get_api_key(&self, key: &str) -> Result<Option<ApiKey>, DomainError>;
    async fn update_api_key_usage(&self, key: &str, usage: &UsageStats) -> Result<(), DomainError>;
    async fn create_api_key(&self, api_key: &ApiKey) -> Result<(), DomainError>;
    async fn revoke_api_key(&self, key: &str) -> Result<(), DomainError>;
    async fn get_user_api_keys(&self, user_id: &UserId) -> Result<Vec<ApiKey>, DomainError>;
}

#[async_trait::async_trait]
pub trait PlanRepositoryPort: Send + Sync {
    async fn get_plan_features(&self, plan_id: i32) -> Result<Option<PlanFeatures>, DomainError>;
    async fn is_feature_enabled(&self, plan_id: i32, feature: &str) -> Result<bool, DomainError>;
    async fn get_rate_limits(&self, plan_id: i32) -> Result<UsageLimits, DomainError>;
}

#[async_trait::async_trait]
pub trait UsageTrackerPort: Send + Sync {
    async fn track_api_request(&self, api_key: &str, endpoint: &str, cost: Decimal) -> Result<(), DomainError>;
    async fn get_current_usage(&self, api_key: &str) -> Result<UsageStats, DomainError>;
    async fn check_rate_limits(&self, api_key: &str, limits: &UsageLimits) -> Result<bool, DomainError>;
    async fn reset_usage_counters(&self, reset_period: ResetPeriod) -> Result<u32, DomainError>;
}

#[derive(Debug, Clone)]
pub enum ResetPeriod {
    Minute,
    Hour,
    Day,
    Month,
}

#[derive(Error, Debug)]
pub enum ExternalAuthError {
    #[error("Invalid API key")]
    InvalidApiKey,
    
    #[error("API key suspended")]
    ApiKeySuspended,
    
    #[error("API key revoked")]
    ApiKeyRevoked,
    
    #[error("API key expired")]
    ApiKeyExpired,
    
    #[error("Rate limit exceeded: {limit_type}")]
    RateLimitExceeded { limit_type: String },
    
    #[error("Quota exceeded for plan")]
    QuotaExceeded,
    
    #[error("Feature not available on current plan")]
    FeatureNotAvailable,
    
    #[error("Plan not found")]
    PlanNotFound,
    
    #[error("Insufficient credits")]
    InsufficientCredits,
    
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
}

impl ExternalAuthService {
    pub fn new(
        api_key_repository: Arc<dyn ApiKeyRepositoryPort>,
        plan_repository: Arc<dyn PlanRepositoryPort>,
        usage_tracker: Arc<dyn UsageTrackerPort>,
    ) -> Self {
        Self {
            api_key_repository,
            plan_repository,
            usage_tracker,
        }
    }

    /// Authenticate API request and check plan-based access
    pub async fn authenticate(
        &self,
        request: ExternalAuthRequest,
    ) -> Result<ExternalAuthResponse, ExternalAuthError> {
        // Get and validate API key
        let api_key_info = self.api_key_repository
            .get_api_key(&request.api_key)
            .await?
            .ok_or(ExternalAuthError::InvalidApiKey)?;

        // Check API key status
        self.validate_api_key_status(&api_key_info)?;

        // Get plan features
        let plan_features = self.plan_repository
            .get_plan_features(api_key_info.plan_id)
            .await?
            .ok_or(ExternalAuthError::PlanNotFound)?;

        // Get current usage
        let current_usage = self.usage_tracker
            .get_current_usage(&request.api_key)
            .await?;

        // Check rate limits
        let rate_limit_ok = self.usage_tracker
            .check_rate_limits(&request.api_key, &plan_features.rate_limits)
            .await?;

        if !rate_limit_ok {
            return Err(ExternalAuthError::RateLimitExceeded {
                limit_type: "API requests".to_string()
            });
        }

        // Check feature access for requested endpoint
        let feature_access = self.check_endpoint_access(&request.requested_endpoint, &plan_features)?;
        if !feature_access.enabled {
            return Err(ExternalAuthError::FeatureNotAvailable);
        }

        // Calculate estimated cost
        let estimated_cost = plan_features.cost_per_request;

        // Check quota limits
        let is_quota_exceeded = self.is_quota_exceeded(&current_usage, &plan_features.rate_limits);
        
        if is_quota_exceeded {
            return Err(ExternalAuthError::QuotaExceeded);
        }

        // Build authentication context
        let auth_context = ExternalAuthContext {
            api_key: request.api_key.clone(),
            user_id: api_key_info.user_id.clone(),
            plan_id: api_key_info.plan_id,
            plan_features: plan_features.clone(),
            usage_limits: plan_features.rate_limits.clone(),
            current_usage: current_usage.clone(),
            client_ip: request.client_ip.clone(),
            last_activity: Utc::now(),
            is_quota_exceeded,
            estimated_cost,
        };

        // Calculate remaining quotas
        let quota_remaining = self.calculate_quota_remaining(&current_usage, &plan_features.rate_limits);
        let rate_limit_remaining = self.calculate_rate_limit_remaining(&current_usage, &plan_features.rate_limits);

        // Generate warnings if approaching limits
        let warnings = self.generate_usage_warnings(&current_usage, &plan_features.rate_limits);

        Ok(ExternalAuthResponse {
            auth_context,
            access_granted: true,
            quota_remaining,
            rate_limit_remaining,
            estimated_cost,
            warnings,
        })
    }

    /// Track API usage after successful request
    pub async fn track_usage(
        &self,
        api_key: &str,
        endpoint: &str,
        response_size_bytes: u64,
    ) -> Result<Decimal, ExternalAuthError> {
        // Get plan to calculate cost
        let api_key_info = self.api_key_repository
            .get_api_key(api_key)
            .await?
            .ok_or(ExternalAuthError::InvalidApiKey)?;

        let plan_features = self.plan_repository
            .get_plan_features(api_key_info.plan_id)
            .await?
            .ok_or(ExternalAuthError::PlanNotFound)?;

        // Calculate cost
        let request_cost = plan_features.cost_per_request;
        let data_cost = plan_features.cost_per_gb * Decimal::from(response_size_bytes) / Decimal::from(1_000_000_000);
        let total_cost = request_cost + data_cost;

        // Track the usage
        self.usage_tracker
            .track_api_request(api_key, endpoint, total_cost)
            .await?;

        Ok(total_cost)
    }

    /// Check if feature is available for plan
    pub async fn check_feature_access(
        &self,
        plan_id: i32,
        feature: &str,
    ) -> Result<FeatureAccess, ExternalAuthError> {
        let plan_features = self.plan_repository
            .get_plan_features(plan_id)
            .await?
            .ok_or(ExternalAuthError::PlanNotFound)?;

        let feature_access = plan_features.features
            .get(feature)
            .cloned()
            .unwrap_or(FeatureAccess {
                enabled: false,
                quota: None,
                rate_limit: None,
                requires_approval: false,
            });

        Ok(feature_access)
    }

    /// Generate usage analytics for API developer
    pub async fn get_usage_analytics(
        &self,
        api_key: &str,
    ) -> Result<ApiUsageAnalytics, ExternalAuthError> {
        let usage_stats = self.usage_tracker
            .get_current_usage(api_key)
            .await?;

        let api_key_info = self.api_key_repository
            .get_api_key(api_key)
            .await?
            .ok_or(ExternalAuthError::InvalidApiKey)?;

        let plan_features = self.plan_repository
            .get_plan_features(api_key_info.plan_id)
            .await?
            .ok_or(ExternalAuthError::PlanNotFound)?;

        Ok(ApiUsageAnalytics {
            current_usage: usage_stats.clone(),
            plan_limits: plan_features.rate_limits,
            cost_analysis: CostAnalysis {
                requests_cost: plan_features.cost_per_request * Decimal::from(usage_stats.requests_this_month),
                data_transfer_cost: plan_features.cost_per_gb * Decimal::from(usage_stats.data_transfer_mb) / Decimal::from(1000),
                total_cost: Decimal::ZERO, // Calculate total
            },
            efficiency_metrics: EfficiencyMetrics {
                avg_requests_per_day: usage_stats.requests_this_month / 30,
                peak_hour_usage: usage_stats.requests_this_hour,
                cost_per_request: plan_features.cost_per_request,
            },
        })
    }

    // Private helper methods

    fn validate_api_key_status(&self, api_key: &ApiKey) -> Result<(), ExternalAuthError> {
        match api_key.status {
            ApiKeyStatus::Active => Ok(()),
            ApiKeyStatus::Suspended => Err(ExternalAuthError::ApiKeySuspended),
            ApiKeyStatus::Revoked => Err(ExternalAuthError::ApiKeyRevoked),
            ApiKeyStatus::Expired => Err(ExternalAuthError::ApiKeyExpired),
        }
    }

    fn check_endpoint_access(&self, endpoint: &str, plan_features: &PlanFeatures) -> Result<FeatureAccess, ExternalAuthError> {
        // Map endpoints to features
        let feature_name = match endpoint {
            endpoint if endpoint.contains("/rankings") => "analytics_rankings",
            endpoint if endpoint.contains("/countries") => "analytics_countries", 
            endpoint if endpoint.contains("/webhooks") => "webhook_support",
            endpoint if endpoint.contains("/bulk") => "bulk_operations",
            _ => "basic_api_access",
        };

        let feature_access = plan_features.features
            .get(feature_name)
            .cloned()
            .unwrap_or(FeatureAccess {
                enabled: false,
                quota: None,
                rate_limit: None,
                requires_approval: false,
            });

        Ok(feature_access)
    }

    fn is_quota_exceeded(&self, usage: &UsageStats, limits: &UsageLimits) -> bool {
        if let Some(daily_limit) = limits.requests_per_day {
            if usage.requests_today >= daily_limit {
                return true;
            }
        }

        if let Some(monthly_limit) = limits.requests_per_month {
            if usage.requests_this_month >= monthly_limit {
                return true;
            }
        }

        false
    }

    fn calculate_quota_remaining(&self, usage: &UsageStats, limits: &UsageLimits) -> u32 {
        if let Some(daily_limit) = limits.requests_per_day {
            return daily_limit.saturating_sub(usage.requests_today);
        }

        if let Some(monthly_limit) = limits.requests_per_month {
            return monthly_limit.saturating_sub(usage.requests_this_month);
        }

        u32::MAX
    }

    fn calculate_rate_limit_remaining(&self, usage: &UsageStats, limits: &UsageLimits) -> u32 {
        if let Some(minute_limit) = limits.requests_per_minute {
            return minute_limit.saturating_sub(usage.requests_this_minute);
        }

        u32::MAX
    }

    fn generate_usage_warnings(&self, usage: &UsageStats, limits: &UsageLimits) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check if approaching daily limit (80% threshold)
        if let Some(daily_limit) = limits.requests_per_day {
            let usage_percentage = (usage.requests_today as f32 / daily_limit as f32) * 100.0;
            if usage_percentage > 80.0 {
                warnings.push(format!("Approaching daily limit: {:.1}% used", usage_percentage));
            }
        }

        // Check if approaching monthly limit (90% threshold)
        if let Some(monthly_limit) = limits.requests_per_month {
            let usage_percentage = (usage.requests_this_month as f32 / monthly_limit as f32) * 100.0;
            if usage_percentage > 90.0 {
                warnings.push(format!("Approaching monthly limit: {:.1}% used", usage_percentage));
            }
        }

        warnings
    }
}

// Analytics types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageAnalytics {
    pub current_usage: UsageStats,
    pub plan_limits: UsageLimits,
    pub cost_analysis: CostAnalysis,
    pub efficiency_metrics: EfficiencyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    pub requests_cost: Decimal,
    pub data_transfer_cost: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub avg_requests_per_day: u32,
    pub peak_hour_usage: u32,
    pub cost_per_request: Decimal,
}