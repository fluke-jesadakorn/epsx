//! Common prelude for `epsx-identity-shared`.
//!
//! Mirrors the `apps/backend::prelude` shape used by the source
//! files moved from `apps/backend/src/auth/`. The `TlsPool` alias is
//! a self-contained type alias backed by `deadpool::managed::Pool`
//! so the moved auth code can keep `&'static TlsPool` parameters
//! without taking a hard dep on the backend's
//! `diesel_connection_manager`.

pub use std::sync::Arc;
pub use std::fmt::{Debug, Display};
pub use std::error::Error as StdError;

pub use async_trait::async_trait;
pub use serde::{Serialize, Deserialize};
pub use chrono::{DateTime, Utc};

pub use crate::core::AppError;
pub use crate::connection::AsyncPgConnectionManager;

/// Async Postgres connection pool type used by the moved auth code.
///
/// Thin alias over `deadpool::managed::Pool<AsyncPgConnectionManager>`.
/// The backend's real `TlsPool` (which enforces TLS via `rustls`) is
/// a separate, unrelated type; this alias exists only so the moved
/// auth code compiles standalone.
pub type TlsPool = deadpool::managed::Pool<AsyncPgConnectionManager>;
