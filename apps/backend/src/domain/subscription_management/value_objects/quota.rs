use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quota {
    /// -1 means unlimited, 0 means no access, >0 means specific limit
    pub limit: i64,
}

impl Quota {
    pub fn unlimited() -> Self {
        Self { limit: -1 }
    }

    pub fn no_access() -> Self {
        Self { limit: 0 }
    }

    pub fn new(limit: i64) -> Self {
        Self { limit }
    }

    pub fn is_unlimited(&self) -> bool {
        self.limit == -1
    }

    pub fn has_access(&self) -> bool {
        self.limit != 0
    }

    /// Check if usage is within quota
    pub fn is_within_limit(&self, usage: i64) -> bool {
        if self.is_unlimited() {
            true
        } else {
            usage < self.limit
        }
    }
}

impl Default for Quota {
    fn default() -> Self {
        Self::no_access()
    }
}
