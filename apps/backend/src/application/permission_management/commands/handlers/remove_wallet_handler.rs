use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    RemoveWalletFromPlanCommand, RemoveWalletFromPlanResponse
};
use crate::domain::permission_management::{PlanAssignmentRepositoryPort, PlanId, events::WalletRemovedFromPlanEvent};
use crate::domain::wallet_management::WalletAddress;
// wave11(track-c) R7: migrated from `Arc<dyn DomainEventBus>` to the
// kernel-level `EventPublisherPort`. See `delete_plan_handler.rs` for
// the design notes.
use epsx_contracts::event_publisher_port::EventPublisherPort;

/// Command handler for removing wallets from plans
pub struct RemoveWalletFromPlanCommandHandler {
    assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl RemoveWalletFromPlanCommandHandler {
    pub fn new(
        assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            assignment_repository,
            event_publisher,
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

        // 3. Publish WalletRemovedFromPlanEvent (R7 + R8 wiring — was
        //    _event_bus before wave 10; routed through the new
        //    EventPublisherPort in wave 11). The in-process adapter
        //    is a no-op stub (logs at tracing::info!); no real
        //    consumer exists today.
        let event = WalletRemovedFromPlanEvent::new(
            plan_id.as_str().to_string(),
            0,
            plan_id.as_str().to_string(),
            wallet_address.as_str().to_string(),
            Utc::now(),
        );
        let event_box: Box<dyn crate::domain::shared_kernel::DomainEvent> = Box::new(event);
        if let Err(e) = self.event_publisher.publish(event_box).await {
            tracing::warn!(
                error = %e,
                event = "WalletRemovedFromPlanEvent",
                "EventPublisherPort.publish returned error; command continues"
            );
        }

        // 4. Return response
        Ok(RemoveWalletFromPlanResponse {
            plan_id: command.plan_id,
            wallet_address: wallet_address.as_str().to_string(),
            removed: true,
        })
    }
}
