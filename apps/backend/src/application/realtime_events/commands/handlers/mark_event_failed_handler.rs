use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::realtime_events::commands::{
    MarkEventFailedCommand, MarkEventFailedResponse
};
use crate::domain::realtime_events::{EventRepositoryPort, aggregates::EventStatus};

/// Handler for marking events as failed
pub struct MarkEventFailedCommandHandler {
    event_repository: Arc<dyn EventRepositoryPort>,
}

impl MarkEventFailedCommandHandler {
    pub fn new(event_repository: Arc<dyn EventRepositoryPort>) -> Self {
        Self { event_repository }
    }
}

#[async_trait]
impl CommandHandler<MarkEventFailedCommand> for MarkEventFailedCommandHandler {
    async fn handle(&self, command: MarkEventFailedCommand) -> ApplicationResult<MarkEventFailedResponse> {
        // 1. Retrieve event
        let mut event = self.event_repository
            .find_by_id(&command.event_id)
            .await
            .map_err(ApplicationError::infrastructure)?
            .ok_or_else(|| ApplicationError::not_found("event", command.event_id.to_string()))?;

        // 2. Mark as failed (will retry if attempts < max)
        event.mark_failed(command.failure_reason.clone())
            .map_err(|e| ApplicationError::business_rule(e.to_string()))?;

        let status = event.status().clone();
        let will_retry = matches!(status, EventStatus::Retrying);
        let retry_at = if will_retry {
            Some(event.created_at()) // In real impl, get from scheduled_for field
        } else {
            None
        };

        // 3. Save updated event
        self.event_repository
            .save(&event)
            .await
            .map_err(ApplicationError::infrastructure)?;

        // 4. Return response
        Ok(MarkEventFailedResponse {
            event_id: command.event_id,
            status,
            failure_reason: command.failure_reason,
            will_retry,
            retry_at,
        })
    }
}
