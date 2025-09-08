// Database abstraction layer for multi-database support

use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use async_trait::async_trait;

use crate::core::errors::*;

/// Abstract database connection trait
#[async_trait]
pub trait DatabaseConnection: Send + Sync + Debug {
    type Transaction: DatabaseTransaction;
    type QueryBuilder: QueryBuilder;
    
    /// Begin a new database transaction
    async fn begin_transaction(&self) -> AppResult<Self::Transaction>;
    
    /// Execute a query without transaction
    async fn execute<T>(&self, query: Self::QueryBuilder) -> AppResult<Vec<T>>
    where T: DeserializeOwned + Send;
    
    /// Insert a single document/record
    async fn insert<T>(&self, collection: &str, document: &T) -> AppResult<String>
    where T: Serialize + Send + Sync;
    
    /// Update a document/record by ID
    async fn update<T>(&self, collection: &str, id: &str, document: &T) -> AppResult<()>
    where T: Serialize + Send + Sync;
    
    /// Delete a document/record by ID
    async fn delete(&self, collection: &str, id: &str) -> AppResult<()>;
    
    /// Get a document/record by ID
    async fn get_by_id<T>(&self, collection: &str, id: &str) -> AppResult<Option<T>>
    where T: DeserializeOwned + Send;
    
    /// Health check for the database connection
    async fn health_check(&self) -> AppResult<DatabaseHealth>;
    
    /// Get connection statistics
    fn connection_info(&self) -> ConnectionInfo;
}

/// Database transaction trait
#[async_trait]
pub trait DatabaseTransaction: Send + Sync + Debug {
    /// Commit the transaction
    async fn commit(self) -> AppResult<()>;
    
    /// Rollback the transaction
    async fn rollback(self) -> AppResult<()>;
    
    /// Execute a query within the transaction (simplified for trait objects)
    async fn execute_query(&self, query: &str) -> AppResult<serde_json::Value>;
    
    /// Insert within the transaction
    async fn insert<T>(&self, collection: &str, document: &T) -> AppResult<String>
    where T: Serialize + Send + Sync;
    
    /// Update within the transaction
    async fn update<T>(&self, collection: &str, id: &str, document: &T) -> AppResult<()>
    where T: Serialize + Send + Sync;
    
    /// Delete within the transaction
    async fn delete(&self, collection: &str, id: &str) -> AppResult<()>;
    
    /// Check if transaction is still active
    fn is_active(&self) -> bool;
}

/// Query builder abstraction
pub trait QueryBuilder: Send + Sync + Debug {
    type Filter: QueryFilter;
    type Sort: QuerySort;
    
    /// Set the collection/table to query
    fn collection(self, name: &str) -> Self;
    
    /// Add a filter condition
    fn filter(self, filter: Self::Filter) -> Self;
    
    /// Add sorting
    fn sort(self, sort: Self::Sort) -> Self;
    
    /// Limit the results
    fn limit(self, limit: u32) -> Self;
    
    /// Skip results (offset)
    fn offset(self, offset: u32) -> Self;
}

/// Query filter trait
pub trait QueryFilter: Send + Sync + Debug {
    /// Equality filter
    fn eq(field: &str, value: serde_json::Value) -> Self;
    
    /// Not equal filter
    fn ne(field: &str, value: serde_json::Value) -> Self;
    
    /// Greater than filter
    fn gt(field: &str, value: serde_json::Value) -> Self;
    
    /// Less than filter
    fn lt(field: &str, value: serde_json::Value) -> Self;
    
    /// In array filter
    fn in_array(field: &str, values: Vec<serde_json::Value>) -> Self;
    
    /// Combine filters with AND
    fn and(self, other: Self) -> Self;
    
    /// Combine filters with OR
    fn or(self, other: Self) -> Self;
}

/// Query sort trait
pub trait QuerySort: Send + Sync + Debug {
    /// Ascending sort
    fn asc(field: &str) -> Self;
    
    /// Descending sort
    fn desc(field: &str) -> Self;
}

/// Database health information
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub last_check: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Connection information
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub database_type: String,
    pub host: String,
    pub database_name: String,
    pub connection_pool_size: Option<u32>,
    pub active_connections: Option<u32>,
}