//! `epsx-database-pools` — shared Postgres connection pool types.
//!
//! BIG-BANG: After migration, `TlsPool` is a type alias for the canonical
//! `sqlx::PgPool`. The historical `TlsConnectionManager` / `ManagerError` /
//! `deadpool::managed::Pool` / `PoolExt` types are retained as deprecated
//! shims so that downstream code can still resolve them — they forward to
//! the canonical sqlx pool via constructors in `apps/backend/src/main.rs`.
//!
//! What lives here:
//!   - `TlsPool` — `pub type TlsPool = sqlx::PgPool;` (canonical).
//!   - `sqlx_pool` — canonical 4-pool creation (`core/payments/analytics/notifications`).
//!
//! Deprecated legacy shims (kept for one release):
//!   - `TlsConnectionManager`, `ManagerError`, `PoolExt` (deadpool).

pub mod sqlx_pool;

/// Canonical Postgres connection pool (BIG-BANG: alias for `sqlx::PgPool`).
pub type TlsPool = sqlx::PgPool;

/// Re-export the sqlx pool creation helpers as the canonical pool API.
use async_trait::async_trait;

pub use sqlx_pool::{create_all_pools, create_pool, health_check, SqlxPoolConfig};

// ---------------------------------------------------------------------------
// DEPRECATED shims (retained for one release) — deadpool/diesel removed.
// ---------------------------------------------------------------------------
// `TlsConnectionManager` is now a stub; new code must use `sqlx::PgPool`
// (`TlsPool`) directly. The `Manager` trait impl was removed with `deadpool`.

/// Deprecated: use `sqlx::PgPool` (alias `TlsPool`) instead.
#[deprecated(note = "Diesel migration complete — use sqlx::PgPool (TlsPool) directly")]
#[derive(Clone)]
pub struct TlsConnectionManager {
    #[allow(dead_code)]
    database_url: String,
}

#[allow(deprecated)]
impl TlsConnectionManager {
    #[deprecated(note = "Diesel migration complete — use sqlx::PgPool directly")]
    #[allow(dead_code)]
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }
}

/// Deprecated: use `AppError::database_error` directly with sqlx errors.
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("Database connection error: {0}")]
    Connection(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Deprecated: use `sqlx::PgPool` directly (no `.conn()` extension needed).
#[async_trait]
pub trait PoolExt {
    /// Deprecated: use `sqlx::PgPool::acquire()` directly.
    async fn conn(&self) -> epsx_contracts::errors::AppResult<sqlx::PgPool>;
}

#[async_trait]
impl PoolExt for TlsPool {
    async fn conn(&self) -> epsx_contracts::errors::AppResult<sqlx::PgPool> {
        Ok(self.clone())
    }
}
