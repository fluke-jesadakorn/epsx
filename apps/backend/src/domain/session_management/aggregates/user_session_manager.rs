// User Session Manager Aggregate Root
// Central orchestrator for all session operations for a user

use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

use crate::domain::shared_kernel::AggregateRoot;
use crate::domain::authentication::{SessionId, AuthenticatedUserId, ProviderType};
use super::super::value_objects::{
    SessionMetadata, SessionCollection, SessionActivity, SessionHistory,
    DeviceInfo, ActivityType, HistoryEventType, SessionStatus
};

/// Main aggregate for managing all sessions for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessionManager {
    /// User this manager belongs to
    user_id: AuthenticatedUserId,
    
    /// Session collection with active and historical sessions
    session_collection: SessionCollection,
    
    /// Activity tracking across all sessions
    global_activity: SessionActivity,
    
    /// Historical record of all session events
    global_history: SessionHistory,
    
    /// Manager state
    created_at: DateTime<Utc>,
    last_updated: DateTime<Utc>,
    
    /// Configuration
    max_concurrent_sessions: u32,
    enable_security_monitoring: bool,
    enable_activity_tracking: bool,
    
    /// Aggregate infrastructure
    version: u64,
    uncommitted_events: Vec<Box<dyn crate::domain::shared_kernel::DomainEvent>>,
}

impl UserSessionManager {
    /// Create new session manager for user
    pub fn create_for_user(user_id: AuthenticatedUserId) -> Self {
        let now = Utc::now();
        let global_session_id = SessionId::generate(); // Virtual session ID for global tracking
        
        Self {
            user_id: user_id.clone(),
            session_collection: SessionCollection::new(user_id.clone()),
            global_activity: SessionActivity::new(global_session_id.clone()),
            global_history: SessionHistory::new(global_session_id, user_id),
            created_at: now,
            last_updated: now,
            max_concurrent_sessions: super::super::SessionManagementBoundedContext::MAX_CONCURRENT_SESSIONS_PER_USER,
            enable_security_monitoring: true,
            enable_activity_tracking: true,
            version: 1,
            uncommitted_events: Vec::new(),
        }
    }
    
    /// Add new session to management
    pub fn add_session(&mut self, session_metadata: SessionMetadata) -> Result<(), SessionManagerError> {
        // Validate session belongs to this user
        if session_metadata.user_id != self.user_id {
            return Err(SessionManagerError::UserMismatch);
        }
        
        // Check business rules
        self.enforce_session_limits()?;
        self.validate_session_security(&session_metadata)?;
        
        // Add to collection
        self.session_collection.add_session(session_metadata.clone())
            .map_err(SessionManagerError::CollectionError)?;
        
        // Record activity
        if self.enable_activity_tracking {
            self.global_activity.record_activity(
                ActivityType::SessionCreated,
                Some(format!("Session {} created with provider {:?}", 
                    session_metadata.session_id, session_metadata.provider_type))
            );
        }
        
        // Record history
        self.global_history.add_event(
            HistoryEventType::SessionCreated,
            Some(format!("New session created: {}", session_metadata.session_id))
        );
        
        // Update state
        self.last_updated = Utc::now();
        self.version += 1;
        
        Ok(())
    }
    
    /// Update session activity
    pub fn record_session_activity(
        &mut self,
        session_id: &SessionId,
        activity_type: ActivityType,
        ip_address: Option<String>,
        user_agent: Option<String>
    ) -> Result<(), SessionManagerError> {
        // Update session metadata
        self.session_collection.record_session_access(session_id)
            .map_err(SessionManagerError::CollectionError)?;
        
        // Record global activity if enabled
        if self.enable_activity_tracking {
            self.global_activity.record_activity_with_client(
                activity_type.clone(),
                Some(format!("Session {}", session_id)),
                ip_address.clone(),
                user_agent.clone()
            );
        }
        
        // Check for suspicious patterns
        if self.enable_security_monitoring {
            let patterns = self.global_activity.detect_suspicious_patterns();
            if !patterns.is_empty() {
                self.handle_suspicious_patterns(&patterns, session_id)?;
            }
        }
        
        // Record history event for significant activities
        if self.is_significant_activity(&activity_type) {
            self.global_history.add_event(
                HistoryEventType::ApiCallMade { 
                    endpoint: format!("{:?}", activity_type) 
                },
                Some(format!("Activity from session {}", session_id))
            );
        }
        
        self.last_updated = Utc::now();
        Ok(())
    }
    
