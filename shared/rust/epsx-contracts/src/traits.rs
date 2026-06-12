// kernel extraction wave9 — re-exports the trait files. The path
// `epsx_contracts::traits::aggregate_root` etc. mirrors what the
// `shared_kernel::mod` previously did inline.
pub mod aggregate_root;
pub mod domain_event;
pub mod specification;

pub use aggregate_root::{AggregateRoot, AggregateBase, Identity, new_id};
pub use domain_event::{DomainEvent, DomainEventBus, EventMetadata, InMemoryEventBus};
pub use specification::Specification;
