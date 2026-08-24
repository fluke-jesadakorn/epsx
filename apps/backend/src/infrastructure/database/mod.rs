// Database Infrastructure Module
// sqlx-based connection pool management

pub mod diesel_connection_manager;

// Re-export sqlx pool types from epsx-database-pools (canonical)
pub use diesel_connection_manager::{
    AllPoolsHealth, DieselConnectionManager, DieselServerlessConfig,
};

// Backward-compatible shim re-exports (BIG-BANG migration).
// These names are kept for one minor release to avoid breaking callers.
pub use diesel_connection_manager::DieselConnectionManager as _DieselCompat;
pub use epsx_database_pools::PoolExt;
