use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::queries::{
    ListPlansQuery, ListPlansResponse, PlanSummary
};
use crate::domain::subscription_management::{PlanRepositoryPort, PlanSearchCriteria};

/// Query handler for listing plans
pub struct ListPlansQueryHandler {
    plan_repository: Arc<dyn PlanRepositoryPort>,
}

impl ListPlansQueryHandler {
    pub fn new(plan_repository: Arc<dyn PlanRepositoryPort>) -> Self {
        Self {
            plan_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<ListPlansQuery> for ListPlansQueryHandler {
    async fn handle(&self, query: ListPlansQuery) -> ApplicationResult<ListPlansResponse> {
        // 1. Calculate pagination
        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(20);
        let offset = ((page - 1) * limit) as i64;

        // 2. Build search criteria
        let criteria = PlanSearchCriteria {
            is_active: query.is_active,
            is_promoted: query.is_promoted,
            min_price: None,
            max_price: None,
            search_term: None,
            limit: Some(limit as i64),
            offset: Some(offset),
        };

        // 3. Fetch plans and total count in parallel
        let plans = self.plan_repository.find_all(criteria.clone()).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        let total = self.plan_repository.count(criteria).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Map to summary DTOs
        let plan_summaries: Vec<PlanSummary> = plans.iter().map(|plan| {
            PlanSummary {
                id: plan.id().as_i32(),
                name: plan.name().to_string(),
                description: plan.description().to_string(),
                price: plan.price().to_f64(),
                currency: plan.price().currency().to_string(),
                billing_cycle: plan.billing_cycle().as_str().to_string(),
                is_active: plan.is_active(),
                is_promoted: plan.is_promoted(),
            }
        }).collect();

        // 5. Return response
        Ok(ListPlansResponse {
            plans: plan_summaries,
            total,
            page,
            limit,
        })
    }
}
