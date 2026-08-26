// Database Infrastructure Module
// sqlx-based connection pool management (BIG-BANG migration canonical)

pub mod diesel_connection_manager;

// Re-export sqlx pool types from epsx-database-pools (canonical)
pub use diesel_connection_manager::{
    AllPoolsHealth, DieselConnectionManager, DieselServerlessConfig,
};

// Backward-compatible shim re-exports (BIG-BANG migration).
// These names are kept for one minor release to avoid breaking callers.
// They return sqlx::PgPool (the canonical TlsPool).
pub use diesel_connection_manager::DieselConnectionManager as _DieselCompat;
pub use epsx_database_pools::PoolExt;

// Re-export async pool-getter functions for callers that previously imported them from `infrastructure::database`.
pub use diesel_connection_manager::{
    diesel_health_check, diesel_health_check_all, get_analytics_pool, get_diesel_pool,
    get_notifications_pool, get_payments_pool,
};
