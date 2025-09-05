use std::sync::Arc;
use async_trait::async_trait;

use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::user_management::commands::models::{GrantPermissionCommand, GrantPermissionResponse};

use crate::domain::shared_kernel::{DomainEventBus, AggregateRoot};
use crate::domain::user_management::{UserRepositoryPort, Permission, UserId};

/// Command handler for granting permissions to users
pub struct GrantPermissionCommandHandler {
    user_repository: Arc<dyn UserRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl GrantPermissionCommandHandler {
    pub fn new(
        user_repository: Arc<dyn UserRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            user_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<GrantPermissionCommand> for GrantPermissionCommandHandler {
    async fn handle(&self, command: GrantPermissionCommand) -> ApplicationResult<GrantPermissionResponse> {
        // TODO: Implement the full handler logic
        // This is a stub implementation for now
        
        // 1. Parse user ID and permission
        let user_id = UserId::from_string(&command.user_id)
            .map_err(|e| ApplicationError::validation("user_id", e.to_string()))?;
        
        let permission = Permission::new(&command.permission)
            .map_err(|e| ApplicationError::validation("permission", e.to_string()))?;
        
        // 2. Find user
        let mut user = self.user_repository.find_by_id(&user_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", command.user_id.clone()))?;
        
        // 3. Grant permission using domain logic
        let granted_by = if let Some(ref granted_by_str) = command.granted_by {
            Some(UserId::from_string(granted_by_str)
                .map_err(|e| ApplicationError::validation("granted_by", e.to_string()))?)
        } else {
            None
        };
        
        user.grant_permission(permission.clone(), granted_by.clone())
            .map_err(ApplicationError::from)?;
        
        // 4. Save user
        self.user_repository.save(&user).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // 5. Publish events
        for event in user.uncommitted_events() {
            self.event_bus.publish(event);
        }
        
        // 6. Return response
        Ok(GrantPermissionResponse {
            user_id: user.id().clone(),
            permission,
            granted_by,
            granted_at: chrono::Utc::now(),
            expires_at: command.expires_at,
        })
    }
}