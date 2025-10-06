use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::queries::{
    GetSubscriptionQuery, GetSubscriptionResponse
};
use crate::domain::subscription_management::{SubscriptionId, SubscriptionRepositoryPort};

/// Query handler for getting a single subscription
pub struct GetSubscriptionQueryHandler {
    subscription_repository: Arc<dyn SubscriptionRepositoryPort>,
}

impl GetSubscriptionQueryHandler {
    pub fn new(subscription_repository: Arc<dyn SubscriptionRepositoryPort>) -> Self {
        Self {
            subscription_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetSubscriptionQuery> for GetSubscriptionQueryHandler {
    async fn handle(&self, query: GetSubscriptionQuery) -> ApplicationResult<GetSubscriptionResponse> {
        // 1. Parse subscription ID
        let subscription_id = SubscriptionId::from_str(&query.subscription_id)
            .map_err(|e| ApplicationError::validation("subscription_id", e.to_string()))?;

        // 2. Load subscription from repository
        let subscription = self.subscription_repository.find_by_id(&subscription_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("subscription_id", "Subscription not found"))?;

        // 3. Map domain aggregate to response DTO
        Ok(GetSubscriptionResponse {
            id: subscription.id().as_str().to_string(),
            wallet_address: subscription.wallet_address().as_str().to_string(),
            plan_id: subscription.plan_id().as_i32(),
            status: format!("{:?}", subscription.status()),
            started_at: subscription.started_at(),
            expires_at: subscription.expires_at(),
            auto_renew: subscription.auto_renew(),
            created_at: subscription.created_at(),
        })
    }
}
