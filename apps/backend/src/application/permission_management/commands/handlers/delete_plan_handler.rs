use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    DeletePermissionPlanCommand, DeletePermissionPlanResponse
};
use crate::domain::permission_management::{PermissionPlanRepositoryPort, PlanId, events::PlanDeletedEvent};
use epsx_contracts::traits::DomainEventBus;

/// Command handler for deleting permission plans
pub struct DeletePermissionPlanCommandHandler {
    plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl DeletePermissionPlanCommandHandler {
    pub fn new(
        plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            plan_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<DeletePermissionPlanCommand> for DeletePermissionPlanCommandHandler {
    async fn handle(&self, command: DeletePermissionPlanCommand) -> ApplicationResult<DeletePermissionPlanResponse> {
        // 1. Parse plan ID
        let plan_id = PlanId::parse(&command.plan_id)
            .map_err(|e| ApplicationError::validation("plan_id", e.to_string()))?;

        // 2. Check if plan exists
        let _plan = self.plan_repository.find_by_id(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("PermissionPlan", command.plan_id.clone()))?;

        // 3. Delete plan
        self.plan_repository.delete(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Publish PlanDeletedEvent (R8 wiring — was _event_bus before)
        let event = PlanDeletedEvent::new(
            plan_id.as_str().to_string(),
            0, // aggregate version not tracked per-delete; event-sourcing build fills this
            plan_id.as_str().to_string(),
            Utc::now(),
        );
        self.event_bus.publish(&event);

        // 5. Return response
        Ok(DeletePermissionPlanResponse {
            plan_id: command.plan_id,
            deleted: true,
        })
    }
}
