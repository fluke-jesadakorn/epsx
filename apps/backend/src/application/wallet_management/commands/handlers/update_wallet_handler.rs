use crate::prelude::*;

use crate::application::shared::{ApplicationResult, ApplicationError, CommandHandler};
use crate::application::wallet_management::{UpdateWalletCommand, UpdateWalletResponse};
use crate::domain::wallet_management::{WalletUserRepositoryPort, WalletAddress};
use crate::infrastructure::cqrs::TransactionalOutbox;

/// Update User Command Handler
/// Handles the business logic for updating user information
/// CQRS-enabled: Uses TransactionalOutbox.append_and_publish_events() for event persistence
pub struct UpdateWalletCommandHandler {
    user_repository: Arc<dyn WalletUserRepositoryPort>,
    outbox: Arc<TransactionalOutbox>,
}

impl UpdateWalletCommandHandler {
    pub fn new(
        user_repository: Arc<dyn WalletUserRepositoryPort>,
        outbox: Arc<TransactionalOutbox>,
    ) -> Self {
        Self {
            user_repository,
            outbox,
        }
    }
}

#[async_trait]
impl CommandHandler<UpdateWalletCommand> for UpdateWalletCommandHandler {
    async fn handle(&self, command: UpdateWalletCommand) -> ApplicationResult<UpdateWalletResponse> {
        tracing::info!("Processing UpdateWalletCommand for wallet_address: {}", command.wallet_address.to_string());
        
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
                let permission = crate::domain::wallet_management::value_objects::Permission::new(permission_str.clone())
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
        
        // Take events from aggregate before saving
        let events = user.take_events();
        let aggregate_id = user.wallet_address().to_string();

        // Save the updated user
        self.user_repository
            .save(&user)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // Append events to outbox for async publishing
        // NOTE: This is not fully atomic with aggregate save (known trade-off)
        // Events are persisted to event_store and outbox_events in a separate transaction
        self.outbox.append_and_publish_events(
            &aggregate_id,
            "WalletUser",
            events,
            None, // causation_id - could use command.command_id if available
            None, // correlation_id - could use request trace_id if available
            None, // user_id - admin who triggered this update
        ).await
        .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        tracing::info!("Successfully updated user: {}", command.wallet_address.to_string());
        
        // Create response (Web3-first approach)
        Ok(UpdateWalletResponse {
            wallet_address: user.wallet_address().to_string(),
            email: format!("{}@wallet.web3", user.wallet_address()), // Web3-first: synthetic email
            email_verified: false, // Web3-first: no email verification needed
            is_active: user.is_active(),
            permissions: user.permissions().clone(),
        })
    }
}