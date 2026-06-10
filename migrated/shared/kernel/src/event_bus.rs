//! Re-export of the in-memory event bus from domain_event.
//! Real implementations (Redis, NATS) live in shared/events.

pub use crate::domain_event::InMemoryEventBus;
