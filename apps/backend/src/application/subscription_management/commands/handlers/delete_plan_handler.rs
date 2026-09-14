use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::commands::{
    DeletePlanCommand, DeletePlanResponse
};
use crate::domain::subscription_management::{PlanId, PlanRepositoryPort};
use epsx_contracts::event_publisher_port::EventPublisherPort;

/// Command handler for deleting plans
pub struct DeletePlanCommandHandler {
    plan_repository: Arc<dyn PlanRepositoryPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl DeletePlanCommandHandler {
    pub fn new(
        plan_repository: Arc<dyn PlanRepositoryPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            plan_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl CommandHandler<DeletePlanCommand> for DeletePlanCommandHandler {
    async fn handle(&self, command: DeletePlanCommand) -> ApplicationResult<DeletePlanResponse> {
        // 1. Parse plan ID
        let plan_id = PlanId::from_i32(command.plan_id);

        // 2. Verify plan exists
        let plan = self.plan_repository.find_by_id(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("plan_id", "Plan not found"))?;

        // 3. Delete plan
        self.plan_repository.delete(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Publish domain events (if any)
        for event in plan.uncommitted_events() {
            let owned: Box<dyn crate::domain::shared_kernel::DomainEvent> = Box::new(epsx_contracts::domain_event::OwnedEvent::from_borrowed(&**event));
            if let Err(e) = self.event_publisher.publish(owned).await {
                tracing::warn!(
                    error = %e,
                    "EventPublisherPort.publish returned error; command continues"
                );
            }
        }

        // 5. Return response
        Ok(DeletePlanResponse {
            plan_id: command.plan_id,
            deleted: true,
        })
    }
}
