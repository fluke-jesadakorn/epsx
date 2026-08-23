//! `sqlx_pool` — big-bang canonical pool (sqlx) for all 4 DBs.
//!
//! This is the BIG-BANG replacement for `diesel_connection_manager.rs`.
//! All 4 pools (core, payments, analytics, notifications) are created here
//! via `sqlx::PgPoolOptions`. The `TlsPool` (deadpool+diesel) is retained
//! side-by-side until the last `diesel` query is migrated, then deleted.
//!
//! Pool sizes mirror `diesel_connection_manager.rs:26`:
//!   core: 10, payments: 10, analytics: 5, notifications: 8

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Per-DB pool sizing. Mirrors the OnceLock statics in diesel_connection_manager.
#[derive(Debug, Clone)]
pub struct SqlxPoolConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
}

impl SqlxPoolConfig {
    pub fn core(url: String) -> Self {
        Self {
            url,
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
        }
    }
    pub fn payments(url: String) -> Self {
        Self {
            url,
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),
        }
    }
    pub fn analytics(url: String) -> Self {
        Self {
            url,
            max_connections: 5,
            min_connections: 0,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
        }
    }
    pub fn notifications(url: String) -> Self {
        Self {
            url,
            max_connections: 8,
            min_connections: 0,
            acquire_timeout: Duration::from_secs(3),
            idle_timeout: Duration::from_secs(300),
        }
    }
}

/// Build a `PgPool` from config. Uses `sqlx::postgres::PgPoolOptions`.
pub async fn create_pool(cfg: SqlxPoolConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .min_connections(cfg.min_connections)
        .acquire_timeout(cfg.acquire_timeout)
        .idle_timeout(cfg.idle_timeout)
        .connect(&cfg.url)
        .await
}

/// Convenience: create all 4 pools. Returns error if any fails.
/// BIG-BANG: caller must provide all 4 URLs; no fallback to primary.
pub async fn create_all_pools(
    core_url: String,
    payments_url: String,
    analytics_url: String,
    notifications_url: String,
) -> Result<(PgPool, PgPool, PgPool, PgPool), sqlx::Error> {
    let core = create_pool(SqlxPoolConfig::core(core_url)).await?;
    let payments = create_pool(SqlxPoolConfig::payments(payments_url)).await?;
    let analytics = create_pool(SqlxPoolConfig::analytics(analytics_url)).await?;
    let notifications = create_pool(SqlxPoolConfig::notifications(notifications_url)).await?;
    Ok((core, payments, analytics, notifications))
}

/// Extension: health check via `SELECT 1`
pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}
