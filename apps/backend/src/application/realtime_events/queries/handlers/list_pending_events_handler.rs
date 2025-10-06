use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::realtime_events::queries::{
    ListPendingEventsQuery, ListPendingEventsResponse, PendingEventSummary
};
use crate::domain::realtime_events::EventRepositoryPort;

/// Handler for listing pending events
pub struct ListPendingEventsQueryHandler {
    event_repository: Arc<dyn EventRepositoryPort>,
}

impl ListPendingEventsQueryHandler {
    pub fn new(event_repository: Arc<dyn EventRepositoryPort>) -> Self {
        Self { event_repository }
    }
}

#[async_trait]
impl QueryHandler<ListPendingEventsQuery> for ListPendingEventsQueryHandler {
    async fn handle(&self, query: ListPendingEventsQuery) -> ApplicationResult<ListPendingEventsResponse> {
        // 1. Retrieve pending events
        let events = self.event_repository
            .find_pending_events(query.limit)
            .await
            .map_err(|e| ApplicationError::infrastructure(e))?;

        // 2. Map to summaries
        let summaries: Vec<PendingEventSummary> = events.iter().map(|event| {
            PendingEventSummary {
                event_id: event.id().to_string(),
                channel: event.channel().to_string(),
                target_user_count: event.target_users().len(),
                priority: format!("{:?}", event.priority()),
                delivery_attempts: event.delivery_attempts(),
                created_at: event.created_at(),
            }
        }).collect();

        let count = summaries.len();

        // 3. Return response
        Ok(ListPendingEventsResponse { events: summaries, count })
    }
}
