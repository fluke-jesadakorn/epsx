use crate::prelude::*;
use crate::application::shared::Command;
use crate::domain::realtime_events::value_objects::EventId;

/// Command to mark event as delivered
#[derive(Debug, Clone)]
pub struct MarkEventDeliveredCommand {
    pub event_id: EventId,
}

impl Command for MarkEventDeliveredCommand {
    type Response = MarkEventDeliveredResponse;
}

/// Response after marking event delivered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkEventDeliveredResponse {
    pub event_id: EventId,
    pub delivered_at: DateTime<Utc>,
    pub delivery_attempts: u32,
}
