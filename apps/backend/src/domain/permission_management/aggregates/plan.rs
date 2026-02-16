use crate::prelude::*;
use crate::domain::shared_kernel::{AggregateRoot, AggregateBase, DomainEvent};
use crate::domain::permission_management::{
    PlanId, PlanSlug, PermissionString, PlanCategory, PlanGroup,
    events::{PlanCreatedEvent, PlanUpdatedEvent}
};
use std::collections::HashSet;

/// Plan Aggregate Root
/// Represents a plan of permissions that can be assigned to wallets
#[derive(Debug, Clone)]
pub struct Plan {
    id: PlanId,
    name: String,
    slug: PlanSlug,
    description: String,
    plan_type: String,
    plan_category: PlanCategory,
    plan_group: PlanGroup,
    permissions: HashSet<PermissionString>,
    price: f64,
    currency: String,
    billing_cycle: String,
    is_active: bool,
    is_promoted: bool,
    tier_level: i32,
    max_members: Option<i32>,
    auto_assign_enabled: bool,
    metadata: serde_json::Value,
    is_public: bool,
    grace_period_hours: i32,
    base: AggregateBase,
}

pub struct CreatePlanParams {
    pub name: String,
    pub slug: PlanSlug,
    pub description: String,
    pub plan_type: String,
    pub plan_category: Option<PlanCategory>,
    pub plan_group: Option<PlanGroup>,
    pub permissions: Vec<PermissionString>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub tier_level: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub metadata: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub grace_period_hours: Option<i32>,
}

pub struct LoadPlanParams {
    pub id: PlanId,
    pub name: String,
    pub slug: PlanSlug,
    pub description: String,
    pub plan_type: String,
    pub plan_category: PlanCategory,
    pub plan_group: PlanGroup,
    pub permissions: HashSet<PermissionString>,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub tier_level: i32,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: bool,
    pub metadata: serde_json::Value,
    pub is_public: bool,
    pub grace_period_hours: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
}

#[derive(Default)]
pub struct UpdatePlanParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub plan_category: Option<PlanCategory>,
    pub plan_group: Option<PlanGroup>,
    pub permissions: Option<Vec<PermissionString>>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub tier_level: Option<i32>,
    pub max_members: Option<Option<i32>>,
    pub auto_assign_enabled: Option<bool>,
    pub metadata: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub grace_period_hours: Option<i32>,
}

impl Plan {
    /// Create a new plan
    pub fn create(params: CreatePlanParams) -> AppResult<Self> {
        let now = Utc::now();
        let id = PlanId::new();
        let permissions: HashSet<PermissionString> = params.permissions.into_iter().collect();

        let mut plan = Self {
            id: id.clone(),
            name: params.name.clone(),
            slug: params.slug.clone(),
            description: params.description.clone(),
            plan_type: params.plan_type.clone(),
            plan_category: params.plan_category.unwrap_or_default(),
            plan_group: params.plan_group.unwrap_or_default(),
            permissions: permissions.clone(),
            price: params.price.unwrap_or(0.0),
            currency: params.currency.unwrap_or_else(|| "USD".to_string()),
            billing_cycle: params.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
            is_active: params.is_active.unwrap_or(true),
            is_promoted: params.is_promoted.unwrap_or(false),
            tier_level: params.tier_level.unwrap_or(0),
            max_members: params.max_members,
            auto_assign_enabled: params.auto_assign_enabled.unwrap_or(false),
            metadata: params.metadata.unwrap_or_else(|| serde_json::json!({})),
            is_public: params.is_public.unwrap_or(true),
            grace_period_hours: params.grace_period_hours.unwrap_or(0),
            base: AggregateBase::new(),
        };

        // Emit created event
        plan.base.add_event(Box::new(PlanCreatedEvent::new(
            id.as_str(),
            plan.base.version,
            id.as_str(),
            params.name,
            params.slug.as_str().to_string(),
            permissions.iter().map(|p| p.as_str().to_string()).collect(),
            now,
        )));

        Ok(plan)
    }

