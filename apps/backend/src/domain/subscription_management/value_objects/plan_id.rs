use crate::prelude::*;

/// Plan ID value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlanId(i32);

impl PlanId {
    /// Create a new plan ID (placeholder for new plans before database insertion)
    pub fn new() -> Self {
        Self(0) // Placeholder - will be assigned by database auto-increment
    }

    /// Create from existing ID (for loading from database)
    pub fn from_i32(id: i32) -> Self {
        Self(id)
    }

    /// Get the inner i32 value
    pub fn as_i32(&self) -> i32 {
        self.0
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

impl std::fmt::Display for PlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for PlanId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}
