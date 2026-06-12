use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    RemoveWalletFromPlanCommand, RemoveWalletFromPlanResponse
};
use crate::domain::permission_management::{PlanAssignmentRepositoryPort, PlanId, events::WalletRemovedFromPlanEvent};
use crate::domain::wallet_management::WalletAddress;
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for removing wallets from plans
pub struct RemoveWalletFromPlanCommandHandler {
    assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl RemoveWalletFromPlanCommandHandler {
    pub fn new(
        assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            assignment_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<RemoveWalletFromPlanCommand> for RemoveWalletFromPlanCommandHandler {
    async fn handle(&self, command: RemoveWalletFromPlanCommand) -> ApplicationResult<RemoveWalletFromPlanResponse> {
        // 1. Parse plan ID and wallet address
        let plan_id = PlanId::parse(&command.plan_id)
            .map_err(|e| ApplicationError::validation("plan_id", e.to_string()))?;

        let wallet_address = WalletAddress::new(&command.wallet_address)
            .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?;

        // 2. Delete assignment
        self.assignment_repository.delete(&wallet_address, &plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Publish WalletRemovedFromPlanEvent (R8 wiring — was _event_bus before)
        let event = WalletRemovedFromPlanEvent::new(
            plan_id.as_str().to_string(),
            0,
            plan_id.as_str().to_string(),
            wallet_address.as_str().to_string(),
            Utc::now(),
        );
        self.event_bus.publish(&event);

        // 4. Return response
        Ok(RemoveWalletFromPlanResponse {
            plan_id: command.plan_id,
            wallet_address: wallet_address.as_str().to_string(),
            removed: true,
        })
    }
}
