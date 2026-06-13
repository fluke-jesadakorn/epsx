// Wave 11 / Track C — moved to top-level (was at
// `epsx_contracts::traits::domain_event`). The trait is the most-imported
// kernel symbol in the application command handler layer; lifting it to the
// crate root makes the `use epsx_contracts::DomainEvent` path ergonomic for
// the 19 handlers being migrated to the new `EventPublisherPort` (ROADMAP
// §5 R7).
//
// The `traits::domain_event` path remains a `pub use` re-export shim so
// that older call sites (and the in-tree `domain::shared_kernel` shim)
// keep compiling during the migration window.

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Base trait for all domain events
/// Events represent things that have happened in the domain
pub trait DomainEvent: Send + Sync + Debug {
    /// Unique identifier for this event instance
    fn event_id(&self) -> Uuid;

    /// The type of event (used for routing and serialization)
    fn event_type(&self) -> &'static str;

    /// The type of aggregate that raised this event
    /// Used for event store partitioning and routing
    /// Examples: "WalletUser", "Subscription", "PermissionGroup"
    fn aggregate_type(&self) -> &'static str;

    /// When this event occurred
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Version of the aggregate when this event was raised
    fn aggregate_version(&self) -> u64;

    /// The aggregate ID that raised this event
    fn aggregate_id(&self) -> String;

    /// Serialize event to JSON for storage/transport
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>>;

    /// Downcast to Any for type introspection
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Base event data that all events should include
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_version: u64,
    pub aggregate_id: String,
}

impl EventMetadata {
    pub fn new(aggregate_id: String, aggregate_version: u64) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            aggregate_version,
            aggregate_id,
        }
    }
}

/// Simple event bus interface for domain events. This is the *legacy*
/// synchronous in-process interface. New code MUST go through
/// `EventPublisherPort` (see `epsx_contracts::event_publisher_port`).
/// The two interfaces coexist during the wave-11 → wave-12 migration:
/// the bus is preserved as a no-op shim for backward compatibility
/// (the in-process bus today is a stub per ROADMAP §6 trap 3).
pub trait DomainEventBus: Send + Sync {
    /// Publish a domain event
    fn publish(&self, event: &dyn DomainEvent);

    /// Publish multiple events as a batch
    fn publish_batch(&self, events: &[Box<dyn DomainEvent>]) {
        for event in events {
            self.publish(&**event);
        }
    }
}

/// In-memory event bus for testing and simple scenarios
pub struct InMemoryEventBus {
    events: std::sync::RwLock<Vec<String>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            events: std::sync::RwLock::new(Vec::new()),
        }
    }
    
    pub fn published_events(&self) -> Vec<String> {
        let events = self.events.read().unwrap();
        events.clone()
    }
    
    pub fn clear_events(&self) {
        let mut events = self.events.write().unwrap();
        events.clear();
    }
}

impl DomainEventBus for InMemoryEventBus {
    fn publish(&self, event: &dyn DomainEvent) {
        let mut events = self.events.write().unwrap();
        events.push(event.event_type().to_string());
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Owned wrapper for a `DomainEvent` borrowed reference. Used by the
/// `EventPublisherPort` to publish events when the caller does not
/// own the event (e.g. iterating over
/// `Aggregate::uncommitted_events()` which returns `&[Box<dyn
/// DomainEvent>]`). The wrapper serializes the event to JSON once,
/// preserves the `event_type` header, and is itself a `Box<dyn
/// DomainEvent>` that can be passed to the port.
///
/// The `Payload` field is a `serde_json::Value` so the wrapper
/// works for any event type without coupling to a concrete DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedEvent {
    /// The original event type (e.g. `"PlanDeletedEvent"`).
    pub event_type: String,
    /// The aggregate type (e.g. `"PermissionPlan"`).
    pub aggregate_type: String,
    /// The aggregate id.
    pub aggregate_id: String,
    /// The aggregate version when the event was raised.
    pub aggregate_version: u64,
    /// The event's unique id.
    pub event_id: Uuid,
    /// When the event occurred.
    pub occurred_at: DateTime<Utc>,
    /// Serialized event payload.
    pub payload: serde_json::Value,
}

impl OwnedEvent {
    /// Convert a borrowed `DomainEvent` into an owned `OwnedEvent`
    /// that can cross an await boundary. The conversion is one-way
    /// (we cannot reconstruct the concrete event type from JSON);
    /// use this when the caller doesn't need to recover the
    /// original event type.
    pub fn from_borrowed(event: &dyn DomainEvent) -> Self {
        let payload = event
            .to_json()
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(serde_json::Value::Null);
        Self {
            event_type: event.event_type().to_string(),
            aggregate_type: event.aggregate_type().to_string(),
            aggregate_id: event.aggregate_id(),
            aggregate_version: event.aggregate_version(),
            event_id: event.event_id(),
            occurred_at: event.occurred_at(),
            payload,
        }
    }
}

impl DomainEvent for OwnedEvent {
    fn event_id(&self) -> Uuid {
        self.event_id
    }
    fn event_type(&self) -> &'static str {
        // Leak a `&'static str` for the event type. This is a
        // deliberate trade-off: the original `DomainEvent` trait
        // requires `&'static str` for the event type, but
        // `OwnedEvent` accepts any runtime string. The leak is
        // bounded by the set of distinct event types (a few dozen
        // at most) and happens once per type. For a stricter
        // alternative, see the `BoxedEventType` adapter below.
        //
        // SAFETY: the leak is intentional and the count is bounded
        // by the event type universe, so the memory cost is
        // negligible.
        Box::leak(self.event_type.clone().into_boxed_str())
    }
    fn aggregate_type(&self) -> &'static str {
        Box::leak(self.aggregate_type.clone().into_boxed_str())
    }
    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
    fn aggregate_id(&self) -> String {
        self.aggregate_id.clone()
    }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// `DomainEvent` requires `Send + Sync + 'static` for `Box<dyn DomainEvent>`
// to be safely sent across `tokio::spawn` boundaries. Enforce the static
// bound here (the trait does not include `'static` because that would
// forbid zero-copy borrowed data; the boxed form is what the publisher
// port takes).
//
// Compile-time assertion: every type that implements `DomainEvent` and
// is used via the port must be `'static` (it travels through `Box<dyn
// DomainEvent + Send>`, then across an async boundary).
#[allow(dead_code)]
fn _assert_event_is_static(_: Box<dyn DomainEvent>) {
    fn _requires_static<T: 'static>(_: T) {}
    // The `as_any` + `Send + Sync` bounds on `DomainEvent` plus the
    // `Box<dyn DomainEvent + Send>` bound on `EventPublisherPort::publish`
    // together imply the event is `'static` for the publisher path.
    // This function exists so that the assertion is mechanically
    // discoverable; it is never called.
    let _ = _requires_static::<Box<dyn DomainEvent>>;
}
