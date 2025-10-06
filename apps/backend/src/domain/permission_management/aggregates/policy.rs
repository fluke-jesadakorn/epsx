use crate::prelude::*;
use crate::domain::shared_kernel::{AggregateRoot, AggregateBase, DomainEvent};
use crate::domain::permission_management::{PolicyId, PolicyRule, events::PolicyCreatedEvent};

/// Policy Aggregate Root
/// Represents access control policies and rules
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Policy {
    id: PolicyId,
    name: String,
    description: String,
    rules: Vec<PolicyRule>,
    is_active: bool,
    priority: i32,
    base: AggregateBase,
}

impl Policy {
    /// Create a new policy
    pub fn create(
        name: impl Into<String>,
        description: impl Into<String>,
        rules: Vec<PolicyRule>,
        priority: Option<i32>,
    ) -> AppResult<Self> {
        let now = Utc::now();
        let id = PolicyId::new();
        let name = name.into();

        let mut policy = Self {
            id: id.clone(),
            name: name.clone(),
            description: description.into(),
            rules,
            is_active: true,
            priority: priority.unwrap_or(0),
            base: AggregateBase::new(),
        };

        policy.base.add_event(Box::new(PolicyCreatedEvent::new(
            id.as_str(),
            policy.base.version,
            id.as_str(),
            name,
            now,
        )));

        Ok(policy)
    }

    /// Add rule to policy
    pub fn add_rule(&mut self, rule: PolicyRule) -> AppResult<()> {
        self.rules.push(rule);
        self.base.touch();
        self.base.increment_version();
        Ok(())
    }

    /// Activate policy
    pub fn activate(&mut self) {
        self.is_active = true;
        self.base.touch();
    }

    /// Deactivate policy
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.base.touch();
    }

    // Getters
    pub fn id(&self) -> &PolicyId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rules(&self) -> &[PolicyRule] {
        &self.rules
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

impl AggregateRoot for Policy {
    type Id = PolicyId;

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
