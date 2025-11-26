use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::application::shared::{
    Query, 
    ApplicationResult, 
    PaginationParams, 
    SortParams, 
    ValidationUtils
};
use crate::domain::shared_kernel::value_objects::UserId;
// use crate::domain::wallet_management::value_objects::Email; // REMOVED - Web3-first uses wallet addresses

/// Query to search for users with various criteria (Web3-first: wallet-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWalletsQuery {
    /// Text search across wallet address and other fields
    pub search_term: Option<String>,

    /// Filter by wallet address pattern
    pub wallet_pattern: Option<String>,

    /// Filter by active status
    pub is_active: Option<bool>,

    /// Filter by users who have specific permissions
    pub has_permissions: Vec<String>,
    
    /// Filter by users created after this date
    pub created_after: Option<DateTime<Utc>>,
    
    /// Filter by users created before this date
    pub created_before: Option<DateTime<Utc>>,
    
    /// Filter by users who logged in after this date
    pub last_login_after: Option<DateTime<Utc>>,
    
    /// Pagination parameters
    pub pagination: PaginationParams,
    
    /// Sorting parameters
    pub sort: Option<SortParams>,
    
    /// Whether to include user statistics
    pub include_stats: bool,
    
    /// Query metadata
    pub requested_by: Option<String>,
    pub correlation_id: Option<String>,
}

/// Search results response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWalletsResponse {
    /// Users that matched the search criteria
    pub users: Vec<WalletSummary>,
    
    /// Pagination information
    pub pagination: PaginationResult,
    
    /// Search metadata
    pub search_metadata: SearchMetadata,
}

/// Summary information about a user in search results (Web3-first: wallet-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSummary {
    pub wallet_address: UserId,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub permission_count: u32,
    pub active_session_count: u32,
}

/// Pagination result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResult {
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub total_count: u64,
    pub has_next: bool,
    pub has_previous: bool,
}

/// Search execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub execution_time_ms: u64,
    pub filters_applied: Vec<String>,
    pub sort_applied: Option<String>,
    pub cache_hit: bool,
}

impl Query for SearchWalletsQuery {
    type Response = SearchWalletsResponse;
    
    fn validate(&self) -> ApplicationResult<()> {
        // Validate pagination parameters
        self.pagination.validate()?;
        
        // Validate date ranges
        if let (Some(created_after), Some(created_before)) = (self.created_after, self.created_before) {
            if created_after >= created_before {
                return Err(crate::application::ApplicationError::validation(
                    "date_range",
                    "created_after must be before created_before"
                ));
            }
        }
        
        // Validate search term length if provided
        if let Some(ref search_term) = self.search_term {
            if let Some(error) = ValidationUtils::length("search_term", search_term, 2, 100) {
                return Err(crate::application::ApplicationError::validation(
                    &error.field,
                    &error.message
                ));
            }
        }
        
        // Validate permissions format
        for permission in &self.has_permissions {
            if permission.split(':').count() < 3 {
                return Err(crate::application::ApplicationError::validation(
                    "has_permissions",
                    format!("Invalid permission format: {}", permission)
                ));
            }
        }
        
        Ok(())
    }
}

impl SearchWalletsQuery {
    /// Create a new SearchWalletsQuery with default pagination
    pub fn new() -> Self {
        Self {
            search_term: None,
            wallet_pattern: None,
            is_active: None,
            has_permissions: Vec::new(),
            created_after: None,
            created_before: None,
            last_login_after: None,
            pagination: PaginationParams::default(),
            sort: None,
            include_stats: false,
            requested_by: None,
            correlation_id: None,
        }
    }

    /// Set search term for text search
    pub fn with_search_term(mut self, search_term: String) -> Self {
        self.search_term = Some(search_term);
        self
    }

    /// Filter by wallet address pattern
    pub fn with_wallet_pattern(mut self, wallet_pattern: String) -> Self {
        self.wallet_pattern = Some(wallet_pattern);
        self
    }

    /// Filter by active status
    pub fn with_active_status(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }
    
    /// Filter by users who have specific permissions
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.has_permissions = permissions;
        self
    }
    
    /// Set date range filter
    pub fn with_date_range(
        mut self, 
        created_after: Option<DateTime<Utc>>, 
        created_before: Option<DateTime<Utc>>
    ) -> Self {
        self.created_after = created_after;
        self.created_before = created_before;
        self
    }
    
    /// Set pagination parameters
    pub fn with_pagination(mut self, page: u32, page_size: u32) -> Self {
        self.pagination = PaginationParams::new(page, page_size);
        self
    }
    
    /// Set sorting parameters
    pub fn with_sort(mut self, sort: SortParams) -> Self {
        self.sort = Some(sort);
        self
    }
    
    /// Include statistics in the response
    pub fn with_stats(mut self) -> Self {
        self.include_stats = true;
        self
    }
    
    /// Set who requested this query (for audit)
    pub fn requested_by(mut self, wallet_address: String) -> Self {
        self.requested_by = Some(wallet_address);
        self
    }
    
    /// Set correlation ID for tracing
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

impl Default for SearchWalletsQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl PaginationResult {
    pub fn new(page: u32, page_size: u32, total_count: u64) -> Self {
        let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as u32;
        let has_next = page < total_pages;
        let has_previous = page > 1;
        
        Self {
            page,
            page_size,
            total_pages,
            total_count,
            has_next,
            has_previous,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn search_users_query_validation_success() {
        let query = SearchWalletsQuery::new();
        assert!(query.validate().is_ok());
    }
    
    #[test]
    fn search_users_query_validation_invalid_date_range() {
        let now = Utc::now();
        let future = now + chrono::Duration::hours(1);
        
        let query = SearchWalletsQuery::new()
            .with_date_range(Some(future), Some(now));
        
        assert!(query.validate().is_err());
    }
    
    #[test]
    fn search_users_query_builder_pattern() {
        let query = SearchWalletsQuery::new()
            .with_search_term("test".to_string())
            .with_active_status(true)
            .with_pagination(2, 50)
            .with_sort(SortParams::desc("created_at"))
            .with_stats()
            .requested_by("admin_user".to_string());
        
        assert_eq!(query.search_term, Some("test".to_string()));
        assert_eq!(query.is_active, Some(true));
        assert_eq!(query.pagination.page, 2);
        assert_eq!(query.pagination.page_size, 50);
        assert!(query.sort.is_some());
        assert!(query.include_stats);
        assert_eq!(query.requested_by, Some("admin_user".to_string()));
    }
    
    #[test]
    fn pagination_result_calculation() {
        let result = PaginationResult::new(2, 10, 95);
        
        assert_eq!(result.page, 2);
        assert_eq!(result.total_pages, 10);
        assert!(result.has_next);
        assert!(result.has_previous);
    }
}