    /// Terminate specific session
    pub fn terminate_session(&mut self, session_id: &SessionId, reason: String) -> Result<(), SessionManagerError> {
        // Get session to validate it exists
        let session = self.session_collection.get_session_mut(session_id)
            .ok_or(SessionManagerError::SessionNotFound)?;
        
        // Terminate the session
        session.terminate(reason.clone());
        
        // Record activity
        if self.enable_activity_tracking {
            self.global_activity.record_activity(
                ActivityType::SessionTerminated,
                Some(format!("Session {} terminated: {}", session_id, reason))
            );
        }
        
        // Record history
        self.global_history.add_event(
            HistoryEventType::SessionTerminated { reason: reason.clone() },
            Some(format!("Session {} terminated", session_id))
        );
        
        self.last_updated = Utc::now();
        self.version += 1;
        
        Ok(())
    }
    
    /// Terminate all active sessions
    pub fn terminate_all_sessions(&mut self, reason: String) -> Result<u32, SessionManagerError> {
        let terminated_count = self.session_collection.terminate_all_sessions(reason.clone());
        
        // Record global activity
        if self.enable_activity_tracking && terminated_count > 0 {
            self.global_activity.record_activity(
                ActivityType::Custom("mass_session_termination".to_string()),
                Some(format!("Terminated {} sessions: {}", terminated_count, reason))
            );
        }
        
        // Record history
        self.global_history.add_event(
            HistoryEventType::Custom { 
                event_name: "mass_session_termination".to_string() 
            },
            Some(format!("Terminated {} sessions: {}", terminated_count, reason))
        );
        
        self.last_updated = Utc::now();
        self.version += 1;
        
        Ok(terminated_count)
    }
    
    /// Update device information for session
    pub fn update_session_device(&mut self, session_id: &SessionId, device_info: DeviceInfo) -> Result<(), SessionManagerError> {
        let session = self.session_collection.get_session_mut(session_id)
            .ok_or(SessionManagerError::SessionNotFound)?;
        
        // Check for device changes
        let device_changed = session.device_info.as_ref()
            .map(|existing| existing.device_fingerprint.hash != device_info.device_fingerprint.hash)
            .unwrap_or(false);
        
        session.update_device_info(device_info.clone());
        
        // Record device change if significant
        if device_changed {
            self.global_history.add_event(
                HistoryEventType::DeviceChanged { 
                    device_fingerprint: device_info.device_fingerprint.hash.clone() 
                },
                Some(format!("Device changed for session {}", session_id))
            );
            
            // This might be suspicious
            if !device_info.is_trusted {
                session.add_security_flag("device_change_suspicious".to_string());
            }
        }
        
        self.last_updated = Utc::now();
        Ok(())
    }
    
    /// Get active session count
    pub fn active_session_count(&self) -> u32 {
        self.session_collection.active_sessions_count()
    }
    
    /// Get session by ID
    pub fn get_session(&self, session_id: &SessionId) -> Option<&SessionMetadata> {
        self.session_collection.get_session(session_id)
    }
    
    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<&SessionMetadata> {
        self.session_collection.get_active_sessions()
    }
    
    /// Get suspicious sessions
    pub fn get_suspicious_sessions(&self) -> Vec<&SessionMetadata> {
        self.session_collection.get_suspicious_sessions()
    }
    
    /// Perform security assessment
    pub fn perform_security_assessment(&self) -> SessionSecurityAssessment {
        let suspicious_sessions = self.get_suspicious_sessions();
        let suspicious_patterns = self.global_activity.detect_suspicious_patterns();
        
        let risk_score = self.calculate_risk_score(&suspicious_sessions, &suspicious_patterns);
        let risk_level = if risk_score > 80.0 {
            RiskLevel::Critical
        } else if risk_score > 60.0 {
            RiskLevel::High
        } else if risk_score > 30.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };
        
