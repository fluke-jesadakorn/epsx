use crate::prelude::*;
use crate::application::shared::Query;
use crate::domain::realtime_events::value_objects::{EventId, EventPayload};
use crate::domain::realtime_events::aggregates::{EventStatus, EventPriority};

/// Query to get a realtime event by ID
#[derive(Debug, Clone)]
pub struct GetRealtimeEventQuery {
    pub event_id: EventId,
}

impl Query for GetRealtimeEventQuery {
    type Response = GetRealtimeEventResponse;
}

/// Response containing realtime event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRealtimeEventResponse {
    pub event_id: EventId,
    pub payload: EventPayload,
    pub channel: String,
    pub priority: EventPriority,
    pub status: EventStatus,
    pub delivery_attempts: u32,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}
