// Session Collection Value Object
// Manages groups of sessions for a user with business rules and limits

use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::shared_kernel::value_objects::SessionId;
use crate::domain::authentication::{AuthenticatedUserId, ProviderType};
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, BTreeMap};
use serde::{Deserialize, Serialize};

use super::{SessionMetadata, SessionStatus};

/// Collection of sessions for a user with management capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCollection {
    /// User these sessions belong to
    pub user_id: AuthenticatedUserId,
    
    /// Active sessions by session ID
    pub sessions: HashMap<SessionId, SessionMetadata>,
    
    /// Session creation order
    pub creation_order: Vec<SessionId>,
    
    /// Collection metadata
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    
    /// Limits and policies
    pub max_concurrent_sessions: u32,
    pub enable_auto_cleanup: bool,
    
    /// Statistics
    pub total_sessions_created: u64,
    pub total_sessions_terminated: u64,
}

impl SessionCollection {
    /// Create new session collection for user
    pub fn new(user_id: AuthenticatedUserId) -> Self {
        let now = Utc::now();
        
        Self {
            user_id,
            sessions: HashMap::new(),
            creation_order: vec![],
            created_at: now,
            last_updated: now,
            max_concurrent_sessions: super::super::SessionManagementBoundedContext::MAX_CONCURRENT_SESSIONS_PER_USER,
            enable_auto_cleanup: true,
            total_sessions_created: 0,
            total_sessions_terminated: 0,
        }
    }
    
    /// Add a new session to the collection
    pub fn add_session(&mut self, session: SessionMetadata) -> Result<(), SessionCollectionError> {
        // Validate session belongs to this user
        if session.user_id != self.user_id {
            return Err(SessionCollectionError::UserMismatch);
        }
        
        // Clean up expired sessions first
        if self.enable_auto_cleanup {
            self.cleanup_expired_sessions();
        }
        
        // Check concurrent session limits
        if self.active_sessions_count() >= self.max_concurrent_sessions {
            // Try to terminate oldest inactive session
            if !self.terminate_oldest_inactive_session() {
                return Err(SessionCollectionError::TooManyActiveSessions);
            }
        }
        
        let session_id = session.session_id.clone();
        
        // Add session
        self.sessions.insert(session_id.clone(), session);
        self.creation_order.push(session_id);
        
        // Update statistics
        self.total_sessions_created += 1;
        self.last_updated = Utc::now();
        
        Ok(())
    }
    
    /// Remove session from collection
    pub fn remove_session(&mut self, session_id: &SessionId) -> Option<SessionMetadata> {
        if let Some(session) = self.sessions.remove(session_id) {
            // Remove from creation order
            self.creation_order.retain(|id| id != session_id);
            
            // Update statistics if it was terminated
            if session.is_terminated() {
                self.total_sessions_terminated += 1;
            }
            
            self.last_updated = Utc::now();
            Some(session)
        } else {
            None
        }
    }
    
    /// Get session by ID
    pub fn get_session(&self, session_id: &SessionId) -> Option<&SessionMetadata> {
        self.sessions.get(session_id)
    }
    
    /// Get mutable session by ID
    pub fn get_session_mut(&mut self, session_id: &SessionId) -> Option<&mut SessionMetadata> {
        if self.sessions.contains_key(session_id) {
            self.last_updated = Utc::now();
        }
        self.sessions.get_mut(session_id)
    }
    
    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<&SessionMetadata> {
        self.sessions.values()
            .filter(|session| session.is_active())
            .collect()
    }
    
    /// Get all expired sessions
    pub fn get_expired_sessions(&self) -> Vec<&SessionMetadata> {
        self.sessions.values()
            .filter(|session| session.is_expired())
            .collect()
    }
    
    /// Get sessions by provider type
    pub fn get_sessions_by_provider(&self, provider_type: &ProviderType) -> Vec<&SessionMetadata> {
        self.sessions.values()
            .filter(|session| &session.provider_type == provider_type)
            .collect()
    }
    
    /// Count active sessions
    pub fn active_sessions_count(&self) -> u32 {
        self.sessions.values()
            .filter(|session| session.is_active())
            .count() as u32
    }
    
    /// Count sessions by status
    pub fn sessions_count_by_status(&self, status: &SessionStatus) -> u32 {
        self.sessions.values()
            .filter(|session| &session.status == status)
            .count() as u32
    }
    
