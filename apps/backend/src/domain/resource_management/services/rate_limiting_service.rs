// Rate limiting service with context-aware strategies
// Implements different rate limiting approaches for internal vs external vs admin access

use std::{sync::Arc, collections::HashMap};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::domain::{
    shared_kernel::domain_error::DomainError,
    resource_management::value_objects::{
        resource_type::{ResourceType, ResourceCategory},
        usage_metrics::{RealTimeUsageTracker},
    },
};

use super::resource_tracking_service::{RealTimeCachePort};

/// Context-aware rate limiting service
pub struct RateLimitingService {
    real_time_cache: Arc<dyn RealTimeCachePort>,
    plan_repository: Arc<dyn PlanRepositoryPort>,
    rate_limit_config: Arc<RateLimitConfig>,
}

/// Rate limit configuration for different access contexts
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub internal_limits: InternalRateLimits,
    pub external_limits: ExternalRateLimits, 
    pub admin_limits: AdminRateLimits,
}

/// Rate limits for internal web app users (lenient, UX-focused)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalRateLimits {
    pub requests_per_minute: u64,     // 120 requests/minute (2 per second)
    pub requests_per_hour: u64,       // 3600 requests/hour (1 per second average)
    pub requests_per_day: u64,        // 50000 requests/day
    pub burst_allowance: u64,         // 20 requests burst
    pub analytics_requests_per_hour: u64, // 300 analytics queries/hour
    pub enable_user_feedback: bool,   // Show friendly rate limit messages
}

/// Rate limits for external API access (strict, plan-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalRateLimits {
    pub base_requests_per_minute: u64,  // Plan-based: 10-1000 requests/minute
    pub base_requests_per_day: u64,     // Plan-based: 1000-1000000 requests/day
    pub burst_multiplier: f64,          // 1.5x burst allowance
    pub cost_per_request: Decimal,      // Dynamic pricing per request
    pub enable_quota_warnings: bool,    // Warn at 80% quota usage
    pub hard_limit_enforcement: bool,   // Enforce strict limits
    pub overage_allowed: bool,          // Allow paid overages
    pub overage_cost_multiplier: Decimal, // 2.0x cost for overages
}

/// Rate limits for admin interface (moderate, audit-focused)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRateLimits {
    pub requests_per_minute: u64,      // 60 requests/minute
    pub requests_per_hour: u64,        // 2000 requests/hour
    pub bulk_operations_per_hour: u64, // 20 bulk operations/hour
    pub user_management_per_minute: u64, // 30 user operations/minute
    pub system_queries_per_minute: u64, // 100 system queries/minute
    pub audit_everything: bool,         // Log all admin actions
}

/// Rate limit check request
#[derive(Debug, Clone)]
pub struct RateLimitRequest {
    pub identifier: String,           // user_id, api_key, or admin_session
    pub identifier_type: IdentifierType,
    pub access_context: AccessContext,
    pub resource_type: ResourceType,
    pub quantity: u64,                // Usually 1, but can be batch requests
    pub plan_id: Option<i32>,        // For plan-based limiting
    pub session_metadata: Option<SessionMetadata>,
}

#[derive(Debug, Clone)]
pub enum IdentifierType {
    UserId,
    ApiKey,
    AdminSession,
}

#[derive(Debug, Clone)]
pub enum AccessContext {
    Internal,   // Web app users
    External,   // API developers  
    Admin,      // Admin interface
}

