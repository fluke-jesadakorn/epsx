use crate::prelude::*;
use crate::domain::permission_management::{GroupId, entities::GroupAssignment};
use crate::domain::wallet_management::WalletAddress;

/// Repository port for group assignment operations
#[async_trait]
pub trait GroupAssignmentRepositoryPort: Send + Sync {
    /// Find assignments for a wallet
    async fn find_by_wallet(&self, wallet: &WalletAddress) -> AppResult<Vec<GroupAssignment>>;

    /// Find assignments for a group
    async fn find_by_group(&self, group_id: &GroupId) -> AppResult<Vec<GroupAssignment>>;

    /// Find specific assignment
    async fn find_assignment(
        &self,
        wallet: &WalletAddress,
        group_id: &GroupId,
    ) -> AppResult<Option<GroupAssignment>>;

    /// Save (create or update) an assignment
    async fn save(&self, assignment: &GroupAssignment) -> AppResult<()>;

    /// Delete an assignment
    async fn delete(&self, wallet: &WalletAddress, group_id: &GroupId) -> AppResult<()>;

    /// Count members in a group
    async fn count_group_members(&self, group_id: &GroupId) -> AppResult<i64>;
}
