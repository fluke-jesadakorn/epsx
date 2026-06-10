use crate::prelude::*;
use crate::application::shared::Command;
use crate::domain::realtime_events::value_objects::EventId;
use crate::domain::realtime_events::aggregates::EventStatus;

/// Command to mark event as failed
#[derive(Debug, Clone)]
pub struct MarkEventFailedCommand {
    pub event_id: EventId,
    pub failure_reason: String,
}

impl Command for MarkEventFailedCommand {
    type Response = MarkEventFailedResponse;
}

/// Response after marking event failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkEventFailedResponse {
    pub event_id: EventId,
    pub status: EventStatus,
    pub failure_reason: String,
    pub will_retry: bool,
    pub retry_at: Option<DateTime<Utc>>,
}
