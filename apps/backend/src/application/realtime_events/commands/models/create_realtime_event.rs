use crate::prelude::*;
use crate::application::shared::Command;
use crate::domain::realtime_events::value_objects::{EventPayload, EventId, UserId};

/// Command to create a new realtime event
#[derive(Debug, Clone)]
pub struct CreateRealtimeEventCommand {
    pub payload: EventPayload,
    pub target_users: Vec<UserId>,
    pub channel: String,
    pub is_broadcast: bool,
}

impl Command for CreateRealtimeEventCommand {
    type Response = CreateRealtimeEventResponse;
}

/// Response after creating realtime event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRealtimeEventResponse {
    pub event_id: EventId,
    pub channel: String,
    pub target_user_count: usize,
    pub created_at: DateTime<Utc>,
}
