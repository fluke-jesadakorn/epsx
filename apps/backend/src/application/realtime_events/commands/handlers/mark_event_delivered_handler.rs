use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::realtime_events::commands::{
    MarkEventDeliveredCommand, MarkEventDeliveredResponse
};
use crate::domain::realtime_events::EventRepositoryPort;

/// Handler for marking events as delivered
pub struct MarkEventDeliveredCommandHandler {
    event_repository: Arc<dyn EventRepositoryPort>,
}

impl MarkEventDeliveredCommandHandler {
    pub fn new(event_repository: Arc<dyn EventRepositoryPort>) -> Self {
        Self { event_repository }
    }
}

#[async_trait]
impl CommandHandler<MarkEventDeliveredCommand> for MarkEventDeliveredCommandHandler {
    async fn handle(&self, command: MarkEventDeliveredCommand) -> ApplicationResult<MarkEventDeliveredResponse> {
        // 1. Retrieve event
        let mut event = self.event_repository
            .find_by_id(&command.event_id)
            .await
            .map_err(|e| ApplicationError::infrastructure(e))?
            .ok_or_else(|| ApplicationError::not_found("event", command.event_id.to_string()))?;

        // 2. Mark as delivered
        event.mark_delivered()
            .map_err(|e| ApplicationError::business_rule(e.to_string()))?;

        let delivered_at = event.created_at();
        let delivery_attempts = event.delivery_attempts();

        // 3. Save updated event
        self.event_repository
            .save(&event)
            .await
            .map_err(|e| ApplicationError::infrastructure(e))?;

        // 4. Return response
        Ok(MarkEventDeliveredResponse {
            event_id: command.event_id,
            delivered_at,
            delivery_attempts,
        })
    }
}
