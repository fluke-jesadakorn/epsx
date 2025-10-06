use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    UpdatePermissionGroupCommand, UpdatePermissionGroupResponse
};
use crate::domain::permission_management::{
    PermissionGroupRepositoryPort, GroupId, PermissionString
};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for updating permission groups
pub struct UpdatePermissionGroupCommandHandler {
    group_repository: Arc<dyn PermissionGroupRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl UpdatePermissionGroupCommandHandler {
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
impl CommandHandler<UpdatePermissionGroupCommand> for UpdatePermissionGroupCommandHandler {
    async fn handle(&self, command: UpdatePermissionGroupCommand) -> ApplicationResult<UpdatePermissionGroupResponse> {
        // 1. Parse group ID
        let group_id = GroupId::from_str(&command.group_id)
            .map_err(|e| ApplicationError::validation("group_id", e.to_string()))?;

        // 2. Find group
        let mut group = self.group_repository.find_by_id(&group_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("PermissionGroup", command.group_id.clone()))?;

        // 3. Parse permissions if provided
        let permissions = if let Some(perms) = command.permissions {
            let parsed: Result<Vec<PermissionString>, _> = perms
                .iter()
                .map(|p| PermissionString::new(p))
                .collect();
            Some(parsed.map_err(|e| ApplicationError::validation("permissions", e.to_string()))?)
        } else {
            None
        };

        // 4. Update group
        group.update(
            command.name,
            command.description,
            permissions,
            command.price,
            command.currency,
            command.billing_cycle,
            command.is_active,
            command.is_promoted,
            command.display_order,
            command.max_members,
            command.auto_assign_enabled,
            command.metadata,
        ).map_err(ApplicationError::from)?;

        // 5. Save group
        self.group_repository.save(&group).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Publish events
        for event in group.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 7. Return response
        Ok(UpdatePermissionGroupResponse {
            group_id: group.id().as_str(),
            updated_at: group.updated_at(),
        })
    }
}
