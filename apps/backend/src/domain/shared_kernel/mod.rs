// Shared Kernel - Common domain patterns and abstractions
// These are shared across all bounded contexts.
//
// kernel extraction wave9: every member of this module is now a thin
// re-export shim that pulls from the new `epsx-contracts` workspace
// crate. The `services` subdirectory was deleted (eps_ranking_service
// moved to `domain::market_analytics::services` as part of R5) and the
// 3-LOC `event_bus` stub was deleted (the real bus lives in
// `infrastructure/cqrs`).
//
// wave11(track-c): `domain_event` was lifted to the crate root at
// `epsx_contracts::domain_event` (R7). The `pub mod domain_event;` shim
// here is no longer needed — callers that imported the trait via
// `crate::domain::shared_kernel::domain_event::*` were already updated
// to use the `epsx_contracts::traits::domain_event` path in wave 9.
// The shim file is left in place for now as a 2-line re-export; remove
// in wave 12 once the migration is complete.

pub mod aggregate_root;
pub mod domain_event;
pub mod specification;
pub mod value_object;
pub mod value_objects;
pub mod entities;
pub mod ports;
pub mod app_error;

pub use aggregate_root::{ AggregateRoot, AggregateBase, Identity, new_id };
pub use domain_event::{ DomainEvent, DomainEventBus, EventMetadata };
pub use specification::Specification;
pub use value_object::ValueObject;
pub use value_objects::*;
pub use entities::*;
pub use ports::*;
pub use app_error::{AppError, AppResult};
