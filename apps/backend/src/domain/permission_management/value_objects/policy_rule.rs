use crate::prelude::*;

/// Policy rule value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRule {
    condition: String,
    action: PolicyAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    RequireMFA,
    AuditLog,
}

impl PolicyRule {
    pub fn new(condition: impl Into<String>, action: PolicyAction) -> AppResult<Self> {
        let condition = condition.into();

        if condition.is_empty() {
            return Err(AppError::validation_error("Policy condition cannot be empty"));
        }

        Ok(Self { condition, action })
    }

    pub fn condition(&self) -> &str {
        &self.condition
    }

    pub fn action(&self) -> &PolicyAction {
        &self.action
    }
}
