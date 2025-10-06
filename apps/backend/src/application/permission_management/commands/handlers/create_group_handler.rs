use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    CreatePermissionGroupCommand, CreatePermissionGroupResponse
};
use crate::domain::permission_management::{
    PermissionGroupRepositoryPort, PermissionGroup, GroupSlug, PermissionString,
    aggregates::permission_group::CreatePermissionGroupParams
};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for creating permission groups
pub struct CreatePermissionGroupCommandHandler {
    group_repository: Arc<dyn PermissionGroupRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl CreatePermissionGroupCommandHandler {
    pub fn new(
        group_repository: Arc<dyn PermissionGroupRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            group_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<CreatePermissionGroupCommand> for CreatePermissionGroupCommandHandler {
    async fn handle(&self, command: CreatePermissionGroupCommand) -> ApplicationResult<CreatePermissionGroupResponse> {
        // 1. Validate slug format
        let slug = GroupSlug::new(&command.slug)
            .map_err(|e| ApplicationError::validation("slug", e.to_string()))?;

        // 2. Check if slug already exists
        if self.group_repository.slug_exists(&slug).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))? {
            return Err(ApplicationError::conflict("Group with this slug already exists"));
        }

        // 3. Parse permissions
        let permissions: Result<Vec<PermissionString>, _> = command.permissions
            .iter()
            .map(|p| PermissionString::new(p))
            .collect();

        let permissions = permissions
            .map_err(|e| ApplicationError::validation("permissions", e.to_string()))?;

        // 4. Create permission group aggregate
        let group = PermissionGroup::create(CreatePermissionGroupParams {
            name: command.name.clone(),
            slug,
            description: command.description,
            group_type: command.group_type,
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
        }).map_err(ApplicationError::from)?;

        // 5. Save group
        self.group_repository.save(&group).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Publish events
        for event in group.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 7. Return response
        Ok(CreatePermissionGroupResponse {
            group_id: group.id().as_str(),
            name: command.name,
            slug: group.slug().as_str().to_string(),
            created_at: group.created_at(),
        })
    }
}
