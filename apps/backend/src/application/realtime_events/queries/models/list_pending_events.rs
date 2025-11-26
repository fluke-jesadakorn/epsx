use crate::prelude::*;
use crate::application::shared::Query;

/// Query to list pending events ready for delivery
#[derive(Debug, Clone)]
pub struct ListPendingEventsQuery {
    pub limit: u32,
}

impl Query for ListPendingEventsQuery {
    type Response = ListPendingEventsResponse;
}

/// Response containing list of pending events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPendingEventsResponse {
    pub events: Vec<PendingEventSummary>,
    pub count: usize,
}

/// Summary of a pending event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingEventSummary {
    pub event_id: String,
    pub channel: String,
    pub target_user_count: usize,
    pub priority: String,
    pub delivery_attempts: u32,
    pub created_at: DateTime<Utc>,
}
