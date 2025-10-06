use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::commands::{
    DeletePlanCommand, DeletePlanResponse
};
use crate::domain::subscription_management::{PlanId, PlanRepositoryPort};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for deleting plans
pub struct DeletePlanCommandHandler {
    plan_repository: Arc<dyn PlanRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl DeletePlanCommandHandler {
    pub fn new(
        plan_repository: Arc<dyn PlanRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            plan_repository,
            event_bus,
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
            self.event_bus.publish(&**event);
        }

        // 5. Return response
        Ok(DeletePlanResponse {
            plan_id: command.plan_id,
            deleted: true,
        })
    }
}
