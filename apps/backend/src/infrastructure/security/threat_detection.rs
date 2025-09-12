// Real-time Threat Detection and Security Monitoring
// Detects anomalies, token tampering attempts, and security violations

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use tracing::{warn, error, info};

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityEvent {
    InvalidJwtAttempt,
    PermissionIntegrityViolation,
    DeviceBindingViolation,
    SuspiciousRefreshPattern,
    UnauthorizedPermissionAccess,
    MultipleFailedAttempts,
    TokenReplayAttack,
    AnomalousPermissionRequest,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub request_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecurityMetrics {
    failed_attempts: u32,
    last_failure: DateTime<Utc>,
    threat_level: u8,
    blocked_until: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub enum SecurityError {
    MonitoringFailed(String),
    BlockedUser(String),
    RateLimited(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SecurityError::MonitoringFailed(msg) => write!(f, "Security monitoring failed: {}", msg),
            SecurityError::BlockedUser(msg) => write!(f, "User blocked: {}", msg),
            SecurityError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Real-time threat detection and security monitoring service
pub struct ThreatDetectionService {
    // In-memory cache for demo - in production use Redis
    security_metrics: std::sync::RwLock<HashMap<String, SecurityMetrics>>,
    blocked_users: std::sync::RwLock<HashMap<String, DateTime<Utc>>>,
}

impl ThreatDetectionService {
    pub fn new() -> Self {
        Self {
            security_metrics: std::sync::RwLock::new(HashMap::new()),
            blocked_users: std::sync::RwLock::new(HashMap::new()),
        }
    }
    
    /// Analyze security event and determine threat level
    pub async fn analyze_security_event(
        &self,
        event: SecurityEvent,
        context: SecurityContext,
    ) -> Result<ThreatLevel, SecurityError> {
        // Check if user is currently blocked
        self.check_user_blocked(&context.user_id)?;
        
        // Log security event
        self.log_security_event(&event, &context).await?;
        
        // Analyze threat level
        let threat_level = self.calculate_threat_level(&event, &context).await?;
        
        // Take action based on threat level
        self.handle_threat_response(&threat_level, &context).await?;
        
        Ok(threat_level)
    }
    
    /// Check if user is currently blocked
    pub fn check_user_blocked(&self, user_id: &str) -> Result<(), SecurityError> {
        let blocked_users = self.blocked_users.read().unwrap();
        
        if let Some(blocked_until) = blocked_users.get(user_id) {
            if Utc::now() < *blocked_until {
                return Err(SecurityError::BlockedUser(format!(
                    "User {} blocked until {}", user_id, blocked_until
                )));
            }
        }
        
        Ok(())
    }
    
    /// Log security event for audit trail
    async fn log_security_event(
        &self,
        event: &SecurityEvent,
        context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        // Update security metrics
        let mut metrics = self.security_metrics.write().unwrap();
        let user_metrics = metrics.entry(context.user_id.clone())
            .or_insert_with(|| SecurityMetrics {
                failed_attempts: 0,
                last_failure: Utc::now(),
                threat_level: 1,
                blocked_until: None,
            });
        
        match event {
            SecurityEvent::InvalidJwtAttempt |
            SecurityEvent::PermissionIntegrityViolation |
            SecurityEvent::DeviceBindingViolation => {
                user_metrics.failed_attempts += 1;
                user_metrics.last_failure = Utc::now();
            },
            _ => {}
        }
        
        // Log to structured logging
        match event {
            SecurityEvent::InvalidJwtAttempt => {
                warn!(
                    user_id = %context.user_id,
                    ip = ?context.ip_address,
                    "Invalid JWT token attempt detected"
                );
            },
            SecurityEvent::PermissionIntegrityViolation => {
                error!(
                    user_id = %context.user_id,
                    ip = ?context.ip_address,
                    "Permission integrity violation - possible token tampering"
                );
            },
            SecurityEvent::DeviceBindingViolation => {
                warn!(
                    user_id = %context.user_id,
                    ip = ?context.ip_address,
                    device_fingerprint = ?context.device_fingerprint,
                    "Device binding violation - token used from different device"
                );
            },
            SecurityEvent::UnauthorizedPermissionAccess => {
                warn!(
                    user_id = %context.user_id,
                    path = ?context.request_path,
                    "Unauthorized permission access attempt"
                );
            },
            _ => {
                info!(
                    event = ?event,
                    user_id = %context.user_id,
                    "Security event recorded"
                );
            }
        }
        
        Ok(())
    }
    
    /// Calculate threat level based on event and context
    async fn calculate_threat_level(
        &self,
        event: &SecurityEvent,
        context: &SecurityContext,
    ) -> Result<ThreatLevel, SecurityError> {
        let metrics = self.security_metrics.read().unwrap();
        let user_metrics = metrics.get(&context.user_id);
        
        let base_threat_level = match event {
            SecurityEvent::InvalidJwtAttempt => ThreatLevel::Low,
            SecurityEvent::DeviceBindingViolation => ThreatLevel::Medium,
            SecurityEvent::PermissionIntegrityViolation => ThreatLevel::Critical,
            SecurityEvent::TokenReplayAttack => ThreatLevel::High,
            SecurityEvent::UnauthorizedPermissionAccess => ThreatLevel::Medium,
            SecurityEvent::SuspiciousRefreshPattern => ThreatLevel::Medium,
            SecurityEvent::MultipleFailedAttempts => ThreatLevel::High,
            SecurityEvent::AnomalousPermissionRequest => ThreatLevel::Low,
        };
        
        // Escalate based on user history
        let escalated_threat = if let Some(metrics) = user_metrics {
            match metrics.failed_attempts {
                0..=2 => base_threat_level,
                3..=5 => std::cmp::max(base_threat_level, ThreatLevel::Medium),
                6..=10 => std::cmp::max(base_threat_level, ThreatLevel::High),
                _ => ThreatLevel::Critical,
            }
        } else {
            base_threat_level
        };
        
        Ok(escalated_threat)
    }
    
    /// Handle threat response based on threat level
    async fn handle_threat_response(
        &self,
        threat_level: &ThreatLevel,
        context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        match threat_level {
            ThreatLevel::Low => {
                // Just log, no action needed
                info!(
                    user_id = %context.user_id,
                    "Low threat level - monitoring"
                );
            },
            ThreatLevel::Medium => {
                warn!(
                    user_id = %context.user_id,
                    "Medium threat level - increased monitoring"
                );
                // Could implement rate limiting here
            },
            ThreatLevel::High => {
                error!(
                    user_id = %context.user_id,
                    "High threat level - temporary restrictions applied"
                );
                self.apply_temporary_restrictions(&context.user_id, Duration::minutes(15)).await?;
            },
            ThreatLevel::Critical => {
                error!(
                    user_id = %context.user_id,
                    "Critical threat level - user blocked"
                );
                self.block_user(&context.user_id, Duration::hours(1)).await?;
                // In production: notify security team, trigger incident response
            },
        }
        
        Ok(())
    }
    
    /// Apply temporary restrictions to user
    async fn apply_temporary_restrictions(
        &self,
        user_id: &str,
        duration: Duration,
    ) -> Result<(), SecurityError> {
        let blocked_until = Utc::now() + duration;
        let mut blocked_users = self.blocked_users.write().unwrap();
        blocked_users.insert(user_id.to_string(), blocked_until);
        
        warn!(
            user_id = %user_id,
            blocked_until = %blocked_until,
            "User temporarily restricted due to security concerns"
        );
        
        Ok(())
    }
    
    /// Block user for security violation
    async fn block_user(&self, user_id: &str, duration: Duration) -> Result<(), SecurityError> {
        let blocked_until = Utc::now() + duration;
        let mut blocked_users = self.blocked_users.write().unwrap();
        blocked_users.insert(user_id.to_string(), blocked_until);
        
        error!(
            user_id = %user_id,
            blocked_until = %blocked_until,
            "User blocked due to critical security violation"
        );
        
        // In production: send alert to security team
        // self.send_security_alert(user_id, "User blocked due to critical security violation").await?;
        
        Ok(())
    }
    
    /// Get security summary for user
    pub fn get_security_summary(&self, user_id: &str) -> Option<SecuritySummary> {
        let metrics = self.security_metrics.read().unwrap();
        let blocked_users = self.blocked_users.read().unwrap();
        
        let user_metrics = metrics.get(user_id)?;
        let blocked_until = blocked_users.get(user_id).copied();
        
        Some(SecuritySummary {
            user_id: user_id.to_string(),
            failed_attempts: user_metrics.failed_attempts,
            last_failure: user_metrics.last_failure,
            threat_level: match user_metrics.threat_level {
                1 => ThreatLevel::Low,
                2 => ThreatLevel::Medium,
                3 => ThreatLevel::High,
                4 => ThreatLevel::Critical,
                _ => ThreatLevel::Low,
            },
            blocked_until,
            is_blocked: blocked_until.map_or(false, |until| Utc::now() < until),
        })
    }
    
    /// Reset security metrics for user (admin function)
    pub fn reset_user_security(&self, user_id: &str) {
        let mut metrics = self.security_metrics.write().unwrap();
        let mut blocked_users = self.blocked_users.write().unwrap();
        
        metrics.remove(user_id);
        blocked_users.remove(user_id);
        
        info!(
            user_id = %user_id,
            "Security metrics reset for user"
        );
    }
    
    /// Clean up expired blocks and old metrics
    pub async fn cleanup_expired_data(&self) {
        let now = Utc::now();
        
        // Remove expired blocks
        let mut blocked_users = self.blocked_users.write().unwrap();
        blocked_users.retain(|_, blocked_until| now < *blocked_until);
        
        // Clean up old metrics (older than 24 hours)
        let mut metrics = self.security_metrics.write().unwrap();
        metrics.retain(|_, user_metrics| {
            now.signed_duration_since(user_metrics.last_failure) < Duration::hours(24)
        });
        
        info!("Cleaned up expired security data");
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SecuritySummary {
    pub user_id: String,
    pub failed_attempts: u32,
    pub last_failure: DateTime<Utc>,
    pub threat_level: ThreatLevel,
    pub blocked_until: Option<DateTime<Utc>>,
    pub is_blocked: bool,
}

// Singleton pattern for global threat detection service
use std::sync::OnceLock;
static THREAT_DETECTION: OnceLock<ThreatDetectionService> = OnceLock::new();

/// Get global threat detection service instance
pub fn get_threat_detection_service() -> &'static ThreatDetectionService {
    THREAT_DETECTION.get_or_init(|| ThreatDetectionService::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_threat_escalation() {
        let service = ThreatDetectionService::new();
        let context = SecurityContext {
            user_id: "test_user".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: None,
            device_fingerprint: None,
            timestamp: Utc::now(),
            request_path: None,
        };
        
        // First attempt should be low threat
        let threat1 = service.analyze_security_event(
            SecurityEvent::InvalidJwtAttempt,
            context.clone(),
        ).await.unwrap();
        assert_eq!(threat1, ThreatLevel::Low);
        
        // Multiple attempts should escalate
        for _ in 0..5 {
            service.analyze_security_event(
                SecurityEvent::InvalidJwtAttempt,
                context.clone(),
            ).await.unwrap();
        }
        
        // Should be higher threat level now
        let final_threat = service.analyze_security_event(
            SecurityEvent::InvalidJwtAttempt,
            context.clone(),
        ).await.unwrap();
        assert!(final_threat >= ThreatLevel::Medium);
    }
    
    #[tokio::test]
    async fn test_user_blocking() {
        let service = ThreatDetectionService::new();
        let context = SecurityContext {
            user_id: "blocked_user".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: None,
            device_fingerprint: None,
            timestamp: Utc::now(),
            request_path: None,
        };
        
        // Trigger critical event
        service.analyze_security_event(
            SecurityEvent::PermissionIntegrityViolation,
            context.clone(),
        ).await.unwrap();
        
        // User should be blocked now
        let result = service.check_user_blocked("blocked_user");
        assert!(result.is_err());
    }
}