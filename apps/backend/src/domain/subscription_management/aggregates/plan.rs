use crate::prelude::*;
use crate::domain::shared_kernel::{AggregateRoot, AggregateBase, DomainEvent};
use crate::domain::subscription_management::{PlanId, Price, BillingCycle, PlanFeatures};
use crate::domain::permission_management::GroupId;

/// Plan Aggregate Root
/// Represents a subscription plan with pricing, features, and permission group association
#[derive(Debug, Clone)]
pub struct Plan {
    id: PlanId,
    name: String,
    description: String,
    permission_group_id: GroupId,
    price: Price,
    billing_cycle: BillingCycle,
    features: PlanFeatures,
    target_audience: String,
    is_active: bool,
    is_promoted: bool,
    display_order: i32,
    metadata: serde_json::Value,
    base: AggregateBase,
}

pub struct CreatePlanParams {
    pub name: String,
    pub description: String,
    pub permission_group_id: GroupId,
    pub price: Price,
    pub billing_cycle: BillingCycle,
    pub features: PlanFeatures,
    pub target_audience: String,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

pub struct LoadPlanParams {
    pub id: PlanId,
    pub name: String,
    pub description: String,
    pub permission_group_id: GroupId,
    pub price: Price,
    pub billing_cycle: BillingCycle,
    pub features: PlanFeatures,
    pub target_audience: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

#[derive(Default)]
pub struct UpdatePlanParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<Price>,
    pub billing_cycle: Option<BillingCycle>,
    pub features: Option<PlanFeatures>,
    pub target_audience: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

impl Plan {
    /// Create a new plan
    pub fn create(id: PlanId, params: CreatePlanParams) -> AppResult<Self> {
        Ok(Self {
            id,
            name: params.name,
            description: params.description,
            permission_group_id: params.permission_group_id,
            price: params.price,
            billing_cycle: params.billing_cycle,
            features: params.features,
            target_audience: params.target_audience,
            is_active: params.is_active.unwrap_or(true),
            is_promoted: params.is_promoted.unwrap_or(false),
            display_order: params.display_order.unwrap_or(0),
            metadata: params.metadata.unwrap_or_else(|| serde_json::json!({})),
            base: AggregateBase::new(),
        })
    }

    /// Load existing plan from database
    pub fn load(params: LoadPlanParams) -> Self {
        Self {
            id: params.id,
            name: params.name,
            description: params.description,
            permission_group_id: params.permission_group_id,
            price: params.price,
            billing_cycle: params.billing_cycle,
            features: params.features,
            target_audience: params.target_audience,
            is_active: params.is_active,
            is_promoted: params.is_promoted,
            display_order: params.display_order,
            metadata: params.metadata,
            base: AggregateBase {
                version: params.version,
                created_at: params.created_at,
                updated_at: params.updated_at,
                events: Vec::new(),
            },
        }
    }

    /// Update plan details
    pub fn update(&mut self, params: UpdatePlanParams) -> AppResult<()> {
        if let Some(n) = params.name {
            self.name = n;
        }
        if let Some(d) = params.description {
            self.description = d;
        }
        if let Some(p) = params.price {
            self.price = p;
        }
        if let Some(bc) = params.billing_cycle {
            self.billing_cycle = bc;
        }
        if let Some(f) = params.features {
            self.features = f;
        }
        if let Some(ta) = params.target_audience {
            self.target_audience = ta;
        }
        if let Some(active) = params.is_active {
            self.is_active = active;
        }
        if let Some(promoted) = params.is_promoted {
            self.is_promoted = promoted;
        }
        if let Some(order) = params.display_order {
            self.display_order = order;
        }
        if let Some(meta) = params.metadata {
            self.metadata = meta;
        }

        self.base.touch();
        self.base.increment_version();

        Ok(())
    }

    /// Activate plan
    pub fn activate(&mut self) {
        self.is_active = true;
        self.base.touch();
    }

    /// Deactivate plan
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.base.touch();
    }

    // Getters
    pub fn id(&self) -> &PlanId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn permission_group_id(&self) -> &GroupId {
        &self.permission_group_id
    }

    pub fn price(&self) -> &Price {
        &self.price
    }

    pub fn billing_cycle(&self) -> &BillingCycle {
        &self.billing_cycle
    }

    pub fn features(&self) -> &PlanFeatures {
        &self.features
    }

    pub fn target_audience(&self) -> &str {
        &self.target_audience
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_promoted(&self) -> bool {
        self.is_promoted
    }

    pub fn display_order(&self) -> i32 {
        self.display_order
    }

    pub fn metadata(&self) -> &serde_json::Value {
        &self.metadata
    }
}

impl AggregateRoot for Plan {
    type Id = PlanId;

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
