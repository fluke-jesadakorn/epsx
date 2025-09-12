use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};

use crate::domain::shared_kernel::DomainError;
use crate::domain::shared_kernel::value_objects::{UserId, SessionId};
use crate::domain::user_management::aggregates::Session;

/// Repository port for Session aggregate persistence
/// This interface defines the contract for Session data access without specifying implementation
#[async_trait]
pub trait SessionRepositoryPort: Send + Sync {
    /// Find a session by its unique identifier
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<Session>, DomainError>;
    
    /// Find sessions by user ID
    async fn find_byuser_id(&self, user_id: &UserId) -> Result<Vec<Session>, DomainError>;
    
    /// Find active sessions for a user (not expired, not revoked)
    async fn find_active_byuser_id(&self, user_id: &UserId) -> Result<Vec<Session>, DomainError>;
    
    /// Find sessions by access token
    async fn find_by_access_token(&self, access_token: &str) -> Result<Option<Session>, DomainError>;
    
    /// Find sessions by refresh token
    async fn find_byrefresh_token(&self, refresh_token: &str) -> Result<Option<Session>, DomainError>;
    
    /// Save a session (create or update)
    async fn save(&self, session: &Session) -> Result<(), DomainError>;
    
    /// Delete a session
    async fn delete(&self, id: &SessionId) -> Result<(), DomainError>;
    
    /// Invalidate all sessions for a user
    async fn invalidate_all_for_user(&self, user_id: &UserId) -> Result<u32, DomainError>;
    
    /// Find sessions that need cleanup (expired or revoked)
    async fn find_expired_sessions(&self, before: DateTime<Utc>) -> Result<Vec<Session>, DomainError>;
    
    /// Clean up expired and revoked sessions
    async fn cleanup_expired(&self, before: DateTime<Utc>) -> Result<u32, DomainError>;
    
    /// Find sessions by criteria with pagination
    async fn find_by_criteria(
        &self,
        criteria: &SessionSearchCriteria,
        limit: u32,
        offset: u32
    ) -> Result<SessionSearchResult, DomainError>;
    
    /// Count sessions matching criteria
    async fn count_by_criteria(&self, criteria: &SessionSearchCriteria) -> Result<u64, DomainError>;
    
    /// Get the next available identity
    async fn next_identity(&self) -> Result<SessionId, DomainError>;
    
    /// Health check for the repository
    async fn health_check(&self) -> Result<(), DomainError>;
    
    /// Batch operations for efficiency
    async fn save_batch(&self, sessions: &[Session]) -> Result<(), DomainError>;
    
    /// Find sessions that need renewal (expire within threshold)
    async fn find_sessions_needing_renewal(
        &self, 
        threshold: Duration
    ) -> Result<Vec<Session>, DomainError>;
    
    /// Get session statistics
    async fn get_session_statistics(&self) -> Result<SessionStatistics, DomainError>;
}

/// Search criteria for finding sessions
#[derive(Debug, Clone, Default)]
pub struct SessionSearchCriteria {
    /// Filter by user ID
    pub user_id: Option<UserId>,
    
    /// Filter by active status (not expired and not revoked)
    pub is_active: Option<bool>,
    
    /// Filter by revoked status
    pub is_revoked: Option<bool>,
    
    /// Filter by sessions created after this date
    pub created_after: Option<DateTime<Utc>>,
    
    /// Filter by sessions created before this date
    pub created_before: Option<DateTime<Utc>>,
    
    /// Filter by sessions that expire after this date
    pub expires_after: Option<DateTime<Utc>>,
    
    /// Filter by sessions that expire before this date
    pub expires_before: Option<DateTime<Utc>>,
    
    /// Filter by IP address
    pub ip_address: Option<String>,
    
    /// Filter by user agent pattern
    pub user_agent_pattern: Option<String>,
    
    /// Filter by sessions accessed after this date
    pub last_accessed_after: Option<DateTime<Utc>>,
    
    /// Filter by sessions that haven't been accessed for a duration
    pub inactive_for: Option<Duration>,
}

/// Result of a session search operation
#[derive(Debug, Clone)]
pub struct SessionSearchResult {
    /// The sessions that matched the search criteria
    pub sessions: Vec<Session>,
    
    /// Total count of sessions that match (for pagination)
    pub total_count: u64,
    
    /// The offset used in this search
    pub offset: u32,
    
    /// The limit used in this search
    pub limit: u32,
    
    /// Whether there are more results available
    pub has_more: bool,
}

impl SessionSearchResult {
    pub fn new(sessions: Vec<Session>, total_count: u64, offset: u32, limit: u32) -> Self {
        let has_more = (offset + limit) < total_count as u32;
        
        Self {
            sessions,
            total_count,
            offset,
            limit,
            has_more,
        }
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStatistics {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub expired_sessions: u64,
    pub revoked_sessions: u64,
    pub sessions_created_24h: u64,
    pub sessions_expired_24h: u64,
    pub average_session_duration_minutes: f64,
    pub unique_users_with_sessions: u64,
}

/// Extended port for session analytics
#[async_trait]
pub trait SessionAnalyticsPort: Send + Sync {
    /// Get detailed session statistics
    async fn get_detailed_statistics(&self) -> Result<SessionStatistics, DomainError>;
    
    /// Get session activity patterns over time
    async fn get_activity_patterns(
        &self, 
        days: u32
    ) -> Result<Vec<(chrono::NaiveDate, u64)>, DomainError>;
    
    /// Find suspicious session patterns (multiple IPs, unusual user agents, etc.)
    async fn find_suspicious_sessions(&self) -> Result<Vec<Session>, DomainError>;
    
    /// Get sessions grouped by IP address
    async fn get_sessions_by_ip(&self) -> Result<std::collections::HashMap<String, u64>, DomainError>;
    
    /// Get session duration distribution
    async fn get_duration_distribution(&self) -> Result<Vec<(String, u64)>, DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn session_search_result_has_more_calculation() {
        let sessions = vec![];
        let result = SessionSearchResult::new(sessions, 100, 0, 10);
        assert!(result.has_more);
        
        let result = SessionSearchResult::new(vec![], 5, 0, 10);
        assert!(!result.has_more);
        
        let result = SessionSearchResult::new(vec![], 100, 95, 10);
        assert!(!result.has_more);
    }
    
    #[test]
    fn session_search_criteria_default() {
        let criteria = SessionSearchCriteria::default();
        assert!(criteria.user_id.is_none());
        assert!(criteria.is_active.is_none());
        assert!(criteria.created_after.is_none());
    }
}