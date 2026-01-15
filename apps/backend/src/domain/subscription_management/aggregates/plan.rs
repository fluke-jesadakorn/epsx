use crate::prelude::*;
use crate::domain::shared_kernel::{AggregateRoot, AggregateBase, DomainEvent};
use crate::domain::subscription_management::{PlanId, Price, BillingCycle, PlanFeatures};
use crate::domain::subscription_management::value_objects::quota::Quota;
use crate::domain::permission_management::GroupId;

/// Plan Aggregate Root
/// Represents a subscription plan with pricing, features, and permission group association
#[derive(Debug, Clone)]
pub struct Plan {
    id: PlanId,
    name: String,
    description: String,
    group_id: GroupId,
    price: Price,
    billing_cycle: BillingCycle,
    features: PlanFeatures,
    target_audience: String,
    // New fields for permission-based logic
    pub permissions: Vec<String>, 
    pub quotas: std::collections::HashMap<String, Quota>,
    
    is_active: bool,
    is_promoted: bool,
    display_order: i32,
    metadata: serde_json::Value,
    base: AggregateBase,
}


pub struct CreatePlanParams {
    pub name: String,
    pub description: String,
    pub group_id: GroupId,
    pub permissions: Vec<String>, // Added
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
    pub group_id: GroupId,
    pub permissions: Vec<String>, // Added
    pub quotas: std::collections::HashMap<String, Quota>, // Added
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
    pub permissions: Option<Vec<String>>, // Added
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

impl Plan {
    /// Create a new plan
    pub fn create(mut params: CreatePlanParams) -> AppResult<Self> {
        // Sync permissions from metadata if present
        if let Some(meta) = &params.metadata {
            Self::sync_permissions_vec(&mut params.permissions, meta);
        }

        let mut plan = Self {
            id: PlanId::new(),
            name: params.name,
            description: params.description,
            group_id: params.group_id,
            price: params.price,
            billing_cycle: params.billing_cycle,
            features: params.features,
            target_audience: params.target_audience,
            permissions: params.permissions,
            quotas: std::collections::HashMap::new(),
            is_active: params.is_active.unwrap_or(true),
            is_promoted: params.is_promoted.unwrap_or(false),
            display_order: params.display_order.unwrap_or(0),
            metadata: params.metadata.unwrap_or(serde_json::json!({})),
            base: AggregateBase::new(),
        };

        plan.calculate_quotas();
        Ok(plan)
    }

    /// Reconstruct plan from persistence
    pub fn reconstruct(params: LoadPlanParams) -> Self {
        Self {
            id: params.id,
            name: params.name,
            description: params.description,
            group_id: params.group_id,
            price: params.price,
            billing_cycle: params.billing_cycle,
            features: params.features,
            target_audience: params.target_audience,
            permissions: params.permissions,
            quotas: params.quotas,
            is_active: params.is_active,
            is_promoted: params.is_promoted,
            display_order: params.display_order,
            metadata: params.metadata,
            base: AggregateBase::from_persistence(params.version, params.created_at, params.updated_at),
        }
    }

    /// Helper to sync permissions vector with metadata
    fn sync_permissions_vec(permissions: &mut Vec<String>, metadata: &serde_json::Value) {
        // Remove existing dynamic permissions to avoid duplicates
        permissions.retain(|p| 
            !p.starts_with("epsx:rankings:offset:") && 
            !p.starts_with("epsx:rankings:limit:") &&
            !p.starts_with("epsx:analytics:view:")
        );

        if let Some(features) = metadata.get("features").and_then(|f| f.as_object()) {
            // Handle ranking offset
            if let Some(offset) = features.get("ranking_offset").and_then(|v| v.as_i64()) {
                permissions.push(format!("epsx:rankings:offset:{}", offset));
            }
            
            // Handle ranking limit
            if let Some(limit) = features.get("rankings_limit").and_then(|v| v.as_i64()) {
                permissions.push(format!("epsx:rankings:limit:{}", limit));
                // Add legacy compatibility
                permissions.push(format!("epsx:analytics:view:{}", limit));
            }
        }
        // Also check top-level if not in features object (fallback)
        else {
             if let Some(offset) = metadata.get("ranking_offset").and_then(|v| v.as_i64()) {
                permissions.push(format!("epsx:rankings:offset:{}", offset));
            }
            
            if let Some(limit) = metadata.get("rankings_limit").and_then(|v| v.as_i64()) {
                permissions.push(format!("epsx:rankings:limit:{}", limit));
                permissions.push(format!("epsx:analytics:view:{}", limit));
            }
        }
    }

    /// Calculate quotas based on permissions
    pub fn calculate_quotas(&mut self) {
        let mut quotas = std::collections::HashMap::new();
        
        for permission in &self.permissions {
            if permission.contains("limit:") {
                // Example logic to parse limit from permission string
                // e.g. "epsx:analytics:view:100" -> quota of 100
                if let Some(limit_str) = permission.split(':').next_back() {
                    if let Ok(limit) = limit_str.parse::<i64>() {
                        quotas.insert(permission.clone(), Quota::new(limit));
                    }
                }
            }
        }
        
        self.quotas = quotas;
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
        
        // Handle permissions update
        let permissions_updated = params.permissions.is_some();
        if let Some(perms) = params.permissions {
            self.permissions = perms;
        }
        
        // Handle metadata update and sync permissions
        if let Some(meta) = params.metadata {
            self.metadata = meta;
            // Always resync when metadata changes (or if new permissions were just set)
            Self::sync_permissions_vec(&mut self.permissions, &self.metadata);
        } else if permissions_updated {
             // If only permissions changed but metadata didn't, we should still ensure metadata-derived permissions are present
             // (unless we want to allow manual override?) 
             // Logic decision: Metadata constraints should usually override or be additive.
             // Let's re-run sync to ensure metadata constraints are enforced.
             Self::sync_permissions_vec(&mut self.permissions, &self.metadata);
        }

        // Recalculate quotas after all permission changes
        self.calculate_quotas();
        
        if let Some(active) = params.is_active {
            self.is_active = active;
        }
        if let Some(promoted) = params.is_promoted {
            self.is_promoted = promoted;
        }
        if let Some(order) = params.display_order {
            self.display_order = order;
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

    pub fn group_id(&self) -> &GroupId {
        &self.group_id
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
