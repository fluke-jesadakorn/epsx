// Database Infrastructure Module
// Diesel async connection manager for serverless-optimized database access

pub mod diesel_connection_manager;

// Re-export Diesel types
pub use diesel_connection_manager::{
    DieselConnectionManager,
    DieselServerlessConfig,
    DieselPoolStats,
    PoolExt,
    get_diesel_pool,
    get_analytics_pool,
    get_notifications_pool,
    get_payments_pool,
    diesel_health_check,
    diesel_health_check_all,
    AllPoolsHealth,
};