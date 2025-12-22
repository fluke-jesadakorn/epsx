use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::queries::{GetPlanQuery, GetPlanResponse};
use crate::domain::subscription_management::{PlanId, PlanRepositoryPort};

/// Query handler for getting a single plan
pub struct GetPlanQueryHandler {
    plan_repository: Arc<dyn PlanRepositoryPort>,
}

impl GetPlanQueryHandler {
    pub fn new(plan_repository: Arc<dyn PlanRepositoryPort>) -> Self {
        Self {
            plan_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetPlanQuery> for GetPlanQueryHandler {
    async fn handle(&self, query: GetPlanQuery) -> ApplicationResult<GetPlanResponse> {
        // 1. Parse plan ID
        let plan_id = PlanId::from_i32(query.plan_id);

        // 2. Load plan from repository
        let plan = self.plan_repository.find_by_id(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("plan_id", "Plan not found"))?;

        // 3. Map domain aggregate to response DTO
        Ok(GetPlanResponse {
            id: plan_id.as_i32(),
            name: plan.name().to_string(),
            description: plan.description().to_string(),
            group_id: plan.group_id().as_str().to_string(),
            price: plan.price().to_f64(),
            currency: plan.price().currency().to_string(),
            billing_cycle: plan.billing_cycle().as_str().to_string(),
            target_audience: plan.target_audience().to_string(),
            is_active: plan.is_active(),
            is_promoted: plan.is_promoted(),
            display_order: plan.display_order(),
            features: serde_json::json!({
                "api_calls_limit": plan.features().api_calls_limit(),
                "rankings_limit": plan.features().rankings_limit(),
                "analytics_enabled": plan.features().analytics_enabled(),
                "premium_support": plan.features().premium_support(),
                "custom_features": plan.features().custom_features(),
            }),
            metadata: plan.metadata().clone(),
            created_at: plan.created_at(),
            updated_at: plan.updated_at(),
        })
    }
}
