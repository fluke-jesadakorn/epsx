use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::queries::{
    GetWalletPlansQuery, GetWalletPlansResponse, WalletPlanInfo
};
use crate::domain::permission_management::{PlanAssignmentRepositoryPort, PermissionPlanRepositoryPort};
use crate::domain::wallet_management::WalletAddress;

/// Query handler for getting wallet plans
pub struct GetWalletPlansQueryHandler {
    assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
    plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
}

impl GetWalletPlansQueryHandler {
    pub fn new(
        assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
        plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
    ) -> Self {
        Self {
            assignment_repository,
            plan_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetWalletPlansQuery> for GetWalletPlansQueryHandler {
    async fn handle(&self, query: GetWalletPlansQuery) -> ApplicationResult<GetWalletPlansResponse> {
        // 1. Parse wallet address
        let wallet_address = WalletAddress::new(&query.wallet_address)
            .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?;

        // 2. Find assignments
        let assignments = self.assignment_repository.find_by_wallet(&wallet_address).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Get plan details for each assignment with actual timestamps
        let mut plans = Vec::new();
        for assignment in assignments {
            if let Ok(Some(plan)) = self.plan_repository.find_by_id(assignment.plan_id()).await {
                plans.push(WalletPlanInfo {
                    plan_id: plan.id().as_str(),
                    plan_name: plan.name().to_string(),
                    plan_slug: plan.slug().as_str().to_string(),
                    permissions: plan.permissions().iter().map(|p| p.as_str().to_string()).collect(),
                    assigned_at: assignment.assigned_at(),
                    expires_at: assignment.expires_at(),
                    is_active: assignment.is_active(),
                });
            }
        }

        Ok(GetWalletPlansResponse {
            wallet_address: query.wallet_address,
            total: plans.len() as i64,
            plans,
        })
    }
}
