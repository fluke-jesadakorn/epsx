use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::commands::{
    CancelSubscriptionCommand, CancelSubscriptionResponse
};
use crate::domain::subscription_management::{SubscriptionId, SubscriptionRepositoryPort};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for cancelling subscriptions
pub struct CancelSubscriptionCommandHandler {
    subscription_repository: Arc<dyn SubscriptionRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl CancelSubscriptionCommandHandler {
    pub fn new(
        subscription_repository: Arc<dyn SubscriptionRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            subscription_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<CancelSubscriptionCommand> for CancelSubscriptionCommandHandler {
    async fn handle(&self, command: CancelSubscriptionCommand) -> ApplicationResult<CancelSubscriptionResponse> {
        // 1. Parse subscription ID
        let subscription_id = SubscriptionId::from_str(&command.subscription_id)
            .map_err(|e| ApplicationError::validation("subscription_id", e.to_string()))?;

        // 2. Load subscription
        let mut subscription = self.subscription_repository.find_by_id(&subscription_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("subscription_id", "Subscription not found"))?;

        // 3. Cancel subscription
        subscription.cancel()
            .map_err(|e| ApplicationError::business_logic(e.to_string()))?;

        // 4. Save updated subscription
        self.subscription_repository.save(&subscription).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 5. Publish domain events
        for event in subscription.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 6. Return response
        Ok(CancelSubscriptionResponse {
            subscription_id: command.subscription_id,
            cancelled_at: Utc::now(),
        })
    }
}
