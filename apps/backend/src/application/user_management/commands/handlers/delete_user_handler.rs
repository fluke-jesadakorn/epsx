use std::sync::Arc;
use async_trait::async_trait;

use crate::application::shared::{ApplicationResult, ApplicationError, CommandHandler};
use crate::application::user_management::{DeleteUserCommand, DeleteUserResponse};
use crate::domain::shared_kernel::{DomainEventBus, AggregateRoot};
use crate::domain::user_management::UserRepositoryPort;

/// Delete User Command Handler
/// Handles the business logic for deleting a user from the system
pub struct DeleteUserCommandHandler {
    user_repository: Arc<dyn UserRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl DeleteUserCommandHandler {
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
impl CommandHandler<DeleteUserCommand> for DeleteUserCommandHandler {
    async fn handle(&self, command: DeleteUserCommand) -> ApplicationResult<DeleteUserResponse> {
        tracing::info!("Processing DeleteUserCommand for firebase_uid: {}", command.firebase_uid.to_string());
        
        // Find user by Firebase UID
        let user = self.user_repository
            .find_by_firebase_uid(&command.firebase_uid)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", command.firebase_uid.to_string()))?;
        
        // Delete the user (this should trigger domain events)
        self.user_repository
            .delete(&user.id())
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // Publish domain events if any were generated
        for event in user.uncommitted_events() {
            self.event_bus.publish(event);
        }
        
        tracing::info!("Successfully deleted user: {}", command.firebase_uid.to_string());
        
        // Create response
        Ok(DeleteUserResponse::new(
            format!("User {} deleted successfully", command.firebase_uid.to_string())
        ))
    }
}