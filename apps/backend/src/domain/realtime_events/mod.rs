// Real-time Events Bounded Context
// Handles event broadcasting, connection management, and real-time communication
// for the EPSX platform

pub mod aggregates;
pub mod events;
pub mod repository_ports;
pub mod value_objects;

// Public exports from value objects
pub use value_objects::{
    ConnectionId, ConnectionInfo, ConnectionStatus, ConnectionType, EventChannel, EventId,
    EventMetadata, EventPayload, EventType, NotificationLevel, SubscriptionTopic,
    UserId as RealtimeUserId,
};

// Public exports from aggregates
pub use aggregates::{
    ConnectionManager, EventBroadcaster, EventHistory, EventPriority, EventStatus,
    EventSubscription, RealtimeEvent, RealtimeEventError,
};

// Public exports from events
pub use events::{
    ConnectionClosed, ConnectionEstablished, EventDelivered, EventFailed, EventPublished,
    SubscriptionCreated,
};

// Public exports from repository ports
pub use repository_ports::{ConnectionRepositoryPort, EventRepositoryPort};

/// Real-time Events bounded context business rules and invariants
pub struct RealtimeEventsBoundedContext;

impl RealtimeEventsBoundedContext {
    /// Event management business rules
    pub const MAX_EVENT_PAYLOAD_SIZE_KB: u32 = 64;
    pub const MAX_CONNECTIONS_PER_USER: u32 = 5;
    pub const EVENT_RETENTION_DAYS: u32 = 7;
    pub const MAX_SUBSCRIPTION_TOPICS_PER_CONNECTION: u32 = 20;

    /// Event delivery policies
    pub const EVENT_DELIVERY_TIMEOUT_SECONDS: u64 = 30;
    pub const MAX_RETRY_ATTEMPTS: u32 = 3;
    pub const BROADCAST_EVENT_TTL_MINUTES: u64 = 60;

    /// Connection management
    pub const CONNECTION_HEARTBEAT_INTERVAL_SECONDS: u64 = 30;
    pub const IDLE_CONNECTION_TIMEOUT_MINUTES: u64 = 15;
    pub const CONNECTION_CLEANUP_INTERVAL_MINUTES: u64 = 5;

    /// Event channels
    pub const SYSTEM_EVENTS_CHANNEL: &'static str = "system";
    pub const PAYMENT_EVENTS_CHANNEL: &'static str = "payments";
    pub const TRADING_EVENTS_CHANNEL: &'static str = "trading";
    pub const NOTIFICATION_EVENTS_CHANNEL: &'static str = "notifications";
    pub const HEALTH_EVENTS_CHANNEL: &'static str = "health";
}
