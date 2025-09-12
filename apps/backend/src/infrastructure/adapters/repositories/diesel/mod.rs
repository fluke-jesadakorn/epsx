// Diesel ORM infrastructure adapter
// This module provides database connectivity and ORM functionality

pub mod models;
pub mod pool;
pub mod types;
pub mod mappers;
pub mod repos;
pub mod schema;
pub mod marketing_repository;

// Re-export commonly used types
pub use pool::{DbPool, create_pool, create_test_pool};
pub use types::*;
pub use models::*;

// Create database pool function for backward compatibility
pub async fn create_diesel_pool(database_url: &str) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    create_pool(database_url).await
}