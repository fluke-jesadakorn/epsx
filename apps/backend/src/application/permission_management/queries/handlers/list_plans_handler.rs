use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::queries::{
    ListPermissionPlansQuery, ListPermissionPlansResponse, PermissionPlanSummary
};
use crate::domain::permission_management::{PermissionPlanRepositoryPort, PlanAssignmentRepositoryPort, repository_ports::PlanSearchCriteria};

/// Query handler for listing permission plans
pub struct ListPermissionPlansQueryHandler {
    plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
    assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
}

impl ListPermissionPlansQueryHandler {
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
impl QueryHandler<ListPermissionPlansQuery> for ListPermissionPlansQueryHandler {
    async fn handle(&self, query: ListPermissionPlansQuery) -> ApplicationResult<ListPermissionPlansResponse> {
        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(20);
        let offset = ((page - 1) * limit) as i64;

        // 1. Build search criteria
        let criteria = PlanSearchCriteria {
            plan_type: query.plan_type,
            is_active: query.is_active,
            is_promoted: query.is_promoted,
            search_term: query.search_term,
            limit: Some(limit as i64),
            offset: Some(offset),
        };

        // 2. Find plans
        let plans = self.plan_repository.find_all(criteria.clone()).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Get total count
        let total = self.plan_repository.count(criteria).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Build summaries with member counts
        let mut summaries = Vec::new();
        for plan in plans {
            let member_count = self.assignment_repository.count_plan_members(plan.id()).await
                .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

            summaries.push(PermissionPlanSummary {
                id: plan.id().as_str(),
                name: plan.name().to_string(),
                slug: plan.slug().as_str().to_string(),
                description: plan.description().to_string(),
                plan_type: plan.plan_type().to_string(),
                permissions: plan.permissions().iter().map(|p| p.as_str().to_string()).collect(),
                price: plan.price(),
                currency: plan.currency().to_string(),
                is_active: plan.is_active(),
                is_promoted: plan.is_promoted(),
                member_count,
            });
        }

        Ok(ListPermissionPlansResponse {
            plans: summaries,
            total,
            page,
            limit,
        })
    }
}
