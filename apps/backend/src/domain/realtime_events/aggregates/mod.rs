// Real-time Events Aggregates

pub mod realtime_event;

pub use realtime_event::{RealtimeEvent, EventPriority, EventStatus, RealtimeEventError};

// Placeholder for other aggregates
pub struct EventBroadcaster;
pub struct ConnectionManager;
pub struct EventSubscription;
pub struct EventHistory;