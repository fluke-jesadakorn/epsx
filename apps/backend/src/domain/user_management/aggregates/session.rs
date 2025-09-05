use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::domain::shared_kernel::{
    AggregateRoot, 
    DomainEvent, 
    DomainError, 
    DomainResult,
    aggregate_root::AggregateBase
};

use crate::domain::user_management::value_objects::{
    UserId, SessionId
};

use crate::domain::user_management::events::{
    SessionCreatedEvent,
    SessionInvalidatedEvent,
    SessionExtendedEvent,
    session_events::SessionInvalidationReason
};

/// Session aggregate root
/// Represents a user session with security and lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    // Identity
    id: SessionId,
    user_id: UserId,
    
    // Session data
    access_token: String,
    refresh_token: Option<String>,
    
    // Timestamps
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_accessed_at: DateTime<Utc>,
    
    // Security
    ip_address: Option<String>,
    user_agent: Option<String>,
    is_revoked: bool,
    
    // Aggregate infrastructure
    #[serde(flatten)]
    base: AggregateBase,
}

impl Session {
    /// Create a new session
    pub fn create(
        id: SessionId,
        user_id: UserId,
        access_token: String,
        expires_at: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> DomainResult<Self> {
        // Business rule: Session must expire in the future
        if expires_at <= Utc::now() {
            return Err(DomainError::business_rule_violation(
                "Session expiration must be in the future"
            ));
        }
        
        // Business rule: Access token cannot be empty
        if access_token.trim().is_empty() {
            return Err(DomainError::validation_error(
                "access_token", 
                "Access token cannot be empty"
            ));
        }
        
        let now = Utc::now();
        let base = AggregateBase::new();
        
        let mut session = Self {
            id: id.clone(),
            user_id: user_id.clone(),
            access_token,
            refresh_token: None,
            created_at: now,
            updated_at: now,
            expires_at,
            last_accessed_at: now,
            ip_address: ip_address.clone(),
            user_agent: user_agent.clone(),
            is_revoked: false,
            base,
        };
        
        // Raise domain event
        session.base.add_event(Box::new(SessionCreatedEvent::new(
            id,
            user_id,
            expires_at,
            ip_address,
            user_agent,
            session.base.version
        )));
        
        Ok(session)
    }
    
    /// Load existing session (for repository reconstruction)
    pub fn load(
        id: SessionId,
        user_id: UserId,
        access_token: String,
        refresh_token: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        last_accessed_at: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        is_revoked: bool,
        version: u64,
    ) -> Self {
        let mut base = AggregateBase::new();
        base.version = version;
        base.created_at = created_at;
        base.updated_at = updated_at;
        
        Self {
            id,
            user_id,
            access_token,
            refresh_token,
            created_at,
            updated_at,
            expires_at,
            last_accessed_at,
            ip_address,
            user_agent,
            is_revoked,
            base,
        }
    }
    
    // Getters
    pub fn id(&self) -> &SessionId {
        &self.id
    }
    
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }
    
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
    
    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }
    
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
    
    pub fn last_accessed_at(&self) -> DateTime<Utc> {
        self.last_accessed_at
    }
    
    pub fn ip_address(&self) -> Option<&str> {
        self.ip_address.as_deref()
    }
    
    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }
    
    pub fn is_revoked(&self) -> bool {
        self.is_revoked
    }
    
    // Business operations
    
    /// Check if the session is valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked
    }
    
    /// Check if the session is active (same as is_valid)
    pub fn is_active(&self) -> bool {
        self.is_valid()
    }
    
    /// Check if the session has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Extend the session expiration
    pub fn extend(&mut self, new_expires_at: DateTime<Utc>) -> DomainResult<()> {
        // Business rule: Cannot extend revoked session
        if self.is_revoked {
            return Err(DomainError::business_rule_violation(
                "Cannot extend revoked session"
            ));
        }
        
        // Business rule: New expiration must be in the future
        if new_expires_at <= Utc::now() {
            return Err(DomainError::business_rule_violation(
                "New expiration must be in the future"
            ));
        }
        
        // Business rule: Cannot extend to earlier time than current expiration
        if new_expires_at < self.expires_at {
            return Err(DomainError::business_rule_violation(
                "Cannot extend session to earlier expiration time"
            ));
        }
        
        let old_expires_at = self.expires_at;
        self.expires_at = new_expires_at;
        self.base.touch();
        
        // Raise domain event
        self.base.add_event(Box::new(SessionExtendedEvent::new(
            self.id.clone(),
            self.user_id.clone(),
            old_expires_at,
            new_expires_at,
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Invalidate the session
    pub fn invalidate(&mut self, reason: SessionInvalidationReason) -> DomainResult<()> {
        // Business rule: Cannot invalidate already revoked session
        if self.is_revoked {
            return Err(DomainError::business_rule_violation(
                "Session is already revoked"
            ));
        }
        
        self.is_revoked = true;
        self.base.touch();
        
        // Raise domain event
        self.base.add_event(Box::new(SessionInvalidatedEvent::new(
            self.id.clone(),
            self.user_id.clone(),
            reason,
            self.base.version
        )));
        
        Ok(())
    }
    
    /// Deactivate the session (alias for invalidate with default reason)
    pub fn deactivate(&mut self) {
        let _ = self.invalidate(SessionInvalidationReason::UserLogout);
    }
    
    /// Refresh the session by updating access time and extending if needed
    pub fn refresh(&mut self) {
        let _ = self.update_last_accessed();
    }
    
    /// Update last accessed timestamp
    pub fn update_last_accessed(&mut self) -> DomainResult<()> {
        // Business rule: Cannot update revoked session
        if self.is_revoked {
            return Err(DomainError::business_rule_violation(
                "Cannot update revoked session"
            ));
        }
        
        // Business rule: Cannot update expired session
        if self.is_expired() {
            return Err(DomainError::business_rule_violation(
                "Cannot update expired session"
            ));
        }
        
        self.last_accessed_at = Utc::now();
        self.base.touch();
        
        Ok(())
    }
    
    /// Set refresh token
    pub fn set_refresh_token(&mut self, refresh_token: String) -> DomainResult<()> {
        // Business rule: Cannot set refresh token on revoked session
        if self.is_revoked {
            return Err(DomainError::business_rule_violation(
                "Cannot set refresh token on revoked session"
            ));
        }
        
        // Business rule: Refresh token cannot be empty
        if refresh_token.trim().is_empty() {
            return Err(DomainError::validation_error(
                "refresh_token",
                "Refresh token cannot be empty"
            ));
        }
        
        self.refresh_token = Some(refresh_token);
        self.base.touch();
        
        Ok(())
    }
    
    /// Clear refresh token
    pub fn clear_refresh_token(&mut self) {
        if self.refresh_token.is_some() {
            self.refresh_token = None;
            self.base.touch();
        }
    }
    
    /// Check if session needs renewal (expires soon)
    pub fn needs_renewal(&self, threshold: Duration) -> bool {
        if self.is_revoked || self.is_expired() {
            return false;
        }
        
        let renewal_time = self.expires_at - threshold;
        Utc::now() >= renewal_time
    }
    
    /// Get session age in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }
    
    /// Get time until expiration in seconds
    pub fn time_until_expiry_seconds(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds()
    }
    
    /// Check if this session matches the given IP address
    pub fn matches_ip(&self, ip: &str) -> bool {
        self.ip_address.as_deref() == Some(ip)
    }
    
    /// Check if this session matches the given user agent
    pub fn matches_user_agent(&self, user_agent: &str) -> bool {
        self.user_agent.as_deref() == Some(user_agent)
    }
    
    /// Update session metadata (user agent, IP, etc.)
    pub fn update_metadata(&mut self, metadata: String) -> DomainResult<()> {
        // Parse metadata format "key:value"
        if let Some((key, value)) = metadata.split_once(':') {
            match key {
                "user_agent" => {
                    self.user_agent = Some(value.to_string());
                    self.base.touch();
                }
                "ip_address" => {
                    self.ip_address = Some(value.to_string());
                    self.base.touch();
                }
                _ => {
                    // Ignore unknown metadata keys
                }
            }
        }
        Ok(())
    }
}

impl AggregateRoot for Session {
    type Id = SessionId;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn version(&self) -> u64 {
        self.base.version
    }
    
    fn increment_version(&mut self) {
        self.base.increment_version();
    }
    
    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.base.events
    }
    
    fn mark_events_as_committed(&mut self) {
        self.base.clear_events();
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    
    fn touch(&mut self) {
        self.base.touch();
        self.updated_at = self.base.updated_at;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    
    fn create_test_session() -> Session {
        Session::create(
            SessionId::from_uuid(uuid::Uuid::new_v4()),
            UserId::new(),
            "test_access_token".to_string(),
            Utc::now() + Duration::hours(1),
            Some("127.0.0.1".to_string()),
            Some("test-agent".to_string()),
        ).unwrap()
    }
    
    #[test]
    fn create_session_should_succeed() {
        let session = create_test_session();
        assert!(session.is_valid());
        assert!(!session.is_expired());
        assert!(!session.is_revoked());
        assert_eq!(session.uncommitted_events().len(), 1);
    }
    
    #[test]
    fn create_session_with_past_expiration_should_fail() {
        let result = Session::create(
            SessionId::from_uuid(uuid::Uuid::new_v4()),
            UserId::new(),
            "test_token".to_string(),
            Utc::now() - Duration::hours(1), // Past expiration
            None,
            None,
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn extend_session_should_succeed() {
        let mut session = create_test_session();
        let new_expiration = Utc::now() + Duration::hours(2);
        
        let result = session.extend(new_expiration);
        assert!(result.is_ok());
        assert_eq!(session.expires_at(), new_expiration);
    }
    
    #[test]
    fn invalidate_session_should_succeed() {
        let mut session = create_test_session();
        
        let result = session.invalidate(SessionInvalidationReason::UserLogout);
        assert!(result.is_ok());
        assert!(session.is_revoked());
        assert!(!session.is_valid());
    }
    
    #[test]
    fn extend_revoked_session_should_fail() {
        let mut session = create_test_session();
        session.invalidate(SessionInvalidationReason::UserLogout).unwrap();
        
        let result = session.extend(Utc::now() + Duration::hours(2));
        assert!(result.is_err());
    }
    
    #[test]
    fn needs_renewal_logic() {
        let mut session = create_test_session();
        
        // Should not need renewal initially
        assert!(!session.needs_renewal(Duration::minutes(30)));
        
        // Extend to very short time
        session.extend(Utc::now() + Duration::minutes(10)).unwrap();
        
        // Should need renewal now
        assert!(session.needs_renewal(Duration::minutes(30)));
    }
    
    #[test]
    fn session_matching() {
        let session = Session::create(
            SessionId::from_uuid(uuid::Uuid::new_v4()),
            UserId::new(),
            "test_token".to_string(),
            Utc::now() + Duration::hours(1),
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        ).unwrap();
        
        assert!(session.matches_ip("192.168.1.1"));
        assert!(!session.matches_ip("10.0.0.1"));
        
        assert!(session.matches_user_agent("Mozilla/5.0"));
        assert!(!session.matches_user_agent("Chrome"));
    }
}