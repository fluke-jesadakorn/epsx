use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::queries::{
    GetPermissionPlanQuery, GetPermissionPlanResponse
};
use crate::domain::permission_management::{PermissionPlanRepositoryPort, PlanAssignmentRepositoryPort, PlanId};

/// Query handler for getting a single permission plan
pub struct GetPermissionPlanQueryHandler {
    plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
    assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
}

impl GetPermissionPlanQueryHandler {
    pub fn new(
        plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
        assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
    ) -> Self {
        Self {
            plan_repository,
            assignment_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetPermissionPlanQuery> for GetPermissionPlanQueryHandler {
    async fn handle(&self, query: GetPermissionPlanQuery) -> ApplicationResult<GetPermissionPlanResponse> {
        // 1. Parse plan ID
        let plan_id = PlanId::from_str(&query.plan_id)
            .map_err(|e| ApplicationError::validation("plan_id", e.to_string()))?;

        // 2. Find plan
        let plan = self.plan_repository.find_by_id(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("PermissionPlan", query.plan_id))?;

        // 3. Get member count
        let member_count = self.assignment_repository.count_plan_members(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Return response
        Ok(GetPermissionPlanResponse {
            id: plan.id().as_str(),
            name: plan.name().to_string(),
            slug: plan.slug().as_str().to_string(),
            description: plan.description().to_string(),
            plan_type: plan.plan_type().to_string(),
            permissions: plan.permissions().iter().map(|p| p.as_str().to_string()).collect(),
            price: plan.price(),
            currency: plan.currency().to_string(),
            billing_cycle: plan.billing_cycle().to_string(),
            is_active: plan.is_active(),
            is_promoted: plan.is_promoted(),
            display_order: plan.display_order(),
            max_members: plan.max_members(),
            auto_assign_enabled: plan.auto_assign_enabled(),
            metadata: plan.metadata().clone(),
            created_at: plan.created_at(),
            updated_at: plan.updated_at(),
            member_count,
        })
    }
}
