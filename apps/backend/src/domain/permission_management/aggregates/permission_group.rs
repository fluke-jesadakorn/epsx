use crate::prelude::*;
use crate::domain::shared_kernel::{AggregateRoot, AggregateBase, DomainEvent};
use crate::domain::permission_management::{
    GroupId, GroupSlug, PermissionString,
    events::{PermissionGroupCreatedEvent, PermissionGroupUpdatedEvent}
};
use std::collections::HashSet;

/// Permission Group Aggregate Root
/// Represents a group of permissions that can be assigned to wallets
#[derive(Debug, Clone)]
pub struct PermissionGroup {
    id: GroupId,
    name: String,
    slug: GroupSlug,
    description: String,
    group_type: String,
    permissions: HashSet<PermissionString>,
    price: f64,
    currency: String,
    billing_cycle: String,
    is_active: bool,
    is_promoted: bool,
    display_order: i32,
    max_members: Option<i32>,
    auto_assign_enabled: bool,
    metadata: serde_json::Value,
    base: AggregateBase,
}

pub struct CreatePermissionGroupParams {
    pub name: String,
    pub slug: GroupSlug,
    pub description: String,
    pub group_type: String,
    pub permissions: Vec<PermissionString>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

pub struct LoadPermissionGroupParams {
    pub id: GroupId,
    pub name: String,
    pub slug: GroupSlug,
    pub description: String,
    pub group_type: String,
    pub permissions: HashSet<PermissionString>,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: i32,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
}

impl PermissionGroup {
    /// Create a new permission group
    pub fn create(params: CreatePermissionGroupParams) -> AppResult<Self> {
        let now = Utc::now();
        let id = GroupId::new();
        let permissions: HashSet<PermissionString> = params.permissions.into_iter().collect();

        let mut group = Self {
            id: id.clone(),
            name: params.name.clone(),
            slug: params.slug.clone(),
            description: params.description.clone(),
            group_type: params.group_type.clone(),
            permissions: permissions.clone(),
            price: params.price.unwrap_or(0.0),
            currency: params.currency.unwrap_or_else(|| "USD".to_string()),
            billing_cycle: params.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
            is_active: params.is_active.unwrap_or(true),
            is_promoted: params.is_promoted.unwrap_or(false),
            display_order: params.display_order.unwrap_or(0),
            max_members: params.max_members,
            auto_assign_enabled: params.auto_assign_enabled.unwrap_or(false),
            metadata: params.metadata.unwrap_or_else(|| serde_json::json!({})),
            base: AggregateBase::new(),
        };

        // Emit created event
        group.base.add_event(Box::new(PermissionGroupCreatedEvent::new(
            id.as_str(),
            group.base.version,
            id.as_str(),
            params.name,
            params.slug.as_str().to_string(),
            permissions.iter().map(|p| p.as_str().to_string()).collect(),
            now,
        )));

        Ok(group)
    }

    /// Load existing permission group from database
    pub fn load(params: LoadPermissionGroupParams) -> Self {
        Self {
            id: params.id,
            name: params.name,
            slug: params.slug,
            description: params.description,
            group_type: params.group_type,
            permissions: params.permissions,
            price: params.price,
            currency: params.currency,
            billing_cycle: params.billing_cycle,
            is_active: params.is_active,
            is_promoted: params.is_promoted,
            display_order: params.display_order,
            max_members: params.max_members,
            auto_assign_enabled: params.auto_assign_enabled,
            metadata: params.metadata,
            base: AggregateBase {
                version: params.version,
                created_at: params.created_at,
                updated_at: params.updated_at,
                events: Vec::new(),
            },
        }
    }

    /// Update group information
    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        permissions: Option<Vec<PermissionString>>,
        price: Option<f64>,
        currency: Option<String>,
        billing_cycle: Option<String>,
        is_active: Option<bool>,
        is_promoted: Option<bool>,
        display_order: Option<i32>,
        max_members: Option<Option<i32>>,
        auto_assign_enabled: Option<bool>,
        metadata: Option<serde_json::Value>,
    ) -> AppResult<()> {
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(d) = description {
            self.description = d;
        }
        if let Some(perms) = permissions {
            self.permissions = perms.into_iter().collect();
        }
        if let Some(p) = price {
            self.price = p;
        }
        if let Some(c) = currency {
            self.currency = c;
        }
        if let Some(bc) = billing_cycle {
            self.billing_cycle = bc;
        }
        if let Some(active) = is_active {
            self.is_active = active;
        }
        if let Some(promoted) = is_promoted {
            self.is_promoted = promoted;
        }
        if let Some(order) = display_order {
            self.display_order = order;
        }
        if let Some(max) = max_members {
            self.max_members = max;
        }
        if let Some(auto) = auto_assign_enabled {
            self.auto_assign_enabled = auto;
        }
        if let Some(meta) = metadata {
            self.metadata = meta;
        }

        self.base.touch();
        self.base.increment_version();

        // Emit updated event
        self.base.add_event(Box::new(PermissionGroupUpdatedEvent::new(
            self.id.as_str(),
            self.base.version,
            self.id.as_str(),
            self.base.updated_at,
        )));

        Ok(())
    }

    /// Add permission to group
    pub fn add_permission(&mut self, permission: PermissionString) -> AppResult<()> {
        self.permissions.insert(permission);
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }

    /// Remove permission from group
    pub fn remove_permission(&mut self, permission: &PermissionString) -> AppResult<()> {
        self.permissions.remove(permission);
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }

    /// Check if group has specific permission
    pub fn has_permission(&self, permission: &PermissionString) -> bool {
        self.permissions.contains(permission)
    }

    /// Activate group
    pub fn activate(&mut self) -> AppResult<()> {
        self.is_active = true;
        self.base.touch();
        Ok(())
    }

    /// Deactivate group
    pub fn deactivate(&mut self) -> AppResult<()> {
        self.is_active = false;
        self.base.touch();
        Ok(())
    }

    // Getters
    pub fn id(&self) -> &GroupId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn slug(&self) -> &GroupSlug {
        &self.slug
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn group_type(&self) -> &str {
        &self.group_type
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

    pub fn display_order(&self) -> i32 {
        self.display_order
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
}

impl AggregateRoot for PermissionGroup {
    type Id = GroupId;

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
