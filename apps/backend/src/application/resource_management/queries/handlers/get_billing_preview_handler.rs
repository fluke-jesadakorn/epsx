use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::resource_management::queries::{
    GetBillingPreviewQuery, GetBillingPreviewResponse
};
use crate::domain::resource_management::{
    repository_ports::{UserResourceUsageRepository, PlanResourceConfigRepository},
    services::BillingCalculationService,
};

/// Handler for getting billing preview
pub struct GetBillingPreviewQueryHandler<U: UserResourceUsageRepository, P: PlanResourceConfigRepository> {
    usage_repository: Arc<U>,
    plan_repository: Arc<P>,
    billing_service: BillingCalculationService,
}

impl<U: UserResourceUsageRepository, P: PlanResourceConfigRepository> GetBillingPreviewQueryHandler<U, P> {
    pub fn new(usage_repository: Arc<U>, plan_repository: Arc<P>) -> Self {
        Self {
            usage_repository,
            plan_repository,
            billing_service: BillingCalculationService::new(),
        }
    }
}

#[async_trait]
impl<U, P> QueryHandler<GetBillingPreviewQuery> for GetBillingPreviewQueryHandler<U, P>
where
    U: UserResourceUsageRepository + Send + Sync,
    P: PlanResourceConfigRepository + Send + Sync,
    U::Error: std::fmt::Display,
    P::Error: std::fmt::Display,
{
    async fn handle(&self, query: GetBillingPreviewQuery) -> ApplicationResult<GetBillingPreviewResponse> {
        // 1. Retrieve usage data
        let usage = self.usage_repository
            .get_user_usage(&query.wallet_address, "default")
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("usage", query.wallet_address.clone()))?;

        // 2. Get plan configuration if plan exists
        let (base_cost, overage_pricing) = if let Some(plan_id) = usage.plan_id() {
            let config = self.plan_repository
                .get_plan_config(plan_id)
                .await
                .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
                .ok_or_else(|| ApplicationError::not_found("plan_config", plan_id.to_string()))?;

            (0.0, config.overage_pricing().clone()) // Base cost would come from subscription
        } else {
            (0.0, std::collections::HashMap::new())
        };

        // 3. Calculate billing
        let billing_summary = self.billing_service.calculate_billing(
            &usage,
            base_cost,
            &overage_pricing,
        );

        // 4. Return response
        Ok(GetBillingPreviewResponse {
            wallet_address: billing_summary.wallet_address,
            plan_id: billing_summary.plan_id,
            billing_period_start: billing_summary.billing_period_start,
            billing_period_end: billing_summary.billing_period_end,
            base_cost: billing_summary.base_cost,
            overage_costs: billing_summary.overage_costs,
            total_cost: billing_summary.total_cost,
            currency: billing_summary.currency,
        })
    }
}
