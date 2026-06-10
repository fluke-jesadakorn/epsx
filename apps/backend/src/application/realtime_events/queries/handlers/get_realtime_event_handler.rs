use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::realtime_events::queries::{
    GetRealtimeEventQuery, GetRealtimeEventResponse
};
use crate::domain::realtime_events::EventRepositoryPort;

/// Handler for getting realtime event by ID
pub struct GetRealtimeEventQueryHandler {
    event_repository: Arc<dyn EventRepositoryPort>,
}

impl GetRealtimeEventQueryHandler {
    pub fn new(event_repository: Arc<dyn EventRepositoryPort>) -> Self {
        Self { event_repository }
    }
}

#[async_trait]
impl QueryHandler<GetRealtimeEventQuery> for GetRealtimeEventQueryHandler {
    async fn handle(&self, query: GetRealtimeEventQuery) -> ApplicationResult<GetRealtimeEventResponse> {
        // 1. Retrieve event
        let event = self.event_repository
            .find_by_id(&query.event_id)
            .await
            .map_err(ApplicationError::infrastructure)?
            .ok_or_else(|| ApplicationError::not_found("event", query.event_id.to_string()))?;

        // 2. Map to response
        Ok(GetRealtimeEventResponse {
            event_id: event.id().clone(),
            payload: event.payload().clone(),
            channel: event.channel().to_string(),
            priority: event.priority(),
            status: event.status().clone(),
            delivery_attempts: event.delivery_attempts(),
            created_at: event.created_at(),
            delivered_at: None, // Would need getter on aggregate
            failure_reason: None, // Would need getter on aggregate
        })
    }
}
