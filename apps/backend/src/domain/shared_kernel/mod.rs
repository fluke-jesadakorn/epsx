// Shared Kernel - Common domain patterns and abstractions
// These are shared across all bounded contexts.
//
// kernel extraction wave9: every member of this module is now a thin
// re-export shim that pulls from the new `epsx-contracts` workspace
// crate. The `services` subdirectory was deleted (eps_ranking_service
// moved to `domain::market_analytics::services` as part of R5) and the
// 3-LOC `event_bus` stub was deleted (the real bus lives in
// `infrastructure/cqrs`).

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
