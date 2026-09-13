// CQRS Infrastructure
// Application-level event publishing with transactional outbox pattern

pub mod event_dispatcher;
pub mod event_store;
pub mod outbox;
pub mod projection;
pub mod projections;

pub use event_dispatcher::{
    DispatcherHealth, DispatcherStats, EventDispatcher, EventDispatcherConfig,
};
pub use event_store::{EventStore, EventStoreConfig, PostgresEventStore};
pub use outbox::TransactionalOutbox;
pub use projection::{Projection, ProjectionCheckpoint, ProjectionEvent, ProjectionManager};
pub use projections::WalletReadModelProjection;
