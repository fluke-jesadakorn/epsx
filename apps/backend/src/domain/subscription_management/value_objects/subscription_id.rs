use crate::prelude::*;
use uuid::Uuid;
use std::str::FromStr;

/// Subscription ID value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(Uuid);

impl SubscriptionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> AppResult<Self> {
        s.parse()
    }

    pub fn value(&self) -> &Uuid {
        &self.0
    }

    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for SubscriptionId {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)
            .map_err(|e| AppError::validation_error(format!("Invalid subscription ID: {}", e)))?;
        Ok(Self(uuid))
    }
}

impl Default for SubscriptionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
