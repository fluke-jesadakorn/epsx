// Domain Events for Resource Management
// Events that occur within the resource management domain

use crate::prelude::*;
use crate::domain::shared_kernel::{DomainEvent, EventMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Resource usage events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageExceeded {
    pub metadata: EventMetadata,
    pub wallet_address: String,
    pub plan_id: Option<i32>,
    pub resource_type: String,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub overage_amount: i64,
    pub access_context: String,
}

impl ResourceUsageExceeded {
    pub fn new(
        wallet_address: String,
        plan_id: Option<i32>,
        resource_type: String,
        current_usage: i64,
        quota_limit: i64,
        access_context: String,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.clone(), 1),
            wallet_address: wallet_address.clone(),
            plan_id,
            resource_type,
            current_usage,
            quota_limit,
            overage_amount: current_usage - quota_limit,
            access_context,
        }
    }
}

impl DomainEvent for ResourceUsageExceeded {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "ResourceUsageExceeded"
    }

    fn aggregate_type(&self) -> &'static str {
        "Resource"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageWarning {
    pub metadata: EventMetadata,
    pub wallet_address: String,
    pub plan_id: Option<i32>,
    pub resource_type: String,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub usage_percentage: f64,
    pub access_context: String,
}

impl ResourceUsageWarning {
    pub fn new(
        wallet_address: String,
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
            metadata: EventMetadata::new(wallet_address.clone(), 1),
            wallet_address,
            plan_id,
            resource_type,
            current_usage,
            quota_limit,
            usage_percentage,
            access_context,
        }
    }
}

impl DomainEvent for ResourceUsageWarning {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "ResourceUsageWarning"
    }

    fn aggregate_type(&self) -> &'static str {
        "Resource"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanUpgradeRecommended {
    pub metadata: EventMetadata,
    pub wallet_address: String,
    pub current_plan_id: Option<i32>,
    pub recommended_plan_id: i32,
    pub reason: String,
    pub potential_savings: Option<f64>,
}

impl PlanUpgradeRecommended {
    pub fn new(
        wallet_address: String,
        current_plan_id: Option<i32>,
        recommended_plan_id: i32,
        reason: String,
        potential_savings: Option<f64>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.clone(), 1),
            wallet_address,
            current_plan_id,
            recommended_plan_id,
            reason,
            potential_savings,
        }
    }
}

impl DomainEvent for PlanUpgradeRecommended {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "PlanUpgradeRecommended"
    }

    fn aggregate_type(&self) -> &'static str {
        "Resource"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingCalculated {
    pub metadata: EventMetadata,
    pub wallet_address: String,
    pub plan_id: Option<i32>,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub base_cost: f64,
    pub overage_costs: HashMap<String, f64>,
    pub total_cost: f64,
    pub currency: String,
}

impl BillingCalculated {
    pub fn new(
        wallet_address: String,
        plan_id: Option<i32>,
        billing_period_start: DateTime<Utc>,
        billing_period_end: DateTime<Utc>,
        base_cost: f64,
        overage_costs: HashMap<String, f64>,
        total_cost: f64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.clone(), 1),
            wallet_address,
            plan_id,
            billing_period_start,
            billing_period_end,
            base_cost,
            overage_costs,
            total_cost,
            currency: "USD".to_string(),
        }
    }
}

impl DomainEvent for BillingCalculated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "BillingCalculated"
    }

    fn aggregate_type(&self) -> &'static str {
        "Resource"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatternDetected {
    pub metadata: EventMetadata,
    pub wallet_address: String,
    pub pattern_type: UsagePattern,
    pub resource_types: Vec<String>,
    pub confidence_score: f64,
    pub recommendations: Vec<String>,
}

impl UsagePatternDetected {
    pub fn new(
        wallet_address: String,
        pattern_type: UsagePattern,
        resource_types: Vec<String>,
        confidence_score: f64,
        recommendations: Vec<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(wallet_address.clone(), 1),
            wallet_address,
            pattern_type,
            resource_types,
            confidence_score,
            recommendations,
        }
    }
}

impl DomainEvent for UsagePatternDetected {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "UsagePatternDetected"
    }

    fn aggregate_type(&self) -> &'static str {
        "Resource"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsagePattern {
    HighPeakUsage,
    ConsistentOverage,
    UnderUtilization,
    IrregularSpikes,
    SteadyGrowth,
}
