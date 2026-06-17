use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    CreatePermissionPlanCommand, CreatePermissionPlanResponse
};
use crate::domain::permission_management::{
    PermissionPlanRepositoryPort, PermissionPlan, PlanSlug, PermissionString,
    aggregates::permission_plan::CreatePermissionPlanParams
};
use epsx_contracts::event_publisher_port::EventPublisherPort;

/// Command handler for creating permission plans
pub struct CreatePermissionPlanCommandHandler {
    plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl CreatePermissionPlanCommandHandler {
    pub fn new(
        plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            plan_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl CommandHandler<CreatePermissionPlanCommand> for CreatePermissionPlanCommandHandler {
    async fn handle(&self, command: CreatePermissionPlanCommand) -> ApplicationResult<CreatePermissionPlanResponse> {
        // 1. Validate slug format
        let slug = PlanSlug::new(&command.slug)
            .map_err(|e| ApplicationError::validation("slug", e.to_string()))?;

        // 2. Check if slug already exists
        if self.plan_repository.slug_exists(&slug).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))? {
            return Err(ApplicationError::conflict("Plan with this slug already exists"));
        }

        // 3. Parse permissions
        let permissions: Result<Vec<PermissionString>, _> = command.permissions
            .iter()
            .map(PermissionString::new)
            .collect();

        let permissions = permissions
            .map_err(|e| ApplicationError::validation("permissions", e.to_string()))?;

        // 4. Create permission plan aggregate
        let plan = PermissionPlan::create(CreatePermissionPlanParams {
            name: command.name.clone(),
            slug,
            description: command.description,
            plan_type: command.plan_type,
            permissions,
            price: command.price,
            currency: command.currency,
            billing_cycle: command.billing_cycle,
            is_active: command.is_active,
            is_promoted: command.is_promoted,
            tier_level: command.tier_level,
            max_members: command.max_members,
            auto_assign_enabled: command.auto_assign_enabled,
            metadata: command.metadata,
            is_public: None, // Default to true via domain model
            grace_period_hours: None,
            plan_category: None,
            plan_group: None,
        }).map_err(ApplicationError::from)?;

        // 5. Save plan
        self.plan_repository.save(&plan).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Publish events
        for event in plan.uncommitted_events() {
            let owned: Box<dyn crate::domain::shared_kernel::DomainEvent> = Box::new(epsx_contracts::domain_event::OwnedEvent::from_borrowed(&**event));
            if let Err(e) = self.event_publisher.publish(owned).await {
                tracing::warn!(
                    error = %e,
                    "EventPublisherPort.publish returned error; command continues"
                );
            }
        }

        // 7. Return response
        Ok(CreatePermissionPlanResponse {
            plan_id: plan.id().as_str(),
            name: command.name,
            slug: plan.slug().as_str().to_string(),
            created_at: plan.created_at(),
        })
    }
}
