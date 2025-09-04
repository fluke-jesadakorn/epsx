// Shared Kernel - Common domain patterns and abstractions
// These are shared across all bounded contexts

pub mod aggregate_root;
pub mod domain_event;
pub mod specification;
pub mod value_object;
pub mod domain_error;

pub use aggregate_root::{AggregateRoot, Identity, new_id};
pub use domain_event::{DomainEvent, DomainEventBus};
pub use specification::Specification;
pub use value_object::ValueObject;
pub use domain_error::{DomainError, DomainResult};