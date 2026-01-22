use crate::prelude::*;
use crate::domain::permission_management::{PlanId, entities::PlanAssignment};
use crate::domain::wallet_management::WalletAddress;

/// Repository port for plan assignment operations
#[async_trait]
pub trait PlanAssignmentRepositoryPort: Send + Sync {
    /// Find assignments for a wallet
    async fn find_by_wallet(&self, wallet: &WalletAddress) -> AppResult<Vec<PlanAssignment>>;

    /// Find assignments for a plan
    async fn find_by_plan(&self, plan_id: &PlanId) -> AppResult<Vec<PlanAssignment>>;

    /// Find specific assignment
    async fn find_assignment(
        &self,
        wallet: &WalletAddress,
        plan_id: &PlanId,
    ) -> AppResult<Option<PlanAssignment>>;

    /// Save (create or update) an assignment
    async fn save(&self, assignment: &PlanAssignment) -> AppResult<()>;

    /// Delete an assignment
    async fn delete(&self, wallet: &WalletAddress, plan_id: &PlanId) -> AppResult<()>;

    /// Count members in a plan
    async fn count_plan_members(&self, plan_id: &PlanId) -> AppResult<i64>;
}
