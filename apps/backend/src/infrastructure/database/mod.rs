// Database Infrastructure Module
// Diesel async connection manager for serverless-optimized database access

pub mod diesel_connection_manager;

// Re-export Diesel types
pub use diesel_connection_manager::{
    DieselConnectionManager,
    DieselServerlessConfig,
    DieselPoolStats,
    get_diesel_pool,
    diesel_health_check,
};