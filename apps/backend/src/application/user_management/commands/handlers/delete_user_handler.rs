use async_trait::async_trait;
use std::sync::Arc;

use crate::application::shared::{ApplicationResult, ApplicationError, CommandHandler};
use crate::application::user_management::{DeleteUserCommand, DeleteUserResponse};
use crate::domain::shared_kernel::{DomainEventBus, AggregateRoot};
use crate::domain::user_management::{WalletUserRepositoryPort, WalletAddress};

/// Delete User Command Handler
/// Handles the business logic for deleting a user from the system
pub struct DeleteUserCommandHandler {
    user_repository: Arc<dyn WalletUserRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl DeleteUserCommandHandler {
    pub fn new(
        user_repository: Arc<dyn WalletUserRepositoryPort>,
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
        tracing::info!("Processing DeleteUserCommand for wallet_address: {}", command.wallet_address.to_string());
        
        // Find user by wallet address (Web3-first approach)
        let wallet_address = WalletAddress::new(command.wallet_address.to_string())?;
        let user = self.user_repository
            .find_by_wallet(&wallet_address)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", command.wallet_address.to_string()))?;
        
        // Delete the user (this should trigger domain events)
        self.user_repository
            .delete(&wallet_address)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // Publish domain events if any were generated
        for event in user.uncommitted_events() {
            self.event_bus.publish(event);
        }
        
        tracing::info!("Successfully deleted user: {}", command.wallet_address.to_string());
        
        // Create response
        Ok(DeleteUserResponse::new(
            format!("User {} deleted successfully", command.wallet_address.to_string())
        ))
    }
}