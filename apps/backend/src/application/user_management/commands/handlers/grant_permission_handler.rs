use async_trait::async_trait;
use std::sync::Arc;

use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::user_management::commands::models::{GrantPermissionCommand, GrantPermissionResponse};

use crate::domain::shared_kernel::{DomainEventBus, AggregateRoot};
use crate::domain::user_management::{WalletUserRepositoryPort, Permission, WalletAddress};

/// Command handler for granting permissions to users
pub struct GrantPermissionCommandHandler {
    user_repository: Arc<dyn WalletUserRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl GrantPermissionCommandHandler {
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
impl CommandHandler<GrantPermissionCommand> for GrantPermissionCommandHandler {
    async fn handle(&self, command: GrantPermissionCommand) -> ApplicationResult<GrantPermissionResponse> {
        // TODO: Implement the full handler logic
        // This is a stub implementation for now
        
        // 1. Parse wallet address and permission
        let wallet_addr = WalletAddress::new(command.wallet_address.clone())?;
        
        let permission = Permission::new(&command.permission)
            .map_err(|e| ApplicationError::validation("permission", e.to_string()))?;
        
        // 2. Find user
        let mut user = self.user_repository.find_by_wallet(&wallet_addr).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", wallet_addr.to_string()))?;
        
        // 3. Grant permission using domain logic (Web3-first)
        let granted_by = command.granted_by.clone();
        
        user.grant_permission(permission.clone())
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
            wallet_address: user.wallet_address().to_string(),
            permission,
            granted_by,
            granted_at: chrono::Utc::now(),
            expires_at: command.expires_at,
        })
    }
}