// kernel extraction wave9 — re-exports the trait files. The path
// `epsx_contracts::traits::aggregate_root` etc. mirrors what the
// `shared_kernel::mod` previously did inline.
//
// wave11(track-c) — `domain_event` was lifted to the crate root at
// `epsx_contracts::domain_event` (R7). The `traits::domain_event` path
// remains as a re-export shim so that pre-wave-11 importers compile
// unchanged during the migration window. Remove in wave-12.
pub mod aggregate_root;
pub mod specification;

// Re-export from the new top-level location. The `pub mod domain_event;`
// line in `lib.rs` is now the canonical home; this re-export keeps the
// `epsx_contracts::traits::domain_event` path working.
pub use crate::domain_event::{DomainEvent, DomainEventBus, EventMetadata, InMemoryEventBus};

pub use aggregate_root::{AggregateRoot, AggregateBase, Identity, new_id};
pub use specification::Specification;
