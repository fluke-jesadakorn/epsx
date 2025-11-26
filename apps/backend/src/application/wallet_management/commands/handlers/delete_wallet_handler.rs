use crate::prelude::*;

use crate::application::shared::{ApplicationResult, ApplicationError, CommandHandler};
use crate::application::wallet_management::{DeleteWalletCommand, DeleteWalletResponse};
use crate::domain::wallet_management::{WalletUserRepositoryPort, WalletAddress};
use crate::infrastructure::cqrs::TransactionalOutbox;

/// Delete Wallet Command Handler
/// Handles the business logic for deleting a wallet from the system
/// CQRS-enabled: Uses TransactionalOutbox.append_and_publish_events() for event persistence
pub struct DeleteWalletCommandHandler {
    user_repository: Arc<dyn WalletUserRepositoryPort>,
    outbox: Arc<TransactionalOutbox>,
}

impl DeleteWalletCommandHandler {
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
impl CommandHandler<DeleteWalletCommand> for DeleteWalletCommandHandler {
    async fn handle(&self, command: DeleteWalletCommand) -> ApplicationResult<DeleteWalletResponse> {
        tracing::info!("Processing DeleteWalletCommand for wallet_address: {}", command.wallet_address.to_string());
        
        // Find wallet by wallet address (Web3-first approach)
        let wallet_address = WalletAddress::new(command.wallet_address.to_string())?;
        let mut wallet = self.user_repository
            .find_by_wallet(&wallet_address)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("Wallet", command.wallet_address.to_string()))?;

        // Take events from aggregate before deletion
        let events = wallet.take_events();
        let aggregate_id = wallet.wallet_address().to_string();

        // Delete the wallet (this should trigger domain events)
        self.user_repository
            .delete(&wallet_address)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // Append events to outbox for async publishing
        // NOTE: This is not fully atomic with aggregate delete (known trade-off)
        // Events are persisted to event_store and outbox_events in a separate transaction
        self.outbox.append_and_publish_events(
            &aggregate_id,
            "WalletUser",
            events,
            None, // causation_id - could use command.command_id if available
            None, // correlation_id - could use request trace_id if available
            None, // user_id - admin who triggered this deletion
        ).await
        .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        tracing::info!("Successfully deleted wallet: {}", command.wallet_address.to_string());

        // Create response
        Ok(DeleteWalletResponse::new(
            format!("Wallet {} deleted successfully", command.wallet_address)
        ))
    }
}