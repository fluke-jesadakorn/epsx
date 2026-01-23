//! Common prelude module for reducing import boilerplate
//! 
//! This module re-exports commonly used types and traits across the codebase.
//! Import with: `use crate::prelude::*;`

// ===== Core std types =====
pub use std::sync::Arc;
pub use std::fmt::{Debug, Display};
pub use std::error::Error as StdError;

// ===== Async runtime =====
pub use async_trait::async_trait;

// ===== Serialization =====
pub use serde::{Serialize, Deserialize};

// ===== Date/Time =====
pub use chrono::{DateTime, Utc};

// ===== Core error handling =====
pub use crate::core::errors::{AppResult, AppError};

// ===== Domain-Driven Design core traits =====
pub use crate::domain::shared_kernel::{
    AggregateRoot,
    ValueObject,
    DomainEvent,
    DomainEventBus,
};

// ===== Common value objects =====
pub use crate::domain::shared_kernel::value_objects::{
    UserId,
    SessionId,
    Email,
};
// ===== Database =====
pub use crate::infrastructure::database::diesel_connection_manager::{TlsPool, TlsConnectionManager};
