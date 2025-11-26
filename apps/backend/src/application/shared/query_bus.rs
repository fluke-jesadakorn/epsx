use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

use super::{ApplicationError, ApplicationResult};

/// Base trait for all queries (read operations)
/// Queries represent requests for data without side effects
pub trait Query: Send + Sync + Debug + Clone {
    /// The type of response this query produces
    type Response: Send + Sync;
    
    /// Validate the query before execution
    fn validate(&self) -> ApplicationResult<()> {
        Ok(())
    }
}

/// Handler for a specific query type
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    /// Handle the query and return the response
    async fn handle(&self, query: Q) -> ApplicationResult<Q::Response>;
}

// QueryBus trait and InMemoryQueryBus removed - unused abstraction
// Handlers are called directly in application services, no bus dispatch needed

/// Query metadata for performance monitoring and caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub query_id: uuid::Uuid,
    pub query_type: String,
    pub requested_by: Option<String>, // User ID who requested the query
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<String>, // For tracing across services
    pub cacheable: bool,
    pub cache_ttl_seconds: Option<u64>,
}

impl QueryMetadata {
    pub fn new(query_type: impl Into<String>) -> Self {
        Self {
            query_id: uuid::Uuid::new_v4(),
            query_type: query_type.into(),
            requested_by: None,
            requested_at: chrono::Utc::now(),
            correlation_id: None,
            cacheable: false,
            cache_ttl_seconds: None,
        }
    }
    
    pub fn with_user(mut self, wallet_address: impl Into<String>) -> Self {
        self.requested_by = Some(wallet_address.into());
        self
    }
    
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
    
    pub fn cacheable(mut self, ttl_seconds: u64) -> Self {
        self.cacheable = true;
        self.cache_ttl_seconds = Some(ttl_seconds);
        self
    }
}

/// Pagination parameters for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
    pub max_page_size: u32,
}

impl PaginationParams {
    pub fn new(page: u32, page_size: u32) -> Self {
        Self {
            page: page.max(1), // Pages are 1-indexed
            page_size: page_size.clamp(1, 1000), // Reasonable limits
            max_page_size: 1000,
        }
    }
    
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }
    
    pub fn validate(&self) -> ApplicationResult<()> {
        if self.page_size > self.max_page_size {
            return Err(ApplicationError::validation(
                "page_size",
                format!("Page size cannot exceed {}", self.max_page_size)
            ));
        }
        
        if self.page == 0 {
            return Err(ApplicationError::validation(
                "page",
                "Page number must be greater than 0"
            ));
        }
        
        Ok(())
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self::new(1, 20)
    }
}

/// Sorting parameters for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortParams {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortParams {
    pub fn new(field: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            field: field.into(),
            direction,
        }
    }
    
    pub fn asc(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Asc)
    }
    
    pub fn desc(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Desc)
    }
}