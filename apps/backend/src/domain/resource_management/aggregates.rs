// Resource Management Aggregates
// Domain aggregates for resource management bounded context

use crate::prelude::*;
use crate::domain::shared_kernel::{AggregateRoot, AggregateBase, DomainEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Unique identifier for resource usage aggregate
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceUsageId(String);

impl ResourceUsageId {
    pub fn new(wallet_address: String) -> Self {
        Self(wallet_address)
    }

    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Represents a user's resource consumption aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResourceUsage {
    id: ResourceUsageId,
    pub(crate) wallet_address: String,
    pub(crate) plan_id: Option<i32>,
    pub(crate) access_context: String,
    pub(crate) current_usage: HashMap<String, i64>,
    pub(crate) quota_limits: HashMap<String, i64>,
    pub(crate) billing_period_start: DateTime<Utc>,
    pub(crate) billing_period_end: DateTime<Utc>,
    base: AggregateBase,
}

impl UserResourceUsage {
    pub fn new(
        wallet_address: String,
        plan_id: Option<i32>,
        access_context: String,
        quota_limits: HashMap<String, i64>,
    ) -> Self {
        let now = Utc::now();
        let id = ResourceUsageId::new(wallet_address.clone());

        Self {
            id,
            wallet_address,
            plan_id,
            access_context,
            current_usage: HashMap::new(),
            quota_limits,
            billing_period_start: now,
            billing_period_end: now,
            base: AggregateBase::new(),
        }
    }

    pub fn id(&self) -> &ResourceUsageId {
        &self.id
    }

    pub fn wallet_address(&self) -> &str {
        &self.wallet_address
    }

    pub fn plan_id(&self) -> Option<i32> {
        self.plan_id
    }

    pub fn access_context(&self) -> &str {
        &self.access_context
    }

    pub fn current_usage(&self) -> &HashMap<String, i64> {
        &self.current_usage
    }

    pub fn quota_limits(&self) -> &HashMap<String, i64> {
        &self.quota_limits
    }

    pub fn billing_period_start(&self) -> DateTime<Utc> {
        self.billing_period_start
    }

    pub fn billing_period_end(&self) -> DateTime<Utc> {
        self.billing_period_end
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

    pub fn increment_usage(&mut self, resource_type: String, amount: i64) -> AppResult<()> {
        let current = self.current_usage.entry(resource_type.clone()).or_insert(0);
        *current += amount;
        let current_value = *current;

        self.base.touch();
        self.base.increment_version();

        // Raise event if limit exceeded
        if self.is_limit_exceeded(&resource_type) {
            let quota = *self.quota_limits.get(&resource_type).unwrap_or(&0);
            self.base.add_event(Box::new(super::events::ResourceUsageExceeded::new(
                self.wallet_address.clone(),
                self.plan_id,
                resource_type.clone(),
                current_value,
                quota,
                self.access_context.clone(),
            )));
        }

        Ok(())
    }
}

impl AggregateRoot for UserResourceUsage {
    type Id = ResourceUsageId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.base.version
    }

    fn increment_version(&mut self) {
        self.base.increment_version();
    }

    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        self.base.uncommitted_events()
    }

    fn mark_events_as_committed(&mut self) {
        self.base.mark_events_as_committed();
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }

    fn touch(&mut self) {
        self.base.touch();
    }
}

/// Unique identifier for plan resource config aggregate
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlanResourceConfigId(i32);

impl PlanResourceConfigId {
    pub fn new(plan_id: i32) -> Self {
        Self(plan_id)
    }

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

/// Represents a plan's resource configuration aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResourceConfig {
    id: PlanResourceConfigId,
    plan_id: i32,
    plan_name: String,
    access_context: String,
    resource_limits: HashMap<String, i64>,
    rate_limits: HashMap<String, i32>,
    overage_pricing: HashMap<String, f64>,
    is_active: bool,
    base: AggregateBase,
}

impl PlanResourceConfig {
    pub fn new(plan_id: i32, plan_name: String, access_context: String) -> Self {
        let id = PlanResourceConfigId::new(plan_id);

        Self {
            id,
            plan_id,
            plan_name,
            access_context,
            resource_limits: HashMap::new(),
            rate_limits: HashMap::new(),
            overage_pricing: HashMap::new(),
            is_active: true,
            base: AggregateBase::new(),
        }
    }

    pub fn id(&self) -> &PlanResourceConfigId {
        &self.id
    }

    pub fn plan_id(&self) -> i32 {
        self.plan_id
    }

    pub fn plan_name(&self) -> &str {
        &self.plan_name
    }

    pub fn access_context(&self) -> &str {
        &self.access_context
    }

    pub fn resource_limits(&self) -> &HashMap<String, i64> {
        &self.resource_limits
    }

    pub fn rate_limits(&self) -> &HashMap<String, i32> {
        &self.rate_limits
    }

    pub fn overage_pricing(&self) -> &HashMap<String, f64> {
        &self.overage_pricing
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_resource_limit(&mut self, resource_type: String, limit: i64) -> AppResult<()> {
        self.resource_limits.insert(resource_type, limit);
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }

    pub fn set_rate_limit(&mut self, endpoint: String, limit: i32) -> AppResult<()> {
        self.rate_limits.insert(endpoint, limit);
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }

    pub fn set_overage_price(&mut self, resource_type: String, price: f64) -> AppResult<()> {
        self.overage_pricing.insert(resource_type, price);
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }

    pub fn deactivate(&mut self) -> AppResult<()> {
        self.is_active = false;
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }

    pub fn activate(&mut self) -> AppResult<()> {
        self.is_active = true;
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }
}

impl AggregateRoot for PlanResourceConfig {
    type Id = PlanResourceConfigId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.base.version
    }

    fn increment_version(&mut self) {
        self.base.increment_version();
    }

    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        self.base.uncommitted_events()
    }

    fn mark_events_as_committed(&mut self) {
        self.base.mark_events_as_committed();
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }

    fn touch(&mut self) {
        self.base.touch();
    }
}