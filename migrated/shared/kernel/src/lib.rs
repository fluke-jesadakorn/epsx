//! Shared kernel — cross-cutting types, traits, and errors for the EPSX platform.
//!
//! This crate is the **only** place where types and traits that are shared
//! across multiple services may live. Concrete domain objects (e.g. Payment,
//! Subscription) live in their respective service crates.

pub mod error;
pub mod result;
pub mod value_object;
pub mod aggregate_root;
pub mod domain_event;
pub mod event_bus;
pub mod specification;
pub mod entities;
pub mod value_objects;
pub mod constants;
pub mod permissions;

pub use error::{AppError, AppResult, AsyncResult, ApiResult, ApplicationResult, InfrastructureResult, EmptyResult, ErrorKind, ErrorContext, ErrorSeverity};
pub use result::Result;
pub use value_object::{ValueObject, ValueObjectError};
pub use aggregate_root::{AggregateRoot, AggregateBase, Identity, new_id};
pub use domain_event::{DomainEvent, DomainEventBus, EventMetadata, InMemoryEventBus};
pub use event_bus::InMemoryEventBus as InMemoryBus;
pub use specification::{Specification, AndSpecification, OrSpecification, NotSpecification};
pub use value_objects::*;
pub use entities::*;
pub use constants::*;
pub use permissions::*;
