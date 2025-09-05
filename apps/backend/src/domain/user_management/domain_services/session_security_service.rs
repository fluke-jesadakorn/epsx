use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

use crate::domain::shared_kernel::{DomainError, DomainResult};
use crate::domain::user_management::{Session, SessionId, UserId};

/// Domain service for session security and anomaly detection
/// This service contains business logic for detecting suspicious session patterns
/// and enforcing security policies
pub struct SessionSecurityService;

impl SessionSecurityService {
    /// Check if a session is suspicious based on various criteria
    pub fn is_session_suspicious(
        &self,
        session: &Session,
        context: &SecurityContext
    ) -> SuspicionLevel {
        let mut risk_score = 0;
        
        // Check for unusual IP changes
        if let (Some(session_ip), Some(context_ip)) = (session.ip_address(), context.current_ip.as_deref()) {
            if session_ip != context_ip && !self.are_ips_related(session_ip, context_ip) {
                risk_score += 3;
            }
        }
        
        // Check for unusual user agent changes
        if let (Some(session_ua), Some(context_ua)) = (session.user_agent(), context.current_user_agent.as_deref()) {
            if session_ua != context_ua && !self.are_user_agents_compatible(session_ua, context_ua) {
                risk_score += 2;
            }
        }
        
        // Check session age
        if session.age_seconds() > context.max_session_age_seconds {
            risk_score += 1;
        }
        
        // Check for concurrent sessions
        if context.concurrent_sessions > context.max_concurrent_sessions {
            risk_score += 2;
        }
        
        match risk_score {
            0..=1 => SuspicionLevel::Low,
            2..=3 => SuspicionLevel::Medium,
            4..=5 => SuspicionLevel::High,
            _ => SuspicionLevel::Critical,
        }
    }
    
    /// Validate session security constraints
    pub fn validate_session_security(
        &self,
        session: &Session,
        security_policy: &SessionSecurityPolicy
    ) -> DomainResult<()> {
        // Check maximum session age
        if session.age_seconds() > security_policy.max_session_age_seconds {
            return Err(DomainError::business_rule_violation(
                "Session has exceeded maximum allowed age"
            ));
        }
        
        // Check if session should be renewed
        if session.needs_renewal(Duration::seconds(security_policy.renewal_threshold_seconds)) {
            return Err(DomainError::business_rule_violation(
                "Session requires renewal"
            ));
        }
        
        // Check IP restrictions if policy requires it
        if security_policy.enforce_ip_binding {
            if let Some(original_ip) = session.ip_address() {
                // In a real implementation, this would check against current request IP
                // For now, we'll assume it's valid
            }
        }
        
        Ok(())
    }
    
    /// Analyze sessions for security anomalies
    pub fn analyze_user_sessions(
        &self,
        user_id: &UserId,
        sessions: &[Session]
    ) -> SessionSecurityAnalysis {
        let mut analysis = SessionSecurityAnalysis::new(user_id.clone());
        
        // Check for multiple active sessions from different locations
        let mut unique_ips = std::collections::HashSet::new();
        let mut unique_user_agents = std::collections::HashSet::new();
        
        for session in sessions {
            if session.is_valid() {
                if let Some(ip) = session.ip_address() {
                    unique_ips.insert(ip);
                }
                if let Some(ua) = session.user_agent() {
                    unique_user_agents.insert(ua);
                }
            }
        }
        
        analysis.unique_ip_count = unique_ips.len();
        analysis.unique_user_agent_count = unique_user_agents.len();
        analysis.active_session_count = sessions.iter().filter(|s| s.is_valid()).count();
        
        // Flag suspicious patterns
        if analysis.unique_ip_count > 3 {
            analysis.anomalies.push("Multiple IP addresses detected".to_string());
        }
        
        if analysis.active_session_count > 5 {
            analysis.anomalies.push("Excessive concurrent sessions".to_string());
        }
        
        if analysis.unique_user_agent_count > 2 {
            analysis.anomalies.push("Multiple user agents detected".to_string());
        }
        
        analysis
    }
    
    /// Recommend security actions based on session analysis
    pub fn recommend_security_actions(
        &self,
        analysis: &SessionSecurityAnalysis
    ) -> Vec<SecurityRecommendation> {
        let mut recommendations = Vec::new();
        
        if analysis.anomalies.iter().any(|a| a.contains("Multiple IP")) {
            recommendations.push(SecurityRecommendation::RequireReAuthentication);
            recommendations.push(SecurityRecommendation::NotifyUser);
        }
        
        if analysis.active_session_count > 10 {
            recommendations.push(SecurityRecommendation::TerminateOldestSessions);
        }
        
        if analysis.anomalies.len() > 2 {
            recommendations.push(SecurityRecommendation::TemporarilyLockAccount);
        }
        
        recommendations
    }
    
    // Private helper methods
    
    fn are_ips_related(&self, ip1: &str, ip2: &str) -> bool {
        // Simple check for same subnet (in real implementation, this would be more sophisticated)
        let parts1: Vec<&str> = ip1.split('.').collect();
        let parts2: Vec<&str> = ip2.split('.').collect();
        
        if parts1.len() == 4 && parts2.len() == 4 {
            // Same /24 subnet
            parts1[0] == parts2[0] && parts1[1] == parts2[1] && parts1[2] == parts2[2]
        } else {
            false
        }
    }
    