        SessionSecurityAssessment {
            user_id: self.user_id.clone(),
            risk_level,
            risk_score,
            suspicious_session_count: suspicious_sessions.len() as u32,
            active_session_count: self.active_session_count(),
            suspicious_patterns: suspicious_patterns.len() as u32,
            last_assessed: Utc::now(),
            recommendations: self.generate_security_recommendations(&risk_level, &suspicious_sessions),
        }
    }
    
    /// Cleanup expired sessions
    pub fn cleanup_expired_sessions(&mut self) -> u32 {
        let removed = self.session_collection.cleanup_expired_sessions();
        
        if removed > 0 {
            self.global_history.add_event(
                HistoryEventType::Custom { event_name: "session_cleanup".to_string() },
                Some(format!("Cleaned up {} expired sessions", removed))
            );
            
            self.last_updated = Utc::now();
            self.version += 1;
        }
        
        removed
    }
    
    /// Get comprehensive session manager summary
    pub fn get_manager_summary(&self) -> SessionManagerSummary {
        let collection_summary = self.session_collection.get_summary();
        let activity_summary = self.global_activity.get_summary();
        let history_summary = self.global_history.get_lifecycle_summary();
        
        SessionManagerSummary {
            user_id: self.user_id.clone(),
            created_at: self.created_at,
            last_updated: self.last_updated,
            total_sessions: collection_summary.total_sessions,
            active_sessions: collection_summary.active_sessions,
            suspicious_sessions: collection_summary.suspicious_sessions,
            total_activities: activity_summary.total_activities,
            total_history_events: history_summary.total_events,
            average_session_duration: collection_summary.average_session_duration_minutes,
            security_assessment: self.perform_security_assessment(),
        }
    }
    
    // Private helper methods
    
    fn enforce_session_limits(&self) -> Result<(), SessionManagerError> {
        if self.session_collection.active_sessions_count() >= self.max_concurrent_sessions {
            Err(SessionManagerError::SessionLimitExceeded)
        } else {
            Ok(())
        }
    }
    
    fn validate_session_security(&self, session: &SessionMetadata) -> Result<(), SessionManagerError> {
        // Check if user has too many suspicious sessions
        let suspicious_count = self.session_collection.get_suspicious_sessions().len();
        if suspicious_count > 2 {
            return Err(SessionManagerError::TooManySuspiciousSessions);
        }
        
        // Check session itself for suspicious indicators
        if session.is_suspicious() {
            return Err(SessionManagerError::SuspiciousSessionDetected);
        }
        
        Ok(())
    }
    
    fn handle_suspicious_patterns(&mut self, _patterns: &[super::super::value_objects::SuspiciousPattern], session_id: &SessionId) -> Result<(), SessionManagerError> {
        // Record security alert
        self.global_history.add_event(
            HistoryEventType::SuspiciousActivityDetected { 
                activity: "pattern_detected".to_string() 
            },
            Some(format!("Suspicious patterns detected for session {}", session_id))
        );
        
        // Mark session as suspicious
        if let Some(session) = self.session_collection.get_session_mut(session_id) {
            session.add_security_flag("suspicious_pattern_detected".to_string());
        }
        
        Ok(())
    }
    
    fn is_significant_activity(&self, activity_type: &ActivityType) -> bool {
        matches!(activity_type, 
            ActivityType::Authentication |
            ActivityType::AuthenticationFailed |
            ActivityType::SessionCreated |
            ActivityType::SessionTerminated |
            ActivityType::SecurityAlert |
            ActivityType::TradeExecuted |
            ActivityType::AdminActionPerformed(_)
        )
    }
    
    fn calculate_risk_score(&self, suspicious_sessions: &[&SessionMetadata], patterns: &[super::super::value_objects::SuspiciousPattern]) -> f64 {
        let mut score = 0.0;
        
        // Base score from suspicious sessions
        score += suspicious_sessions.len() as f64 * 20.0;
        
        // Score from patterns
        score += patterns.len() as f64 * 15.0;
        
        // Score from session count (too many sessions is risky)
        if self.active_session_count() > 5 {
            score += 10.0;
        }
        
        // Score from failed activities
        score += self.global_activity.metrics.failed_auth_attempts as f64 * 5.0;
        
        score.min(100.0)
    }
    
    fn generate_security_recommendations(&self, risk_level: &RiskLevel, suspicious_sessions: &[&SessionMetadata]) -> Vec<String> {
        let mut recommendations = vec![];
        
        match risk_level {
            RiskLevel::Critical | RiskLevel::High => {
                recommendations.push("Consider terminating all sessions and requiring re-authentication".to_string());
                recommendations.push("Enable additional security monitoring".to_string());
                recommendations.push("Require multi-factor authentication for new sessions".to_string());
            },
            RiskLevel::Medium => {
                recommendations.push("Monitor user activity closely".to_string());
                if !suspicious_sessions.is_empty() {
                    recommendations.push("Consider terminating suspicious sessions".to_string());
                }
            },
            RiskLevel::Low => {
                recommendations.push("Continue normal monitoring".to_string());
            }
        }
        
        if self.active_session_count() > 5 {
            recommendations.push("User has many active sessions - consider session cleanup".to_string());
        }
        
        recommendations
    }
}

impl AggregateRoot for UserSessionManager {
    type Id = AuthenticatedUserId;
    
    fn id(&self) -> &Self::Id {
        &self.user_id
    }
    
    fn version(&self) -> u64 {
        self.version
    }
    
