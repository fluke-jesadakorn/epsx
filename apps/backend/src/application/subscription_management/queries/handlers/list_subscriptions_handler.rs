use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::queries::{
    ListSubscriptionsQuery, ListSubscriptionsResponse, SubscriptionSummary
};
use crate::domain::subscription_management::{
    SubscriptionRepositoryPort, PlanRepositoryPort, SubscriptionSearchCriteria, PlanId
};
use crate::domain::wallet_management::WalletAddress;
use std::str::FromStr;

/// Query handler for listing subscriptions
pub struct ListSubscriptionsQueryHandler {
    subscription_repository: Arc<dyn SubscriptionRepositoryPort>,
    plan_repository: Arc<dyn PlanRepositoryPort>,
}

impl ListSubscriptionsQueryHandler {
    pub fn new(
        subscription_repository: Arc<dyn SubscriptionRepositoryPort>,
        plan_repository: Arc<dyn PlanRepositoryPort>,
    ) -> Self {
        Self {
            subscription_repository,
            plan_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<ListSubscriptionsQuery> for ListSubscriptionsQueryHandler {
    async fn handle(&self, query: ListSubscriptionsQuery) -> ApplicationResult<ListSubscriptionsResponse> {
        // 1. Calculate pagination
        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(20);
        let offset = ((page - 1) * limit) as i64;

        // 2. Parse optional wallet address
        let wallet_address = if let Some(addr) = &query.wallet_address {
            Some(WalletAddress::from_str(addr)
                .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?)
        } else {
            None
        };

        // 3. Parse optional plan ID
        let plan_id = query.plan_id.map(PlanId::from_i32);

        // 4. Build search criteria
        let criteria = SubscriptionSearchCriteria {
            wallet_address,
            plan_id,
            is_active: query.is_active,
            limit: Some(limit as i64),
            offset: Some(offset),
        };

        // 5. Fetch subscriptions and total count
        let subscriptions = self.subscription_repository.find_all(criteria.clone()).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        let total = self.subscription_repository.count(criteria).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Map to summary DTOs (with plan name lookup)
        let mut subscription_summaries = Vec::new();
        for subscription in subscriptions {
            // Lookup plan name
            let plan = self.plan_repository.find_by_id(subscription.plan_id()).await
                .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

            let plan_name = plan
                .map(|p| p.name().to_string())
                .unwrap_or_else(|| "Unknown Plan".to_string());

            subscription_summaries.push(SubscriptionSummary {
                id: subscription.id().as_str().to_string(),
                wallet_address: subscription.wallet_address().as_str().to_string(),
                plan_id: subscription.plan_id().as_i32(),
                plan_name,
                status: format!("{:?}", subscription.status()),
                started_at: subscription.started_at(),
                expires_at: subscription.expires_at(),
                is_active: subscription.is_active(),
            });
        }

        // 7. Return response
        Ok(ListSubscriptionsResponse {
            subscriptions: subscription_summaries,
            total,
            page,
            limit,
        })
    }
}
