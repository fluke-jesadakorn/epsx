use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::realtime_events::commands::{
    CreateRealtimeEventCommand, CreateRealtimeEventResponse
};
use crate::domain::realtime_events::{
    RealtimeEvent, EventRepositoryPort
};

/// Handler for creating realtime events
pub struct CreateRealtimeEventCommandHandler {
    event_repository: Arc<dyn EventRepositoryPort>,
}

impl CreateRealtimeEventCommandHandler {
    pub fn new(event_repository: Arc<dyn EventRepositoryPort>) -> Self {
        Self { event_repository }
    }
}

#[async_trait]
impl CommandHandler<CreateRealtimeEventCommand> for CreateRealtimeEventCommandHandler {
    async fn handle(&self, command: CreateRealtimeEventCommand) -> ApplicationResult<CreateRealtimeEventResponse> {
        // 1. Create event aggregate
        let event = if command.is_broadcast {
            RealtimeEvent::create_broadcast(command.payload, command.channel)
        } else {
            RealtimeEvent::create(command.payload, command.target_users, command.channel)
        }.map_err(|e| ApplicationError::business_rule(e.to_string()))?;

        let event_id = event.id().clone();
        let channel = event.channel().to_string();
        let target_user_count = event.target_users().len();
        let created_at = event.created_at();

        // 2. Save to repository
        self.event_repository
            .save(&event)
            .await
            .map_err(|e| ApplicationError::infrastructure(e))?;

        // 3. Return response
        Ok(CreateRealtimeEventResponse {
            event_id,
            channel,
            target_user_count,
            created_at,
        })
    }
}