#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub session_start: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Rate limit check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub current_usage: RateLimitUsage,
    pub limits: RateLimitQuotas,
    pub retry_after_seconds: Option<u64>,
    pub cost_impact: Option<CostImpact>,
    pub warnings: Vec<RateLimitWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitUsage {
    pub current_minute: u64,
    pub current_hour: u64,
    pub current_day: u64,
    pub current_month: u64,
    pub burst_usage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitQuotas {
    pub minute_limit: u64,
    pub hour_limit: u64,
    pub day_limit: u64,
    pub month_limit: Option<u64>,
    pub burst_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostImpact {
    pub base_cost: Decimal,
    pub overage_cost: Decimal,
    pub total_estimated_cost: Decimal,
    pub quota_percentage_used: f64,
    pub projected_monthly_cost: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitWarning {
    pub warning_type: WarningType,
    pub message: String,
    pub severity: WarningSeverity,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningType {
    ApproachingLimit,    // 80% of quota used
    CostThresholdExceeded, // Cost limit reached
    UnusualUsagePattern,   // Spike in usage
    PlanUpgradeRecommended, // Suggest better plan
    BurstQuotaExhausted,   // Burst quota used up
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Critical,
}

/// Repository for plan-based limits
#[async_trait]
pub trait PlanRepositoryPort: Send + Sync {
    async fn get_plan_limits(&self, plan_id: i32, context: AccessContext) -> Result<Option<PlanLimits>, DomainError>;
    async fn get_user_current_plan(&self, user_id: &str) -> Result<Option<i32>, DomainError>;
    async fn get_api_key_plan(&self, api_key: &str) -> Result<Option<i32>, DomainError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanLimits {
    pub plan_id: i32,
    pub requests_per_minute: u64,
    pub requests_per_hour: u64,
    pub requests_per_day: u64,
    pub requests_per_month: Option<u64>,
    pub burst_multiplier: f64,
    pub cost_per_request: Decimal,
    pub overage_allowed: bool,
    pub custom_features: HashMap<String, serde_json::Value>,
}

#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Rate limit exceeded for {context:?} access")]
    LimitExceeded { context: AccessContext },
    
    #[error("Plan not found for identifier: {identifier}")]
    PlanNotFound { identifier: String },
    
    #[error("Invalid rate limit configuration")]
    InvalidConfiguration,
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
}

impl RateLimitingService {
    pub fn new(
        real_time_cache: Arc<dyn RealTimeCachePort>,
        plan_repository: Arc<dyn PlanRepositoryPort>,
        rate_limit_config: Arc<RateLimitConfig>,
    ) -> Self {
        Self {
            real_time_cache,
            plan_repository,
            rate_limit_config,
        }
    }

    /// Check if a request should be allowed based on rate limits
    pub async fn check_rate_limit(
        &self,
        request: RateLimitRequest,
    ) -> Result<RateLimitResult, RateLimitError> {
        match request.access_context {
            AccessContext::Internal => {
                self.check_internal_rate_limit(request).await
            }
            AccessContext::External => {
                self.check_external_rate_limit(request).await
            }
            AccessContext::Admin => {
                self.check_admin_rate_limit(request).await
            }
        }
    }

    /// Internal rate limit check (lenient, UX-focused)
    async fn check_internal_rate_limit(
        &self,
        request: RateLimitRequest,
    ) -> Result<RateLimitResult, RateLimitError> {
        let limits = &self.rate_limit_config.internal_limits;
        
        // Get current usage
        let current_usage = self.get_current_usage(&request.identifier).await?;
        
        // Check limits with burst allowance
        let minute_allowed = current_usage.current_minute < (limits.requests_per_minute + limits.burst_allowance);
        let hour_allowed = current_usage.current_hour < limits.requests_per_hour;
        let day_allowed = current_usage.current_day < limits.requests_per_day;
        
        let allowed = minute_allowed && hour_allowed && day_allowed;
        
        // Generate user-friendly warnings
        let mut warnings = Vec::new();
        if current_usage.current_hour > (limits.requests_per_hour as f64 * 0.8) as u64 {
            warnings.push(RateLimitWarning {
                warning_type: WarningType::ApproachingLimit,
                message: "You're approaching your hourly usage limit. Consider taking a break.".to_string(),
                severity: WarningSeverity::Info,
                suggested_action: Some("Try refreshing in a few minutes".to_string()),
            });
        }
        
        Ok(RateLimitResult {
            allowed,
            current_usage,
            limits: RateLimitQuotas {
                minute_limit: limits.requests_per_minute + limits.burst_allowance,
                hour_limit: limits.requests_per_hour,
                day_limit: limits.requests_per_day,
                month_limit: None,
                burst_limit: limits.burst_allowance,
            },
            retry_after_seconds: if !allowed { Some(60) } else { None },
            cost_impact: None, // Internal usage is not billable
            warnings,
        })
    }

    /// External API rate limit check (strict, plan-based)
    async fn check_external_rate_limit(
        &self,
        request: RateLimitRequest,
    ) -> Result<RateLimitResult, RateLimitError> {
        // Get plan-specific limits
        let plan_id = request.plan_id.ok_or_else(|| RateLimitError::PlanNotFound { 
            identifier: request.identifier.clone() 
        })?;
        
        let plan_limits = self.plan_repository
            .get_plan_limits(plan_id, AccessContext::External)
            .await?
            .ok_or_else(|| RateLimitError::PlanNotFound { 
                identifier: request.identifier.clone() 
            })?;
        
        let current_usage = self.get_current_usage(&request.identifier).await?;
        
        // Calculate burst allowance
        let burst_allowance = (plan_limits.requests_per_minute as f64 * plan_limits.burst_multiplier) as u64;
        
        // Check all time windows
        let minute_allowed = current_usage.current_minute < (plan_limits.requests_per_minute + burst_allowance);
        let hour_allowed = current_usage.current_hour < plan_limits.requests_per_hour;
        let day_allowed = current_usage.current_day < plan_limits.requests_per_day;
        
        let month_allowed = if let Some(month_limit) = plan_limits.requests_per_month {
            current_usage.current_month < month_limit
        } else {
            true
        };
        
        let allowed = minute_allowed && hour_allowed && day_allowed && month_allowed;
        
        // Calculate cost impact
        let base_cost = plan_limits.cost_per_request * Decimal::from(request.quantity);
        let cost_impact = CostImpact {
            base_cost,
            overage_cost: Decimal::ZERO, // TODO: Calculate if overages are used
            total_estimated_cost: base_cost,
            quota_percentage_used: (current_usage.current_day as f64 / plan_limits.requests_per_day as f64) * 100.0,
            projected_monthly_cost: base_cost * Decimal::from(30), // Rough estimate
        };
        
        // Generate business-focused warnings
        let mut warnings = Vec::new();
        if cost_impact.quota_percentage_used > 80.0 {
            warnings.push(RateLimitWarning {
                warning_type: WarningType::ApproachingLimit,
                message: format!("You've used {:.1}% of your daily quota", cost_impact.quota_percentage_used),
                severity: WarningSeverity::Warning,
                suggested_action: Some("Consider upgrading your plan".to_string()),
            });
        }
        
        if cost_impact.projected_monthly_cost > Decimal::from(100) {
            warnings.push(RateLimitWarning {
                warning_type: WarningType::PlanUpgradeRecommended,
                message: "Your usage pattern suggests a higher tier plan would be more cost-effective".to_string(),
                severity: WarningSeverity::Info,
                suggested_action: Some("Contact support for plan recommendations".to_string()),
            });
        }
        
        Ok(RateLimitResult {
            allowed,
            current_usage,
            limits: RateLimitQuotas {
                minute_limit: plan_limits.requests_per_minute + burst_allowance,
                hour_limit: plan_limits.requests_per_hour,
                day_limit: plan_limits.requests_per_day,
                month_limit: plan_limits.requests_per_month,
                burst_limit: burst_allowance,
            },
            retry_after_seconds: if !allowed { Some(60) } else { None },
            cost_impact: Some(cost_impact),
            warnings,
        })
    }

    /// Admin rate limit check (moderate, audit-focused)
    async fn check_admin_rate_limit(
        &self,
        request: RateLimitRequest,
    ) -> Result<RateLimitResult, RateLimitError> {
        let limits = &self.rate_limit_config.admin_limits;
        let current_usage = self.get_current_usage(&request.identifier).await?;
        
        // Different limits for different admin operations
        let (minute_limit, hour_limit) = match request.resource_type.category() {
            ResourceCategory::UserManagement => (limits.user_management_per_minute, limits.requests_per_hour),
            ResourceCategory::BulkOperations => (10, limits.bulk_operations_per_hour), // Very restrictive
            ResourceCategory::SystemQueries => (limits.system_queries_per_minute, limits.requests_per_hour),
            _ => (limits.requests_per_minute, limits.requests_per_hour),
        };
        
        let minute_allowed = current_usage.current_minute < minute_limit;
        let hour_allowed = current_usage.current_hour < hour_limit;
        
        let allowed = minute_allowed && hour_allowed;
        
        // Generate admin-specific warnings
        let mut warnings = Vec::new();
        if current_usage.current_hour > (hour_limit as f64 * 0.9) as u64 {
            warnings.push(RateLimitWarning {
                warning_type: WarningType::ApproachingLimit,
                message: "Approaching admin operation limit. All actions are being audited.".to_string(),
                severity: WarningSeverity::Warning,
                suggested_action: Some("Consider batching operations or waiting".to_string()),
            });
        }
        
        Ok(RateLimitResult {
            allowed,
            current_usage,
            limits: RateLimitQuotas {
                minute_limit,
                hour_limit,
                day_limit: u64::MAX, // No daily limit for admins
                month_limit: None,
                burst_limit: 0, // No burst for admins
            },
            retry_after_seconds: if !allowed { Some(60) } else { None },
            cost_impact: None, // Admin usage is not billable
            warnings,
        })
    }

    /// Get current usage from cache
    async fn get_current_usage(&self, identifier: &str) -> Result<RateLimitUsage, RateLimitError> {
        let tracker = self.real_time_cache
            .get_real_time_usage(identifier)
            .await
            .map_err(|e| RateLimitError::CacheError(e.to_string()))?;
        
        if let Some(tracker) = tracker {
            Ok(RateLimitUsage {
                current_minute: tracker.current_minute.total_requests,
                current_hour: tracker.current_hour.total_requests,
                current_day: tracker.current_day.total_requests,
                current_month: tracker.current_month.total_requests,
                burst_usage: tracker.current_minute.total_requests, // Simple burst calculation
            })
        } else {
            // No usage yet
            Ok(RateLimitUsage {
                current_minute: 0,
                current_hour: 0,
                current_day: 0,
                current_month: 0,
                burst_usage: 0,
            })
        }
    }

    /// Record successful request (update counters)
    pub async fn record_request(
        &self,
        identifier: &str,
        resource_type: ResourceType,
        cost: Decimal,
    ) -> Result<(), RateLimitError> {
        let mut tracker = self.real_time_cache
            .get_real_time_usage(identifier)
            .await
            .map_err(|e| RateLimitError::CacheError(e.to_string()))?
            .unwrap_or_else(|| RealTimeUsageTracker::new(identifier.to_string()));
        
        tracker.record_usage(resource_type, cost);
        
        self.real_time_cache
            .update_real_time_usage(&tracker)
            .await
            .map_err(|e| RateLimitError::CacheError(e.to_string()))?;
        
        Ok(())
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            internal_limits: InternalRateLimits {
                requests_per_minute: 120,
                requests_per_hour: 3600,
                requests_per_day: 50000,
                burst_allowance: 20,
                analytics_requests_per_hour: 300,
                enable_user_feedback: true,
            },
            external_limits: ExternalRateLimits {
                base_requests_per_minute: 60,
                base_requests_per_day: 10000,
                burst_multiplier: 1.5,
                cost_per_request: Decimal::from_str("0.001").unwrap(),
                enable_quota_warnings: true,
                hard_limit_enforcement: true,
                overage_allowed: false,
                overage_cost_multiplier: Decimal::from(2),
            },
            admin_limits: AdminRateLimits {
                requests_per_minute: 60,
                requests_per_hour: 2000,
                bulk_operations_per_hour: 20,
                user_management_per_minute: 30,
                system_queries_per_minute: 100,
                audit_everything: true,
            },
        }
    }
}

use std::str::FromStr;