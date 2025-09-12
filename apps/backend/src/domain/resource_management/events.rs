// Domain Events for Resource Management
// Events that occur within the resource management domain

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Base trait for all domain events
pub trait DomainEvent {
    fn event_type(&self) -> &'static str;
    fn event_id(&self) -> String;
    fn user_id(&self) -> String;
    fn timestamp(&self) -> DateTime<Utc>;
}

/// Resource usage events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageExceeded {
    pub event_id: String,
    pub user_id: String,
    pub plan_id: Option<i32>,
    pub resource_type: String,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub overage_amount: i64,
    pub access_context: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageWarning {
    pub event_id: String,
    pub user_id: String,
    pub plan_id: Option<i32>,
    pub resource_type: String,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub usage_percentage: f64,
    pub access_context: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanUpgradeRecommended {
    pub event_id: String,
    pub user_id: String,
    pub current_plan_id: Option<i32>,
    pub recommended_plan_id: i32,
    pub reason: String,
    pub potential_savings: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingCalculated {
    pub event_id: String,
    pub user_id: String,
    pub plan_id: Option<i32>,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub base_cost: f64,
    pub overage_costs: HashMap<String, f64>,
    pub total_cost: f64,
    pub currency: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatternDetected {
    pub event_id: String,
    pub user_id: String,
    pub pattern_type: UsagePattern,
    pub resource_types: Vec<String>,
    pub confidence_score: f64,
    pub recommendations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsagePattern {
    HighPeakUsage,
    ConsistentOverage,
    UnderUtilization,
    IrregularSpikes,
    SteadyGrowth,
}

// Domain event implementations
impl DomainEvent for ResourceUsageExceeded {
    fn event_type(&self) -> &'static str {
        "resource_usage_exceeded"
    }

    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn user_id(&self) -> String {
        self.user_id.clone()
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

impl DomainEvent for ResourceUsageWarning {
    fn event_type(&self) -> &'static str {
        "resource_usage_warning"
    }

    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn user_id(&self) -> String {
        self.user_id.clone()
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

impl DomainEvent for PlanUpgradeRecommended {
    fn event_type(&self) -> &'static str {
        "plan_upgrade_recommended"
    }

    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn user_id(&self) -> String {
        self.user_id.clone()
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

impl DomainEvent for BillingCalculated {
    fn event_type(&self) -> &'static str {
        "billing_calculated"
    }

    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn user_id(&self) -> String {
        self.user_id.clone()
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

impl DomainEvent for UsagePatternDetected {
    fn event_type(&self) -> &'static str {
        "usage_pattern_detected"
    }

    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn user_id(&self) -> String {
        self.user_id.clone()
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

// Factory methods for creating events
impl ResourceUsageExceeded {
    pub fn new(
        user_id: String,
        plan_id: Option<i32>,
        resource_type: String,
        current_usage: i64,
        quota_limit: i64,
        access_context: String,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            plan_id,
            resource_type,
            current_usage,
            quota_limit,
            overage_amount: current_usage - quota_limit,
            access_context,
            timestamp: Utc::now(),
        }
    }
}

impl ResourceUsageWarning {
    pub fn new(
        user_id: String,
        plan_id: Option<i32>,
        resource_type: String,
        current_usage: i64,
        quota_limit: i64,
        access_context: String,
    ) -> Self {
        let usage_percentage = if quota_limit > 0 {
            (current_usage as f64 / quota_limit as f64) * 100.0
        } else {
            0.0
        };

        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            plan_id,
            resource_type,
            current_usage,
            quota_limit,
            usage_percentage,
            access_context,
            timestamp: Utc::now(),
        }
    }
}

impl BillingCalculated {
    pub fn new(
        user_id: String,
        plan_id: Option<i32>,
        billing_period_start: DateTime<Utc>,
        billing_period_end: DateTime<Utc>,
        base_cost: f64,
        overage_costs: HashMap<String, f64>,
        total_cost: f64,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            plan_id,
            billing_period_start,
            billing_period_end,
            base_cost,
            overage_costs,
            total_cost,
            currency: "USD".to_string(),
            timestamp: Utc::now(),
        }
    }
}