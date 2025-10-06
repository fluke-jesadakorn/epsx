use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    RemoveWalletFromGroupCommand, RemoveWalletFromGroupResponse
};
use crate::domain::permission_management::{GroupAssignmentRepositoryPort, GroupId};
use crate::domain::wallet_management::WalletAddress;
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for removing wallets from groups
pub struct RemoveWalletFromGroupCommandHandler {
    assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
    _event_bus: Arc<dyn DomainEventBus>,
}

impl RemoveWalletFromGroupCommandHandler {
    pub fn new(
        assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            assignment_repository,
            _event_bus: event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<RemoveWalletFromGroupCommand> for RemoveWalletFromGroupCommandHandler {
    async fn handle(&self, command: RemoveWalletFromGroupCommand) -> ApplicationResult<RemoveWalletFromGroupResponse> {
        // 1. Parse group ID and wallet address
        let group_id = GroupId::from_str(&command.group_id)
            .map_err(|e| ApplicationError::validation("group_id", e.to_string()))?;

        let wallet_address = WalletAddress::new(&command.wallet_address)
            .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?;

        // 2. Delete assignment
        self.assignment_repository.delete(&wallet_address, &group_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Return response
        Ok(RemoveWalletFromGroupResponse {
            group_id: command.group_id,
            wallet_address: wallet_address.as_str().to_string(),
            removed: true,
        })
    }
}