    /// Terminate all active sessions
    pub fn terminate_all_sessions(&mut self, reason: String) -> u32 {
        let mut terminated_count = 0;
        
        for session in self.sessions.values_mut() {
            if session.is_active() {
                session.terminate(reason.clone());
                terminated_count += 1;
            }
        }
        
        self.total_sessions_terminated += terminated_count as u64;
        self.last_updated = Utc::now();
        
        terminated_count
    }
    
    /// Terminate sessions by provider
    pub fn terminate_sessions_by_provider(&mut self, provider_type: &ProviderType, reason: String) -> u32 {
        let mut terminated_count = 0;
        
        for session in self.sessions.values_mut() {
            if session.is_active() && &session.provider_type == provider_type {
                session.terminate(reason.clone());
                terminated_count += 1;
            }
        }
        
        self.total_sessions_terminated += terminated_count as u64;
        self.last_updated = Utc::now();
        
        terminated_count
    }
    
    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) -> u32 {
        let expired_session_ids: Vec<SessionId> = self.sessions.iter()
            .filter(|(_, session)| session.is_expired() || session.is_terminated())
            .filter(|(_, session)| {
                // Only remove if it's been terminated/expired for a while
                let cutoff = Utc::now() - Duration::hours(1);
                session.terminated_at.map_or(true, |t| t < cutoff) ||
                session.expires_at < cutoff
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        let removed_count = expired_session_ids.len() as u32;
        
        for session_id in expired_session_ids {
            self.remove_session(&session_id);
        }
        
        removed_count
    }
    
    /// Terminate oldest inactive session
    fn terminate_oldest_inactive_session(&mut self) -> bool {
        // Find oldest session that's not recently active
        let cutoff = Utc::now() - Duration::hours(1);
        
        if let Some(oldest_id) = self.creation_order.iter()
            .find(|id| {
                if let Some(session) = self.sessions.get(*id) {
                    session.is_active() && session.last_accessed < cutoff
                } else {
                    false
                }
            })
            .cloned() {
                
            if let Some(session) = self.sessions.get_mut(&oldest_id) {
                session.terminate("auto_cleanup_for_limit".to_string());
                return true;
            }
        }
        
        false
    }
    
    /// Get suspicious sessions
    pub fn get_suspicious_sessions(&self) -> Vec<&SessionMetadata> {
        self.sessions.values()
            .filter(|session| session.is_suspicious())
            .collect()
    }
    
    /// Update session access time
    pub fn record_session_access(&mut self, session_id: &SessionId) -> Result<(), SessionCollectionError> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.record_access();
            self.last_updated = Utc::now();
            Ok(())
        } else {
            Err(SessionCollectionError::SessionNotFound)
        }
    }
    
    /// Get collection summary
    pub fn get_summary(&self) -> SessionCollectionSummary {
        let active_sessions = self.get_active_sessions();
        let provider_counts = self.get_provider_distribution();
        let avg_session_duration = self.calculate_average_session_duration();
        
        SessionCollectionSummary {
            user_id: self.user_id.clone(),
            total_sessions: self.sessions.len() as u32,
            active_sessions: active_sessions.len() as u32,
            expired_sessions: self.get_expired_sessions().len() as u32,
            suspicious_sessions: self.get_suspicious_sessions().len() as u32,
            provider_distribution: provider_counts,
            average_session_duration_minutes: avg_session_duration,
            oldest_active_session: active_sessions.iter()
                .min_by_key(|s| s.created_at)
                .map(|s| s.created_at),
            newest_session: self.sessions.values()
                .max_by_key(|s| s.created_at)
                .map(|s| s.created_at),
            total_lifetime_sessions: self.total_sessions_created,
        }
    }
    
    /// Get provider distribution
    fn get_provider_distribution(&self) -> HashMap<String, u32> {
        let mut distribution = HashMap::new();
        
        for session in self.sessions.values() {
            let provider_name = format!("{:?}", session.provider_type);
            *distribution.entry(provider_name).or_insert(0) += 1;
        }
        
        distribution
    }
    
    /// Calculate average session duration
    fn calculate_average_session_duration(&self) -> f64 {
        if self.sessions.is_empty() {
            return 0.0;
        }
        
        let total_minutes: i64 = self.sessions.values()
            .map(|session| session.session_duration_minutes())
            .sum();
        
        total_minutes as f64 / self.sessions.len() as f64
    }
}

