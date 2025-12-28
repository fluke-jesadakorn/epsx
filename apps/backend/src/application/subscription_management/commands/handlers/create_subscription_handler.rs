use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::commands::{
    CreateSubscriptionCommand, CreateSubscriptionResponse
};
use crate::domain::subscription_management::{
    Subscription, SubscriptionRepositoryPort, PlanRepositoryPort, PlanId, CreateSubscriptionParams
};
use crate::domain::wallet_management::WalletAddress;
use crate::domain::shared_kernel::DomainEventBus;
use std::str::FromStr;

/// Command handler for creating subscriptions
pub struct CreateSubscriptionCommandHandler {
    subscription_repository: Arc<dyn SubscriptionRepositoryPort>,
    plan_repository: Arc<dyn PlanRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl CreateSubscriptionCommandHandler {
    pub fn new(
        subscription_repository: Arc<dyn SubscriptionRepositoryPort>,
        plan_repository: Arc<dyn PlanRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            subscription_repository,
            plan_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<CreateSubscriptionCommand> for CreateSubscriptionCommandHandler {
    async fn handle(&self, command: CreateSubscriptionCommand) -> ApplicationResult<CreateSubscriptionResponse> {
        // 1. Parse wallet address
        let wallet_address = WalletAddress::from_str(&command.wallet_address)
            .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?;

        // 2. Parse plan ID
        let plan_id = PlanId::from_i32(command.plan_id);

        // 3. Fetch plan to get billing cycle information
        let plan = self.plan_repository
            .find_by_id(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("plan", command.plan_id.to_string()))?;

        // 4. Check if active subscription already exists
        if let Some(_existing) = self.subscription_repository
            .find_active_subscription(&wallet_address, &plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
        {
            return Err(ApplicationError::validation(
                "subscription",
                "Active subscription already exists for this wallet and plan"
            ));
        }

        // 5. Calculate subscription expiry based on billing cycle
        use crate::domain::subscription_management::BillingCycle;
        let now = chrono::Utc::now();
        let expires_at = match plan.billing_cycle() {
            BillingCycle::PayPerUse => None,
            BillingCycle::Monthly => Some(now + chrono::Duration::days(30)),
            BillingCycle::Quarterly => Some(now + chrono::Duration::days(90)),
            BillingCycle::Yearly => Some(now + chrono::Duration::days(365)),
            BillingCycle::Lifetime => None, // Lifetime subscriptions don't expire
        };

        // 6. Create subscription aggregate
        let subscription = Subscription::create(CreateSubscriptionParams {
            wallet_address: wallet_address.clone(),
            plan_id: plan_id.clone(),
            expires_at,
            payment_method_id: command.payment_method_id,
        });

        // 7. Save subscription
        self.subscription_repository.save(&subscription).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 8. Publish domain events
        for event in subscription.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 9. Return response
        Ok(CreateSubscriptionResponse {
            subscription_id: subscription.id().as_str().to_string(),
            wallet_address: command.wallet_address,
            plan_id: command.plan_id,
            started_at: subscription.started_at(),
            expires_at: subscription.expires_at(),
        })
    }
}