    fn increment_version(&mut self) {
        self.version += 1;
        self.touch();
    }
    
    fn uncommitted_events(&self) -> &[Box<dyn crate::domain::shared_kernel::DomainEvent>] {
        &self.uncommitted_events
    }
    
    fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> DateTime<Utc> {
        self.last_updated
    }
    
    fn touch(&mut self) {
        self.last_updated = Utc::now();
    }
}

/// Session manager errors
#[derive(Debug, thiserror::Error)]
pub enum SessionManagerError {
    #[error("Session does not belong to this user")]
    UserMismatch,
    
    #[error("Session not found")]
    SessionNotFound,
    
    #[error("Session limit exceeded")]
    SessionLimitExceeded,
    
    #[error("Too many suspicious sessions")]
    TooManySuspiciousSessions,
    
    #[error("Suspicious session detected")]
    SuspiciousSessionDetected,
    
    #[error("Session collection error: {0}")]
    CollectionError(#[from] super::super::value_objects::SessionCollectionError),
    
    #[error("Security validation failed")]
    SecurityValidationFailed,
}

/// Risk levels for security assessment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Security assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSecurityAssessment {
    pub user_id: AuthenticatedUserId,
    pub risk_level: RiskLevel,
    pub risk_score: f64,
    pub suspicious_session_count: u32,
    pub active_session_count: u32,
    pub suspicious_patterns: u32,
    pub last_assessed: DateTime<Utc>,
    pub recommendations: Vec<String>,
}

/// Comprehensive session manager summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagerSummary {
    pub user_id: AuthenticatedUserId,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub suspicious_sessions: u32,
    pub total_activities: usize,
    pub total_history_events: usize,
    pub average_session_duration: f64,
    pub security_assessment: SessionSecurityAssessment,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_management::value_objects::UserId;
    
    fn create_test_session(user_id: AuthenticatedUserId, provider: ProviderType) -> SessionMetadata {
        SessionMetadata::new(
            SessionId::generate(),
            user_id,
            provider,
            Utc::now() + Duration::hours(8),
        )
    }
    
    #[test]
    fn create_session_manager() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let manager = UserSessionManager::create_for_user(user_id.clone());
        
        assert_eq!(*manager.id(), user_id);
        assert_eq!(manager.active_session_count(), 0);
        assert_eq!(manager.version(), 1);
    }
    
    #[test]
    fn add_session_to_manager() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut manager = UserSessionManager::create_for_user(user_id.clone());
        
        let session = create_test_session(user_id, ProviderType::Firebase);
        let session_id = session.session_id.clone();
        
        let result = manager.add_session(session);
        assert!(result.is_ok());
        
        assert_eq!(manager.active_session_count(), 1);
        assert!(manager.get_session(&session_id).is_some());
        assert_eq!(manager.version(), 2); // Version incremented
    }
    
    #[test]
    fn record_session_activity() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut manager = UserSessionManager::create_for_user(user_id.clone());
        
        let session = create_test_session(user_id, ProviderType::Firebase);
        let session_id = session.session_id.clone();
        
        manager.add_session(session).unwrap();
        
        let result = manager.record_session_activity(
            &session_id,
            ActivityType::ApiCall("analytics".to_string()),
            Some("192.168.1.1".to_string()),
            None
        );
        
        assert!(result.is_ok());
        
        let summary = manager.get_manager_summary();
        assert!(summary.total_activities > 0);
    }
    
    #[test]
    fn terminate_session() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut manager = UserSessionManager::create_for_user(user_id.clone());
        
        let session = create_test_session(user_id, ProviderType::Firebase);
        let session_id = session.session_id.clone();
        
        manager.add_session(session).unwrap();
        assert_eq!(manager.active_session_count(), 1);
        
        let result = manager.terminate_session(&session_id, "user_logout".to_string());
        assert!(result.is_ok());
        
        assert_eq!(manager.active_session_count(), 0);
        
        // Session should still exist but be terminated
        let session = manager.get_session(&session_id).unwrap();
        assert!(session.is_terminated());
    }
    
    #[test]
    fn security_assessment() {
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut manager = UserSessionManager::create_for_user(user_id.clone());
        
        // Add a normal session
        let session = create_test_session(user_id, ProviderType::Firebase);
        manager.add_session(session).unwrap();
        
        let assessment = manager.perform_security_assessment();
        
        assert_eq!(assessment.user_id, *manager.id());
        assert_eq!(assessment.active_session_count, 1);
        assert_eq!(assessment.suspicious_session_count, 0);
        assert!(matches!(assessment.risk_level, RiskLevel::Low));
    }
}