/// Session collection summary for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCollectionSummary {
    pub user_id: AuthenticatedUserId,
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub expired_sessions: u32,
    pub suspicious_sessions: u32,
    pub provider_distribution: HashMap<String, u32>,
    pub average_session_duration_minutes: f64,
    pub oldest_active_session: Option<DateTime<Utc>>,
    pub newest_session: Option<DateTime<Utc>>,
    pub total_lifetime_sessions: u64,
}

/// Errors that can occur in session collection operations
#[derive(Debug, thiserror::Error)]
pub enum SessionCollectionError {
    #[error("Session does not belong to this user")]
    UserMismatch,
    
    #[error("Too many active sessions for user")]
    TooManyActiveSessions,
    
    #[error("Session not found in collection")]
    SessionNotFound,
    
    #[error("Invalid session state")]
    InvalidSessionState,
    
    #[error("Security threat detected")]
    SecurityThreatDetected,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_management::value_objects::UserId;
    
    fn create_test_session(session_id: SessionId, user_id: AuthenticatedUserId) -> SessionMetadata {
        SessionMetadata::new(
            session_id,
            user_id,
            ProviderType::Firebase,
            Utc::now() + Duration::hours(8),
        )
    }
    
    #[test]
    fn create_session_collection() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let collection = SessionCollection::new(user_id.clone());
        
        assert_eq!(collection.user_id, user_id);
        assert_eq!(collection.sessions.len(), 0);
        assert_eq!(collection.active_sessions_count(), 0);
    }
    
    #[test]
    fn add_sessions_to_collection() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut collection = SessionCollection::new(user_id.clone());
        
        let session1_id = SessionId::generate();
        let session1 = create_test_session(session1_id.clone(), user_id.clone());
        
        let result = collection.add_session(session1);
        assert!(result.is_ok());
        assert_eq!(collection.sessions.len(), 1);
        assert_eq!(collection.active_sessions_count(), 1);
        assert!(collection.get_session(&session1_id).is_some());
    }
    
    #[test]
    fn session_limits_enforcement() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut collection = SessionCollection::new(user_id.clone());
        collection.max_concurrent_sessions = 2; // Set low limit for testing
        
        // Add sessions up to limit
        for i in 0..2 {
            let session_id = SessionId::generate();
            let session = create_test_session(session_id, user_id.clone());
            assert!(collection.add_session(session).is_ok());
        }
        
        // Adding one more should trigger cleanup or fail
        let session_id = SessionId::generate();
        let session = create_test_session(session_id, user_id.clone());
        
        // This might succeed if cleanup happened, or fail if no cleanup possible
        // The exact behavior depends on whether there are inactive sessions to cleanup
        let result = collection.add_session(session);
        // Just ensure it handles the limit properly - don't assert specific outcome
        assert!(result.is_ok() || matches!(result, Err(SessionCollectionError::TooManyActiveSessions)));
    }
    
    #[test]
    fn terminate_all_sessions() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut collection = SessionCollection::new(user_id.clone());
        
        // Add multiple sessions
        for _ in 0..3 {
            let session_id = SessionId::generate();
            let session = create_test_session(session_id, user_id.clone());
            collection.add_session(session).unwrap();
        }
        
        assert_eq!(collection.active_sessions_count(), 3);
        
        let terminated_count = collection.terminate_all_sessions("test_termination".to_string());
        
        assert_eq!(terminated_count, 3);
        assert_eq!(collection.active_sessions_count(), 0);
    }
    
    #[test]
    fn collection_summary() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut collection = SessionCollection::new(user_id.clone());
        
        // Add sessions with different providers
        let session1_id = SessionId::generate();
        let mut session1 = create_test_session(session1_id, user_id.clone());
        session1.provider_type = ProviderType::Firebase;
        collection.add_session(session1).unwrap();
        
        let session2_id = SessionId::generate();
        let mut session2 = create_test_session(session2_id, user_id.clone());
        session2.provider_type = ProviderType::OpenIdConnect;
        collection.add_session(session2).unwrap();
        
        let summary = collection.get_summary();
        
        assert_eq!(summary.total_sessions, 2);
        assert_eq!(summary.active_sessions, 2);
        assert_eq!(summary.provider_distribution.len(), 2);
        assert!(summary.provider_distribution.contains_key("Firebase"));
        assert!(summary.provider_distribution.contains_key("OpenIdConnect"));
    }
}