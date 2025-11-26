use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    DeletePermissionGroupCommand, DeletePermissionGroupResponse
};
use crate::domain::permission_management::{PermissionGroupRepositoryPort, GroupId};
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for deleting permission groups
pub struct DeletePermissionGroupCommandHandler {
    group_repository: Arc<dyn PermissionGroupRepositoryPort>,
    _event_bus: Arc<dyn DomainEventBus>,
}

impl DeletePermissionGroupCommandHandler {
    pub fn new(
        group_repository: Arc<dyn PermissionGroupRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            group_repository,
            _event_bus: event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<DeletePermissionGroupCommand> for DeletePermissionGroupCommandHandler {
    async fn handle(&self, command: DeletePermissionGroupCommand) -> ApplicationResult<DeletePermissionGroupResponse> {
        // 1. Parse group ID
        let group_id = GroupId::from_str(&command.group_id)
            .map_err(|e| ApplicationError::validation("group_id", e.to_string()))?;

        // 2. Check if group exists
        let _group = self.group_repository.find_by_id(&group_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("PermissionGroup", command.group_id.clone()))?;

        // 3. Delete group
        self.group_repository.delete(&group_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Return response
        Ok(DeletePermissionGroupResponse {
            group_id: command.group_id,
            deleted: true,
        })
    }
}
