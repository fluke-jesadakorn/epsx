use crate::prelude::*;

use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::wallet_management::commands::models::{CreateSessionCommand, CreateSessionResponse};
use crate::domain::wallet_management::{WalletUserRepositoryPort, SessionRepositoryPort, WalletAddress};
use crate::domain::wallet_management::aggregates::Session;
use crate::infrastructure::cqrs::TransactionalOutbox;

/// Command handler for creating wallet sessions
/// CQRS-enabled: Uses TransactionalOutbox.append_and_publish_events() for event persistence
pub struct CreateSessionCommandHandler {
    user_repository: Arc<dyn WalletUserRepositoryPort>,
    session_repository: Arc<dyn SessionRepositoryPort>,
    outbox: Arc<TransactionalOutbox>,
}

impl CreateSessionCommandHandler {
    pub fn new(
        user_repository: Arc<dyn WalletUserRepositoryPort>,
        session_repository: Arc<dyn SessionRepositoryPort>,
        outbox: Arc<TransactionalOutbox>,
    ) -> Self {
        Self {
            user_repository,
            session_repository,
            outbox,
        }
    }
}

#[async_trait]
impl CommandHandler<CreateSessionCommand> for CreateSessionCommandHandler {
    async fn handle(&self, command: CreateSessionCommand) -> ApplicationResult<CreateSessionResponse> {
        // 1. Validate wallet exists and is active
        let wallet_addr = WalletAddress::new(command.wallet_address.clone())?;

        let wallet = self.user_repository.find_by_wallet(&wallet_addr).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("Wallet", wallet_addr.to_string()))?;

        if !wallet.is_active() {
            return Err(ApplicationError::authorization("Cannot create session for inactive wallet"));
        }
        
        // 2. Generate session ID
        let session_id = self.session_repository.next_identity().await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // 3. Create session using domain logic
        let mut session = Session::create(
            session_id,
            wallet_addr.clone(),
            command.access_token,
            command.expires_at,
            command.ip_address,
            command.user_agent,
        ).map_err(ApplicationError::from)?;

        // 4. Take events from aggregate before saving
        let events = session.take_events();
        let aggregate_id = session.id().to_string();

        // 5. Save session
        self.session_repository.save(&session).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 6. Append events to outbox for async publishing
        // NOTE: This is not fully atomic with aggregate save (known trade-off)
        // Events are persisted to event_store and outbox_events in a separate transaction
        self.outbox.append_and_publish_events(
            &aggregate_id,
            "Session",
            events,
            None, // causation_id - could use command.command_id if available
            None, // correlation_id - could use request trace_id if available
            Some(wallet_addr.to_string()), // user_id who created this session
        ).await
        .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 7. Return response
        Ok(CreateSessionResponse {
            session_id: session.id().clone(),
            wallet_address: wallet_addr.to_user_id(),
            created_at: session.created_at(),
            expires_at: session.expires_at(),
            is_valid: session.is_valid(),
        })
    }
}