use crate::prelude::*;

use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::wallet_management::commands::models::{GrantPermissionCommand, GrantPermissionResponse};
use crate::domain::wallet_management::{WalletUserRepositoryPort, Permission, WalletAddress};
use crate::infrastructure::cqrs::TransactionalOutbox;

/// Command handler for granting permissions to users
/// CQRS-enabled: Uses TransactionalOutbox.append_and_publish_events() for event persistence
pub struct GrantPermissionCommandHandler {
    user_repository: Arc<dyn WalletUserRepositoryPort>,
    outbox: Arc<TransactionalOutbox>,
}

impl GrantPermissionCommandHandler {
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
impl CommandHandler<GrantPermissionCommand> for GrantPermissionCommandHandler {
    async fn handle(&self, command: GrantPermissionCommand) -> ApplicationResult<GrantPermissionResponse> {
        // 1. Parse wallet address and permission
        let wallet_addr = WalletAddress::new(command.wallet_address.clone())?;

        let permission = Permission::new(&command.permission)
            .map_err(|e| ApplicationError::validation("permission", e.to_string()))?;

        // 2. Load aggregate (wallet user)
        let mut user = self.user_repository.find_by_wallet(&wallet_addr).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", wallet_addr.to_string()))?;

        // 3. Execute business logic (this creates domain events)
        let granted_by = command.granted_by.clone();

        user.grant_permission(permission.clone())
            .map_err(ApplicationError::from)?;

        // 4. Take events from aggregate before saving
        let events = user.take_events();
        let aggregate_id = user.wallet_address().to_string();

        // 5. Save aggregate state via repository
        self.user_repository.save(&user).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Append events to outbox for async publishing
        // NOTE: This is not fully atomic with aggregate save (known trade-off)
        // Events are persisted to event_store and outbox_events in a separate transaction
        self.outbox.append_and_publish_events(
            &aggregate_id,
            "WalletUser",
            events,
            None, // causation_id - could use command.command_id if available
            None, // correlation_id - could use request trace_id if available
            granted_by.clone(), // user_id who triggered this
        ).await
        .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 7. Return response
        Ok(GrantPermissionResponse {
            wallet_address: user.wallet_address().to_string(),
            permission,
            granted_by,
            granted_at: chrono::Utc::now(),
            expires_at: command.expires_at,
        })
    }
}