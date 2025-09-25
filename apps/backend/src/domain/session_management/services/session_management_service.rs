// Session Management Domain Service
use crate::domain::shared_kernel::value_objects::SessionId;
use crate::domain::user_management::aggregates::Session;
use chrono::{DateTime, Utc, Duration};
use tracing::{debug, warn};

pub struct SessionManagementService {
    session_timeout_hours: i64,
    refresh_threshold_hours: i64,
}

impl SessionManagementService {
    pub fn new() -> Self {
        Self {
            session_timeout_hours: 24,   // Sessions expire after 24 hours
            refresh_threshold_hours: 4,  // Refresh if session expires within 4 hours
        }
    }
    
    pub fn with_config(session_timeout_hours: i64, refresh_threshold_hours: i64) -> Self {
        Self {
            session_timeout_hours,
            refresh_threshold_hours,
        }
    }

    /// Validates session by checking expiry, user status, and security context
    pub fn is_session_valid(&self, session: &Session) -> bool {
        // Check if session has expired
        if self.is_session_expired(session) {
            debug!("Session {} has expired", session.id());
            return false;
        }
        
        // Check if session was explicitly terminated
        if session.is_terminated() {
            debug!("Session {} was terminated", session.id());
            return false;
        }
        
        // Check if user account is still active
        if !session.user_is_active() {
            warn!("Session {} belongs to inactive user", session.id());
            return false;
        }
        
        // Additional security checks
        if session.has_security_violations() {
            warn!("Session {} has security violations", session.id());
            return false;
        }
        
        debug!("Session {} is valid", session.id());
        true
    }

    /// Determines if session should be refreshed based on remaining time
    pub fn should_refresh_session(&self, session: &Session) -> bool {
        if !self.is_session_valid(session) {
            return false;
        }
        
        let now = Utc::now();
        let time_until_expiry = session.expires_at() - now;
        let refresh_threshold = Duration::hours(self.refresh_threshold_hours);
        
        let should_refresh = time_until_expiry <= refresh_threshold;
        
        if should_refresh {
            debug!("Session {} should be refreshed (expires in {} minutes)", 
                   session.id(), time_until_expiry.num_minutes());
        }
        
        should_refresh
    }
    
    /// Helper method to check if session has expired
    fn is_session_expired(&self, session: &Session) -> bool {
        let now = Utc::now();
        session.expires_at() <= now
    }
}