    /// Load existing plan from database
    pub fn load(params: LoadPlanParams) -> Self {
        Self {
            id: params.id,
            name: params.name,
            slug: params.slug,
            description: params.description,
            plan_type: params.plan_type,
            plan_category: params.plan_category,
            plan_group: params.plan_group,
            permissions: params.permissions,
            price: params.price,
            currency: params.currency,
            billing_cycle: params.billing_cycle,
            is_active: params.is_active,
            is_promoted: params.is_promoted,
            tier_level: params.tier_level,
            max_members: params.max_members,
            auto_assign_enabled: params.auto_assign_enabled,
            metadata: params.metadata,
            is_public: params.is_public,
            grace_period_hours: params.grace_period_hours,
            base: AggregateBase {
                version: params.version,
                created_at: params.created_at,
                updated_at: params.updated_at,
                events: Vec::new(),
            },
        }
    }

    /// Update plan information
    pub fn update(&mut self, params: UpdatePlanParams) -> AppResult<()> {
        if let Some(n) = params.name {
            self.name = n;
        }
        if let Some(d) = params.description {
            self.description = d;
        }
        if let Some(cat) = params.plan_category {
            self.plan_category = cat;
        }
        if let Some(grp) = params.plan_group {
            self.plan_group = grp;
        }
        if let Some(perms) = params.permissions {
            self.permissions = perms.into_iter().collect();
        }
        if let Some(p) = params.price {
            self.price = p;
        }
        if let Some(c) = params.currency {
            self.currency = c;
        }
        if let Some(bc) = params.billing_cycle {
            self.billing_cycle = bc;
        }
        if let Some(active) = params.is_active {
            self.is_active = active;
        }
        if let Some(promoted) = params.is_promoted {
            self.is_promoted = promoted;
        }
        if let Some(tier) = params.tier_level {
            self.tier_level = tier;
        }
        if let Some(max) = params.max_members {
            self.max_members = max;
        }
        if let Some(auto) = params.auto_assign_enabled {
            self.auto_assign_enabled = auto;
        }
        if let Some(meta) = params.metadata {
            self.metadata = meta;
        }
        if let Some(public) = params.is_public {
            self.is_public = public;
        }
        if let Some(gph) = params.grace_period_hours {
            self.grace_period_hours = gph;
        }

        self.base.touch();
        self.base.increment_version();

        // Emit updated event
        self.base.add_event(Box::new(PlanUpdatedEvent::new(
            self.id.as_str(),
            self.base.version,
            self.id.as_str(),
            self.base.updated_at,
        )));

        Ok(())
    }

    /// Add permission to plan
    pub fn add_permission(&mut self, permission: PermissionString) -> AppResult<()> {
        self.permissions.insert(permission);
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }

    /// Remove permission from plan
    pub fn remove_permission(&mut self, permission: &PermissionString) -> AppResult<()> {
        self.permissions.remove(permission);
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }

    /// Check if plan has specific permission
    pub fn has_permission(&self, permission: &PermissionString) -> bool {
        self.permissions.contains(permission)
    }

    /// Activate plan
    pub fn activate(&mut self) -> AppResult<()> {
        self.is_active = true;
        self.base.touch();
        Ok(())
    }

    /// Deactivate plan
    pub fn deactivate(&mut self) -> AppResult<()> {
        self.is_active = false;
        self.base.touch();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &PlanId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn slug(&self) -> &PlanSlug {
        &self.slug
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn plan_type(&self) -> &str {
        &self.plan_type
    }

    pub fn plan_category(&self) -> &PlanCategory {
        &self.plan_category
    }

    pub fn plan_group(&self) -> &PlanGroup {
        &self.plan_group
    }

    pub fn permissions(&self) -> &HashSet<PermissionString> {
        &self.permissions
    }

    pub fn price(&self) -> f64 {
        self.price
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn billing_cycle(&self) -> &str {
        &self.billing_cycle
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_promoted(&self) -> bool {
        self.is_promoted
    }

    pub fn tier_level(&self) -> i32 {
        self.tier_level
    }

    pub fn max_members(&self) -> Option<i32> {
        self.max_members
    }

    pub fn auto_assign_enabled(&self) -> bool {
        self.auto_assign_enabled
    }

    pub fn metadata(&self) -> &serde_json::Value {
        &self.metadata
    }

    pub fn is_public(&self) -> bool {
        self.is_public
    }

    pub fn grace_period_hours(&self) -> i32 {
        self.grace_period_hours
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

// Type aliases for backward compatibility during migration
pub type PermissionPlan = Plan;
pub type CreatePermissionPlanParams = CreatePlanParams;
pub type LoadPermissionPlanParams = LoadPlanParams;
pub type UpdatePermissionPlanParams = UpdatePlanParams;
