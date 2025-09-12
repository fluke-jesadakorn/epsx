// Plan repository adapter for rate limiting
// Simplified mock implementation to avoid complex Diesel async issues

use std::sync::Arc;
use async_trait::async_trait;
use std::collections::HashMap;
use rust_decimal::Decimal;

use crate::{
    domain::{
        shared_kernel::domain_error::DomainError,
        resource_management::services::{
            rate_limiting_service::{PlanRepositoryPort, PlanLimits, AccessContext},
        },
    },
    infrastructure::adapters::repositories::diesel::DbPool,
};

/// Repository adapter that implements plan-based rate limiting using mock data
pub struct PlanRepositoryAdapter {
    db_pool: Arc<DbPool>,
}

impl PlanRepositoryAdapter {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl PlanRepositoryPort for PlanRepositoryAdapter {
    async fn get_plan_limits(&self, plan_id: i32, context: AccessContext) -> Result<Option<PlanLimits>, DomainError> {
        // Simplified mock implementation to avoid complex Diesel async issues
        // In production, this would connect to the database properly
        
        // Mock limits based on plan_id and context
        match (plan_id, context) {
            (1, AccessContext::Internal) => {
                Ok(Some(PlanLimits {
                    plan_id,
                    requests_per_minute: 100,
                    requests_per_hour: 3600,
                    requests_per_day: 50000,
                    requests_per_month: Some(1500000),
                    burst_multiplier: 1.5,
                    cost_per_request: Decimal::new(0, 0), // Free
                    overage_allowed: false,
                    custom_features: HashMap::new(),
                }))
            }
            (2, AccessContext::External) => {
                Ok(Some(PlanLimits {
                    plan_id,
                    requests_per_minute: 50,
                    requests_per_hour: 1000,
                    requests_per_day: 10000,
                    requests_per_month: Some(300000),
                    burst_multiplier: 2.0,
                    cost_per_request: Decimal::new(1, 3), // $0.001
                    overage_allowed: true,
                    custom_features: HashMap::new(),
                }))
            }
            (3, AccessContext::Admin) => {
                Ok(Some(PlanLimits {
                    plan_id,
                    requests_per_minute: 200,
                    requests_per_hour: 5000,
                    requests_per_day: 100000,
                    requests_per_month: None, // Unlimited
                    burst_multiplier: 3.0,
                    cost_per_request: Decimal::new(0, 0), // Free
                    overage_allowed: false,
                    custom_features: HashMap::new(),
                }))
            }
            _ => Ok(None), // Plan not found
        }
    }

    async fn get_user_current_plan(&self, user_id: &str) -> Result<Option<i32>, DomainError> {
        // Mock implementation - map user patterns to plans
        if user_id.contains("admin") {
            Ok(Some(3)) // Admin plan
        } else if user_id.starts_with("api_") {
            Ok(Some(2)) // External API plan
        } else {
            Ok(Some(1)) // Internal web plan
        }
    }

    async fn get_api_key_plan(&self, api_key: &str) -> Result<Option<i32>, DomainError> {
        // Mock implementation - all API keys get plan 2 for now
        if api_key.len() >= 8 {
            Ok(Some(2))
        } else {
            Ok(None)
        }
    }
}