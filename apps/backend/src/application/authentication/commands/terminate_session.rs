// Maps to logout/session termination flow

use serde::{Serialize, Deserialize};
use crate::domain::authentication::{SessionId, TerminationReason};
use crate::application::shared::command_bus::Command;
use chrono::{DateTime, Utc};


/// Command to terminate an authentication session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminateSessionCommand {
    /// Session to terminate
    pub session_id: SessionId,
    
    /// Reason for termination
    pub reason: TerminationReason,
    
    /// Whether to revoke all tokens immediately
    pub revoke_tokens: bool,
    
    /// Whether to notify user of termination
    pub notify_user: bool,
}

impl TerminateSessionCommand {
    /// Create user logout termination
    pub fn user_logout(session_id: SessionId) -> Self {
        Self {
            session_id,
            reason: TerminationReason::UserLogout,
            revoke_tokens: true,
            notify_user: false, // User initiated, no need to notify
        }
    }
    
    /// Create admin-initiated termination
    pub fn admin_termination(session_id: SessionId) -> Self {
        Self {
            session_id,
            reason: TerminationReason::AdminTermination,
            revoke_tokens: true,
            notify_user: true, // Admin termination should notify user
        }
    }
    
    /// Create security-related termination
    pub fn security_termination(session_id: SessionId) -> Self {
        Self {
            session_id,
            reason: TerminationReason::SecurityThreat,
            revoke_tokens: true,
            notify_user: true, // Security issues should always notify
        }
    }
    
    /// Create session expiry termination
    pub fn session_expired(session_id: SessionId) -> Self {
        Self {
            session_id,
            reason: TerminationReason::SessionExpiry,
            revoke_tokens: true,
            notify_user: false, // Natural expiry, no notification needed
        }
    }
    
    /// Create token revocation termination
    pub fn token_revocation(session_id: SessionId) -> Self {
        Self {
            session_id,
            reason: TerminationReason::TokenRevocation,
            revoke_tokens: true,
            notify_user: false, // Token-specific, user may not need notification
        }
    }
}

/// Response from session termination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminateSessionResponse {
    /// Session that was terminated
    pub session_id: SessionId,
    
    /// When termination occurred
    pub terminated_at: DateTime<Utc>,
    
    /// Reason for termination
    pub reason: TerminationReason,
    
    /// Whether tokens were revoked
    pub tokens_revoked: bool,
    
    /// Number of tokens that were revoked
    pub revoked_token_count: u32,
    
    /// Whether user was notified
    pub user_notified: bool,
}

impl TerminateSessionResponse {
    /// Create successful termination response
    pub fn success(
        session_id: SessionId,
        reason: TerminationReason,
        tokens_revoked: bool,
        revoked_token_count: u32,
        user_notified: bool,
    ) -> Self {
        Self {
            session_id,
            terminated_at: Utc::now(),
            reason,
            tokens_revoked,
            revoked_token_count,
            user_notified,
        }
    }
}

/// Validation for terminate session command
impl TerminateSessionCommand {
    /// Validate termination command
    pub fn validate(&self) -> Result<(), TerminateSessionValidationError> {
        // Session ID validation is handled by the SessionId type itself
        
        // Business rule: Security terminations must revoke tokens
        if matches!(self.reason, TerminationReason::SecurityThreat) && !self.revoke_tokens {
            return Err(TerminateSessionValidationError::SecurityTerminationMustRevokeTokens);
        }
        
        // Business rule: Admin terminations should notify user
        if matches!(self.reason, TerminationReason::AdminTermination) && !self.notify_user {
            tracing::warn!("Admin termination without user notification - this may be intentional but is unusual");
        }
        
        Ok(())
    }
}

/// Validation errors for terminate session command
#[derive(Debug, thiserror::Error)]
pub enum TerminateSessionValidationError {
    #[error("Security threat terminations must revoke tokens")]
    SecurityTerminationMustRevokeTokens,
}

/// Termination statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminationStatistics {
    /// Total sessions terminated
    pub total_terminated: u64,
    
    /// Breakdown by reason
    pub user_logouts: u64,
    pub admin_terminations: u64,
    pub security_terminations: u64,
    pub session_expiries: u64,
    pub token_revocations: u64,
    
    /// Security metrics
    pub avg_session_duration_minutes: f64,
    pub suspicious_termination_rate: f64,
}

impl Default for TerminationStatistics {
    fn default() -> Self {
        Self {
            total_terminated: 0,
            user_logouts: 0,
            admin_terminations: 0,
            security_terminations: 0,
            session_expiries: 0,
            token_revocations: 0,
            avg_session_duration_minutes: 0.0,
            suspicious_termination_rate: 0.0,
        }
    }
}

impl TerminationStatistics {
    /// Record a new termination
    pub fn record_termination(&mut self, reason: &TerminationReason) {
        self.total_terminated += 1;
        
        match reason {
            TerminationReason::UserLogout => self.user_logouts += 1,
            TerminationReason::AdminTermination => self.admin_terminations += 1,
            TerminationReason::SecurityThreat => self.security_terminations += 1,
            TerminationReason::SessionExpiry => self.session_expiries += 1,
            TerminationReason::TokenRevocation => self.token_revocations += 1,
        }
        
        // Update suspicious termination rate
        let suspicious = self.admin_terminations + self.security_terminations;
        self.suspicious_termination_rate = if self.total_terminated > 0 {
            (suspicious as f64 / self.total_terminated as f64) * 100.0
        } else {
            0.0
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::authentication::SessionId;
    
    #[test]
    fn user_logout_command() {
        let session_id = SessionId::generate();
        let command = TerminateSessionCommand::user_logout(session_id.clone());
        
        assert_eq!(command.session_id, session_id);
        assert_eq!(command.reason, TerminationReason::UserLogout);
        assert!(command.revoke_tokens);
        assert!(!command.notify_user);
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn admin_termination_command() {
        let session_id = SessionId::generate();
        let command = TerminateSessionCommand::admin_termination(session_id.clone());
        
        assert_eq!(command.reason, TerminationReason::AdminTermination);
        assert!(command.revoke_tokens);
        assert!(command.notify_user);
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn security_termination_command() {
        let session_id = SessionId::generate();
        let command = TerminateSessionCommand::security_termination(session_id.clone());
        
        assert_eq!(command.reason, TerminationReason::SecurityThreat);
        assert!(command.revoke_tokens);
        assert!(command.notify_user);
        assert!(command.validate().is_ok());
    }
    
    #[test]
    fn security_termination_must_revoke_tokens() {
        let session_id = SessionId::generate();
        let mut command = TerminateSessionCommand::security_termination(session_id);
        command.revoke_tokens = false; // Invalid for security termination
        
        assert!(command.validate().is_err());
    }
    
    #[test]
    fn termination_statistics() {
        let mut stats = TerminationStatistics::default();
        
        stats.record_termination(&TerminationReason::UserLogout);
        stats.record_termination(&TerminationReason::SecurityThreat);
        stats.record_termination(&TerminationReason::AdminTermination);
        
        assert_eq!(stats.total_terminated, 3);
        assert_eq!(stats.user_logouts, 1);
        assert_eq!(stats.security_terminations, 1);
        assert_eq!(stats.admin_terminations, 1);
        
        // 2 out of 3 are suspicious (admin + security)
        assert!((stats.suspicious_termination_rate - 66.67).abs() < 0.1);
    }
}

impl Command for TerminateSessionCommand {
    type Response = TerminateSessionResponse;
}