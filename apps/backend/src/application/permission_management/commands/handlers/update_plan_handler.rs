use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    UpdatePermissionPlanCommand, UpdatePermissionPlanResponse
};
use crate::domain::permission_management::{
    PermissionPlanRepositoryPort, PlanId, PermissionString, UpdatePermissionPlanParams
};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for updating permission plans
pub struct UpdatePermissionPlanCommandHandler {
    plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl UpdatePermissionPlanCommandHandler {
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
impl CommandHandler<UpdatePermissionPlanCommand> for UpdatePermissionPlanCommandHandler {
    async fn handle(&self, command: UpdatePermissionPlanCommand) -> ApplicationResult<UpdatePermissionPlanResponse> {
        // 1. Parse plan ID
        let plan_id = PlanId::parse(&command.plan_id)
            .map_err(|e| ApplicationError::validation("plan_id", e.to_string()))?;

        // 2. Find plan
        let mut plan = self.plan_repository.find_by_id(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("PermissionPlan", command.plan_id.clone()))?;

        // 3. Parse permissions if provided
        let permissions = if let Some(perms) = command.permissions {
            let parsed: Result<Vec<PermissionString>, _> = perms
                .iter()
                .map(PermissionString::new)
                .collect();
            Some(parsed.map_err(|e| ApplicationError::validation("permissions", e.to_string()))?)
        } else {
            None
        };

        // 4. Update plan
        plan.update(UpdatePermissionPlanParams {
            name: command.name,
            description: command.description,
            permissions,
            price: command.price,
            currency: command.currency,
            billing_cycle: command.billing_cycle,
            is_active: command.is_active,
            is_promoted: command.is_promoted,
            display_order: command.display_order,
            max_members: command.max_members,
            auto_assign_enabled: command.auto_assign_enabled,
            metadata: command.metadata,
            is_public: None, // No visibility change via this handler
            grace_period_hours: None,
        }).map_err(ApplicationError::from)?;

        // 5. Save plan
        self.plan_repository.save(&plan).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Publish events
        for event in plan.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 7. Return response
        Ok(UpdatePermissionPlanResponse {
            plan_id: plan.id().as_str(),
            updated_at: plan.updated_at(),
        })
    }
}
