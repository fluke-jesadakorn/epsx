// CQRS Infrastructure
// Application-level event publishing with transactional outbox pattern

pub mod event_store;
pub mod outbox;
pub mod event_dispatcher;
pub mod projection;
pub mod projections;

pub use event_store::{EventStore, PostgresEventStore, EventStoreConfig};
pub use outbox::TransactionalOutbox;
pub use event_dispatcher::{EventDispatcher, EventDispatcherConfig, DispatcherStats, DispatcherHealth};
pub use projection::{Projection, ProjectionEvent, ProjectionCheckpoint, ProjectionManager};
pub use projections::WalletReadModelProjection;
