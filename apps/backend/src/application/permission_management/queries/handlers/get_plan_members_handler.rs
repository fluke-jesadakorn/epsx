use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::queries::{
    GetPlanMembersQuery, GetPlanMembersResponse, PlanMemberInfo
};
use crate::domain::permission_management::{PlanAssignmentRepositoryPort, PlanId};

/// Query handler for getting plan members
pub struct GetPlanMembersQueryHandler {
    assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
}

impl GetPlanMembersQueryHandler {
    pub fn new(assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>) -> Self {
        Self {
            assignment_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetPlanMembersQuery> for GetPlanMembersQueryHandler {
    async fn handle(&self, query: GetPlanMembersQuery) -> ApplicationResult<GetPlanMembersResponse> {
        // 1. Parse plan ID
        let plan_id = PlanId::parse(&query.plan_id)
            .map_err(|e| ApplicationError::validation("plan_id", e.to_string()))?;

        // 2. Find assignments
        let assignments = self.assignment_repository.find_by_plan(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Build response with actual timestamps
        let members: Vec<PlanMemberInfo> = assignments
            .iter()
            .map(|a| PlanMemberInfo {
                wallet_address: a.wallet_address().as_str().to_string(),
                assigned_at: a.assigned_at(),
                expires_at: a.expires_at(),
                is_active: a.is_active(),
            })
            .collect();

        Ok(GetPlanMembersResponse {
            plan_id: query.plan_id,
            total: members.len() as i64,
            members,
        })
    }
}
