use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    DeletePermissionPlanCommand, DeletePermissionPlanResponse
};
use crate::domain::permission_management::{PermissionPlanRepositoryPort, PlanId};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for deleting permission plans
pub struct DeletePermissionPlanCommandHandler {
    plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
    _event_bus: Arc<dyn DomainEventBus>,
}

impl DeletePermissionPlanCommandHandler {
    pub fn new(
        plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            plan_repository,
            _event_bus: event_bus,
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

        // 4. Return response
        Ok(DeletePermissionPlanResponse {
            plan_id: command.plan_id,
            deleted: true,
        })
    }
}
