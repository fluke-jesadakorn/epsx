use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::shared_kernel::DomainError;
use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::user_management::{
    aggregates::User,
    value_objects::{Email, Permission}
};

/// Repository port for User aggregate persistence
/// This interface defines the contract for User data access without specifying implementation
#[async_trait]
pub trait UserRepositoryPort: Send + Sync {
    /// Find a user by their unique identifier
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError>;
    
    /// Find a user by their email address
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
    
    
    /// Save a user (create or update)
    async fn save(&self, user: &User) -> Result<(), DomainError>;
    
    /// Delete a user
    async fn delete(&self, id: &UserId) -> Result<(), DomainError>;
    
    /// Find users with specific permissions
    async fn find_by_permission(&self, permission: &Permission) -> Result<Vec<User>, DomainError>;
    
    /// Find users by multiple criteria with pagination
    async fn find_by_criteria(
        &self, 
        criteria: &UserSearchCriteria,
        limit: u32,
        offset: u32
    ) -> Result<UserSearchResult, DomainError>;
    
    /// Count users matching criteria
    async fn count_by_criteria(&self, criteria: &UserSearchCriteria) -> Result<u64, DomainError>;
    
    /// Find users eligible for automatic permission assignment
    async fn find_eligible_for_auto_assignment(&self) -> Result<Vec<User>, DomainError>;
    
    /// Batch operations for efficiency
    async fn save_batch(&self, users: &[User]) -> Result<(), DomainError>;
    
    /// Get the next available identity
    async fn next_identity(&self) -> Result<UserId, DomainError>;
    
    /// Health check for the repository
    async fn health_check(&self) -> Result<(), DomainError>;
    
    /// Clean up expired permissions across all users
    async fn cleanup_expired_permissions(&self) -> Result<u32, DomainError>;
}

/// Search criteria for finding users
#[derive(Debug, Clone, Default)]
pub struct UserSearchCriteria {
    /// Text search across email and other fields
    pub search_term: Option<String>,
    
    /// Filter by email pattern
    pub email_pattern: Option<String>,
    
    /// Filter by active status
    pub is_active: Option<bool>,
    
    /// Filter by email verification status
    pub email_verified: Option<bool>,
    
    /// Filter by users who have specific permissions
    pub has_permissions: Vec<Permission>,
    
    /// Filter by users created after this date
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Filter by users created before this date
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Filter by users who logged in after this date
    pub last_login_after: Option<chrono::DateTime<chrono::Utc>>,
    
    
    /// Custom filters for extensibility
    pub custom_filters: HashMap<String, String>,
}

/// Result of a user search operation
#[derive(Debug, Clone)]
pub struct UserSearchResult {
    /// The users that matched the search criteria
    pub users: Vec<User>,
    
    /// Total count of users that match (for pagination)
    pub total_count: u64,
    
    /// The offset used in this search
    pub offset: u32,
    
    /// The limit used in this search
    pub limit: u32,
    
    /// Whether there are more results available
    pub has_more: bool,
}

impl UserSearchResult {
    pub fn new(users: Vec<User>, total_count: u64, offset: u32, limit: u32) -> Self {
        let has_more = (offset + limit) < total_count as u32;
        
        Self {
            users,
            total_count,
            offset,
            limit,
            has_more,
        }
    }
}

/// Aggregated statistics about users
#[derive(Debug, Clone)]
pub struct UserStatistics {
    pub total_users: u64,
    pub active_users: u64,
    pub verified_users: u64,
    pub users_with_permissions: u64,
    pub recent_logins_24h: u64,
    pub recent_registrations_24h: u64,
}

/// Extended port with analytics capabilities
#[async_trait]
pub trait UserAnalyticsPort: Send + Sync {
    /// Get user statistics
    async fn get_statistics(&self) -> Result<UserStatistics, DomainError>;
    
    /// Get permission distribution across users
    async fn get_permission_distribution(&self) -> Result<HashMap<String, u64>, DomainError>;
    
    /// Get user activity patterns
    async fn get_activity_patterns(&self, days: u32) -> Result<Vec<(chrono::NaiveDate, u64)>, DomainError>;
    
    /// Find inactive users (haven't logged in for specified days)
    async fn find_inactive_users(&self, days: u32) -> Result<Vec<User>, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn user_search_result_has_more_calculation() {
        let users = vec![];
        let result = UserSearchResult::new(users, 100, 0, 10);
        assert!(result.has_more);
        
        let result = UserSearchResult::new(vec![], 5, 0, 10);
        assert!(!result.has_more);
        
        let result = UserSearchResult::new(vec![], 100, 95, 10);
        assert!(!result.has_more);
    }
}