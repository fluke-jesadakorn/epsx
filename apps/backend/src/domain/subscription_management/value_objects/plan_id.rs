use crate::prelude::*;
use uuid::Uuid;

/// Plan ID value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlanId(Uuid);

impl Default for PlanId {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanId {
    /// Create a new plan ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    /// Get the inner Uuid value
    pub fn value(&self) -> &Uuid {
        &self.0
    }

    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// Parse from string
    pub fn parse(s: &str) -> Result<Self, AppError> {
        use std::str::FromStr;
        <Self as FromStr>::from_str(s)
    }
}

impl std::fmt::Display for PlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PlanId {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)
            .map_err(|e| AppError::validation_error(format!("Invalid plan ID format: {}", e)))?;
        Ok(Self(uuid))
    }
}

impl From<Uuid> for PlanId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