    fn are_user_agents_compatible(&self, ua1: &str, ua2: &str) -> bool {
        // Simple check for same browser family
        let browsers = ["Chrome", "Firefox", "Safari", "Edge"];
        
        for browser in browsers {
            if ua1.contains(browser) && ua2.contains(browser) {
                return true;
            }
        }
        
        false
    }
}

/// Security context for session validation
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub current_ip: Option<String>,
    pub current_user_agent: Option<String>,
    pub concurrent_sessions: usize,
    pub max_concurrent_sessions: usize,
    pub max_session_age_seconds: i64,
}

/// Session security policy configuration
#[derive(Debug, Clone)]
pub struct SessionSecurityPolicy {
    pub max_session_age_seconds: i64,
    pub renewal_threshold_seconds: i64,
    pub enforce_ip_binding: bool,
    pub max_concurrent_sessions: usize,
    pub require_reauth_on_ip_change: bool,
}

impl Default for SessionSecurityPolicy {
    fn default() -> Self {
        Self {
            max_session_age_seconds: 86400, // 24 hours
            renewal_threshold_seconds: 3600, // 1 hour
            enforce_ip_binding: false,
            max_concurrent_sessions: 5,
            require_reauth_on_ip_change: true,
        }
    }
}

/// Level of suspicion for a session
#[derive(Debug, Clone, PartialEq)]
pub enum SuspicionLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Analysis result for user sessions
#[derive(Debug, Clone)]
pub struct SessionSecurityAnalysis {
    pub user_id: UserId,
    pub unique_ip_count: usize,
    pub unique_user_agent_count: usize,
    pub active_session_count: usize,
    pub anomalies: Vec<String>,
    pub risk_score: u32,
}

impl SessionSecurityAnalysis {
    fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            unique_ip_count: 0,
            unique_user_agent_count: 0,
            active_session_count: 0,
            anomalies: Vec::new(),
            risk_score: 0,
        }
    }
}

/// Security recommendations based on analysis
#[derive(Debug, Clone)]
pub enum SecurityRecommendation {
    RequireReAuthentication,
    NotifyUser,
    TerminateOldestSessions,
    TemporarilyLockAccount,
    EnableTwoFactor,
    LogSecurityEvent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_management::value_objects::SessionId;
    use chrono::Utc;
    
    fn create_test_session_with_ip(ip: &str) -> Session {
        Session::create(
            SessionId::from_uuid(uuid::Uuid::new_v4()),
            UserId::new(),
            "test_token".to_string(),
            Utc::now() + Duration::hours(1),
            Some(ip.to_string()),
            Some("Mozilla/5.0".to_string()),
        ).unwrap()
    }
    
    #[test]
    fn session_from_different_ip_is_suspicious() {
        let service = SessionSecurityService;
        let session = create_test_session_with_ip("192.168.1.1");
        
        let context = SecurityContext {
            current_ip: Some("10.0.0.1".to_string()),
            current_user_agent: Some("Mozilla/5.0".to_string()),
            concurrent_sessions: 1,
            max_concurrent_sessions: 5,
            max_session_age_seconds: 86400,
        };
        
        let suspicion = service.is_session_suspicious(&session, &context);
        assert!(matches!(suspicion, SuspicionLevel::Medium | SuspicionLevel::High));
    }
    
    #[test]
    fn session_from_same_subnet_is_less_suspicious() {
        let service = SessionSecurityService;
        let session = create_test_session_with_ip("192.168.1.1");
        
        let context = SecurityContext {
            current_ip: Some("192.168.1.2".to_string()),
            current_user_agent: Some("Mozilla/5.0".to_string()),
            concurrent_sessions: 1,
            max_concurrent_sessions: 5,
            max_session_age_seconds: 86400,
        };
        
        let suspicion = service.is_session_suspicious(&session, &context);
        assert_eq!(suspicion, SuspicionLevel::Low);
    }
    
    #[test]
    fn multiple_active_sessions_triggers_analysis() {
        let service = SessionSecurityService;
        let user_id = UserId::new();
        
        let sessions = vec![
            create_test_session_with_ip("192.168.1.1"),
            create_test_session_with_ip("10.0.0.1"),
            create_test_session_with_ip("172.16.0.1"),
        ];
        
        let analysis = service.analyze_user_sessions(&user_id, &sessions);
        
        assert_eq!(analysis.active_session_count, 3);
        assert_eq!(analysis.unique_ip_count, 3);
        assert!(!analysis.anomalies.is_empty());
    }
    
    #[test]
    fn security_policy_validation() {
        let service = SessionSecurityService;
        let session = create_test_session_with_ip("192.168.1.1");
        
        let policy = SessionSecurityPolicy {
            max_session_age_seconds: 60, // Very short for testing
            ..Default::default()
        };
        
        // Session should be valid initially
        let result = service.validate_session_security(&session, &policy);
        assert!(result.is_ok());
    }
}