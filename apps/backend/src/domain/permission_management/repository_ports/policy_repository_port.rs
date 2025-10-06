use crate::prelude::*;
use crate::domain::permission_management::{Policy, PolicyId};

/// Repository port for policy operations
#[async_trait]
pub trait PolicyRepositoryPort: Send + Sync {
    /// Find policy by ID
    async fn find_by_id(&self, id: &PolicyId) -> AppResult<Option<Policy>>;

    /// Find all active policies
    async fn find_active(&self) -> AppResult<Vec<Policy>>;

    /// Save (create or update) a policy
    async fn save(&self, policy: &Policy) -> AppResult<()>;

    /// Delete a policy
    async fn delete(&self, id: &PolicyId) -> AppResult<()>;
}
