//! Common prelude module for reducing import boilerplate
//!
//! This module re-exports commonly used types and traits across the codebase.
//! Import with: `use crate::prelude::*;`

// ===== Core std types =====
pub use std::error::Error as StdError;
pub use std::fmt::{Debug, Display};
pub use std::sync::Arc;

// ===== Async runtime =====
pub use async_trait::async_trait;

// ===== Serialization =====
pub use serde::{Deserialize, Serialize};

// ===== Date/Time =====
pub use chrono::{DateTime, Utc};

// ===== Core error handling =====
pub use epsx_contracts::errors::{AppError, AppResult};

// ===== Domain-Driven Design core traits =====
pub use crate::domain::shared_kernel::{AggregateRoot, DomainEvent, DomainEventBus, ValueObject};

// ===== Common value objects =====
pub use epsx_contracts::value_objects::{Email, SessionId, UserId};
// ===== Database =====
pub use crate::infrastructure::database::diesel_connection_manager::TlsPool;
pub use epsx_database_pools::PoolExt;
#[allow(deprecated)]
pub use epsx_database_pools::TlsConnectionManager;
