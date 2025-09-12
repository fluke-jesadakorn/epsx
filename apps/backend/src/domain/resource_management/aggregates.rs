// Resource Management Aggregates
// Domain aggregates for resource management bounded context

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Represents a user's resource consumption aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResourceUsage {
    pub user_id: String,
    pub plan_id: Option<i32>,
    pub access_context: String,
    pub current_usage: HashMap<String, i64>,
    pub quota_limits: HashMap<String, i64>,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl UserResourceUsage {
    pub fn new(
        user_id: String,
        plan_id: Option<i32>,
        access_context: String,
        quota_limits: HashMap<String, i64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            plan_id,
            access_context,
            current_usage: HashMap::new(),
            quota_limits,
            billing_period_start: now,
            billing_period_end: now,
            last_updated: now,
        }
    }

    pub fn is_limit_exceeded(&self, resource_type: &str) -> bool {
        let current = self.current_usage.get(resource_type).unwrap_or(&0);
        let limit = self.quota_limits.get(resource_type).unwrap_or(&0);
        
        if *limit == 0 {
            return false; // Unlimited
        }
        
        current >= limit
    }

    pub fn get_usage_percentage(&self, resource_type: &str) -> f64 {
        let current = self.current_usage.get(resource_type).unwrap_or(&0);
        let limit = self.quota_limits.get(resource_type).unwrap_or(&0);
        
        if *limit == 0 {
            return 0.0; // Unlimited
        }
        
        (*current as f64 / *limit as f64) * 100.0
    }

    pub fn increment_usage(&mut self, resource_type: String, amount: i64) {
        let current = self.current_usage.entry(resource_type).or_insert(0);
        *current += amount;
        self.last_updated = Utc::now();
    }
}

/// Represents a plan's resource configuration aggregate  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResourceConfig {
    pub plan_id: i32,
    pub plan_name: String,
    pub access_context: String,
    pub resource_limits: HashMap<String, i64>,
    pub rate_limits: HashMap<String, i32>,
    pub overage_pricing: HashMap<String, f64>,
    pub is_active: bool,
}

impl PlanResourceConfig {
    pub fn new(plan_id: i32, plan_name: String, access_context: String) -> Self {
        Self {
            plan_id,
            plan_name,
            access_context,
            resource_limits: HashMap::new(),
            rate_limits: HashMap::new(),
            overage_pricing: HashMap::new(),
            is_active: true,
        }
    }

    pub fn set_resource_limit(&mut self, resource_type: String, limit: i64) {
        self.resource_limits.insert(resource_type, limit);
    }

    pub fn set_rate_limit(&mut self, endpoint: String, limit: i32) {
        self.rate_limits.insert(endpoint, limit);
    }

    pub fn set_overage_price(&mut self, resource_type: String, price: f64) {
        self.overage_pricing.insert(resource_type, price);
    }
}