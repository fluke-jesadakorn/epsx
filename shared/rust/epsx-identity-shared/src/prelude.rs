//! Common prelude for `epsx-identity-shared`.
//!
//! Mirrors the `apps/backend::prelude` shape used by the source
//! files moved from `apps/backend/src/auth/`. The `TlsPool` alias
//! now re-exports the real TLS-enforcing `Pool<TlsConnectionManager>`
//! from the `epsx-database-pools` shared crate (added in the wave
//! 10 prep pass) so the moved auth code and the backend see the
//! same concrete type.

pub use std::sync::Arc;
pub use std::fmt::{Debug, Display};
pub use std::error::Error as StdError;

pub use async_trait::async_trait;
pub use serde::{Serialize, Deserialize};
pub use chrono::{DateTime, Utc};

pub use crate::core::AppError;

/// Re-export the shared `TlsPool` from `epsx-database-pools` so the
/// moved auth code's `&'static TlsPool` parameters are satisfied
/// with the *same* type the backend passes in.
pub use epsx_database_pools::TlsPool;
