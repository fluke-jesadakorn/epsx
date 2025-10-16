use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::resource_management::queries::{
    GetResourceUsageQuery, GetResourceUsageResponse
};
use crate::domain::resource_management::repository_ports::UserResourceUsageRepository;

/// Handler for getting resource usage
pub struct GetResourceUsageQueryHandler<R: UserResourceUsageRepository> {
    usage_repository: Arc<R>,
}

impl<R: UserResourceUsageRepository> GetResourceUsageQueryHandler<R> {
    pub fn new(usage_repository: Arc<R>) -> Self {
        Self { usage_repository }
    }
}

#[async_trait]
impl<R: UserResourceUsageRepository + Send + Sync> QueryHandler<GetResourceUsageQuery>
    for GetResourceUsageQueryHandler<R>
where
    R::Error: std::fmt::Display,
{
    async fn handle(&self, query: GetResourceUsageQuery) -> ApplicationResult<GetResourceUsageResponse> {
        // 1. Retrieve usage data
        let usage = self.usage_repository
            .get_user_usage(&query.wallet_address, &query.access_context)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("usage", query.wallet_address.clone()))?;

        // 2. Calculate usage percentages
        let mut usage_percentages = std::collections::HashMap::new();
        for resource_type in usage.current_usage().keys() {
            let percentage = usage.get_usage_percentage(resource_type);
            usage_percentages.insert(resource_type.clone(), percentage);
        }

        // 3. Return response
        Ok(GetResourceUsageResponse {
            wallet_address: usage.wallet_address().to_string(),
            plan_id: usage.plan_id(),
            current_usage: usage.current_usage().clone(),
            quota_limits: usage.quota_limits().clone(),
            usage_percentages,
            billing_period_start: usage.billing_period_start(),
            billing_period_end: usage.billing_period_end(),
        })
    }
}
