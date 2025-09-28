use async_trait::async_trait;
use std::sync::Arc;

use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::user_management::commands::models::{CreateSessionCommand, CreateSessionResponse};

use crate::domain::shared_kernel::{DomainEventBus, AggregateRoot};
use crate::domain::user_management::{WalletUserRepositoryPort, SessionRepositoryPort, WalletAddress};
use crate::domain::user_management::aggregates::Session;

/// Command handler for creating user sessions
pub struct CreateSessionCommandHandler {
    user_repository: Arc<dyn WalletUserRepositoryPort>,
    session_repository: Arc<dyn SessionRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl CreateSessionCommandHandler {
    pub fn new(
        user_repository: Arc<dyn WalletUserRepositoryPort>,
        session_repository: Arc<dyn SessionRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            user_repository,
            session_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<CreateSessionCommand> for CreateSessionCommandHandler {
    async fn handle(&self, command: CreateSessionCommand) -> ApplicationResult<CreateSessionResponse> {
        // TODO: Implement the full handler logic
        // This is a stub implementation for now
        
        // 1. Validate user exists and is active
        let wallet_addr = WalletAddress::new(command.wallet_address.clone())?;
        
        let user = self.user_repository.find_by_wallet(&wallet_addr).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", wallet_addr.to_string()))?;
        
        if !user.is_active() {
            return Err(ApplicationError::authorization("Cannot create session for inactive user"));
        }
        
        // 2. Generate session ID
        let session_id = self.session_repository.next_identity().await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // 3. Create session using domain logic
        let session = Session::create(
            session_id,
            wallet_addr.clone(),
            command.access_token,
            command.expires_at,
            command.ip_address,
            command.user_agent,
        ).map_err(ApplicationError::from)?;
        
        // 4. Save session
        self.session_repository.save(&session).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        // 5. Publish events
        for event in session.uncommitted_events() {
            self.event_bus.publish(event);
        }
        
        // 6. Return response
        Ok(CreateSessionResponse {
            session_id: session.id().clone(),
            wallet_address: wallet_addr.to_user_id(),
            created_at: session.created_at(),
            expires_at: session.expires_at(),
            is_valid: session.is_valid(),
        })
    }
}