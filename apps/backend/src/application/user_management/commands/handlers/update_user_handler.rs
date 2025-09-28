use async_trait::async_trait;
use std::sync::Arc;

use crate::application::shared::{ApplicationResult, ApplicationError, CommandHandler};
use crate::application::user_management::{UpdateUserCommand, UpdateUserResponse};
use crate::domain::shared_kernel::{DomainEventBus, AggregateRoot};
use crate::domain::user_management::{WalletUserRepositoryPort, WalletAddress};

/// Update User Command Handler
/// Handles the business logic for updating user information
pub struct UpdateUserCommandHandler {
    user_repository: Arc<dyn WalletUserRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl UpdateUserCommandHandler {
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
impl CommandHandler<UpdateUserCommand> for UpdateUserCommandHandler {
    async fn handle(&self, command: UpdateUserCommand) -> ApplicationResult<UpdateUserResponse> {
        tracing::info!("Processing UpdateUserCommand for wallet_address: {}", command.wallet_address.to_string());
        
        // Find user by wallet address (Web3 migration)
        let wallet_addr = WalletAddress::new(command.wallet_address.clone())?;
        let mut user = self.user_repository
            .find_by_wallet(&wallet_addr)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", command.wallet_address.to_string()))?;
        
        // Web3-first: Email operations not supported
        if command.email.is_some() {
            return Err(ApplicationError::not_implemented("Email updates not supported in Web3-first architecture"));
        }
        
        // Update permissions if provided
        if let Some(permissions) = &command.permissions {
            // Convert string permissions to Permission value objects
            let mut new_permissions = std::collections::HashSet::new();
            for permission_str in permissions {
                let permission = crate::domain::user_management::value_objects::Permission::new(permission_str.clone())
                    .map_err(|e| ApplicationError::validation("permission", e.to_string()))?;
                new_permissions.insert(permission);
            }
            
            // Update all permissions at once
            user.update_permissions(new_permissions)
                .map_err(ApplicationError::from)?;
        }
        
        // Update active status if provided
        if let Some(is_active) = command.is_active {
            if is_active {
                user.activate()
                    .map_err(ApplicationError::from)?;
            } else {
                user.deactivate("Admin deactivated".to_string())
                    .map_err(ApplicationError::from)?;
            }
        }
        
        // Web3-first: Email verification not supported
        if command.email_verified.is_some() {
            return Err(ApplicationError::not_implemented("Email verification not supported in Web3-first architecture"));
        }
        
        // Save the updated user
        self.user_repository
            .save(&user)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // Publish domain events
        for event in user.uncommitted_events() {
            self.event_bus.publish(event);
        }
        
        // Clear events after publishing
        user.mark_events_as_committed();
        
        tracing::info!("Successfully updated user: {}", command.wallet_address.to_string());
        
        // Create response (Web3-first approach)
        Ok(UpdateUserResponse {
            wallet_address: user.wallet_address().to_string(),
            email: format!("{}@wallet.web3", user.wallet_address().to_string()), // Web3-first: synthetic email
            email_verified: false, // Web3-first: no email verification needed
            is_active: user.is_active(),
            permissions: user.permissions().clone(),
        })
    }
}