// Security hardening for permission system
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::dom::entities::iam::{Permission, PackageTier};
use crate::dom::entities::permission_profile::PermissionProfile;
use crate::dom::values::UserId;

/// Security violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityViolationType {
    PermissionEscalation,
    UnauthorizedAccess,
    SuspiciousActivity,
    BruteForceAttempt,
    RateLimitExceeded,
    InvalidPermissionModification,
    PrivilegeAbuseDetected,
}

/// Security event for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub violation_type: SecurityViolationType,
    pub user_id: UserId,
    pub description: String,
    pub severity: SecuritySeverity,
    pub timestamp: DateTime<Utc>,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub attempted_action: Option<String>,
    pub attempted_resource: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Permission change validation result
#[derive(Debug, Clone)]
pub struct PermissionValidationResult {
    pub is_valid: bool,
    pub violations: Vec<SecurityViolation>,
    pub warnings: Vec<String>,
    pub requires_approval: bool,
}

/// Security violation details
#[derive(Debug, Clone)]
pub struct SecurityViolation {
    pub violation_type: SecurityViolationType,
    pub description: String,
    pub severity: SecuritySeverity,
    pub suggested_action: String,
}

/// User activity tracking for suspicious behavior detection
#[derive(Debug, Clone)]
struct UserActivity {
    user_id: UserId,
    failed_attempts: u32,
    successful_accesses: u32,
    last_failed_attempt: Option<DateTime<Utc>>,
    last_successful_access: Option<DateTime<Utc>>,
    access_patterns: Vec<AccessPattern>,
    suspicious_score: f64,
}

/// Access pattern for behavioral analysis
#[derive(Debug, Clone)]
struct AccessPattern {
    timestamp: DateTime<Utc>,
    resource: String,
    action: String,
    source_ip: Option<String>,
    success: bool,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub suspicious_activity_threshold: f64,
    pub permission_escalation_detection: bool,
    pub require_approval_for_admin_changes: bool,
    pub audit_all_permission_changes: bool,
    pub enable_behavioral_analysis: bool,
    pub rate_limit_window_seconds: u64,
    pub max_requests_per_window: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_failed_attempts: 5,
            lockout_duration_minutes: 15,
            suspicious_activity_threshold: 7.0,
            permission_escalation_detection: true,
            require_approval_for_admin_changes: true,
            audit_all_permission_changes: true,
            enable_behavioral_analysis: true,
            rate_limit_window_seconds: 60,
            max_requests_per_window: 100,
        }
    }
}

/// Main permission security service
pub struct PermissionSecurityService {
    config: SecurityConfig,
    user_activities: Arc<RwLock<HashMap<UserId, UserActivity>>>,
    security_events: Arc<RwLock<Vec<SecurityEvent>>>,
    blocked_users: Arc<RwLock<HashSet<UserId>>>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimitState>>>,
}

/// Rate limiting state
#[derive(Debug, Clone)]
struct RateLimitState {
    requests: u32,
    window_start: DateTime<Utc>,
}

impl PermissionSecurityService {
    pub fn new(config: Option<SecurityConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            user_activities: Arc::new(RwLock::new(HashMap::new())),
            security_events: Arc::new(RwLock::new(Vec::new())),
            blocked_users: Arc::new(RwLock::new(HashSet::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validate permission changes for security violations
    pub async fn validate_permission_change(
        &self,
        user_id: &UserId,
        old_permissions: &[Permission],
        new_permissions: &[Permission],
        requesting_user_id: &UserId,
        requesting_user_tier: &PackageTier,
    ) -> PermissionValidationResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut requires_approval = false;

        // Check for permission escalation
        if self.config.permission_escalation_detection {
            if let Some(escalation) = self.detect_permission_escalation(old_permissions, new_permissions).await {
                violations.push(escalation);
                requires_approval = true;
            }
        }

        // Check if requester has authority to make changes
        if !self.has_authority_for_change(requesting_user_id, requesting_user_tier, new_permissions).await {
            violations.push(SecurityViolation {
                violation_type: SecurityViolationType::UnauthorizedAccess,
                description: "Requesting user lacks authority to grant these permissions".to_string(),
                severity: SecuritySeverity::High,
                suggested_action: "Require approval from higher authority".to_string(),
            });
            requires_approval = true;
        }

        // Check for suspicious patterns
        if self.config.enable_behavioral_analysis {
            if let Some(suspicious) = self.detect_suspicious_permission_patterns(user_id, new_permissions).await {
                warnings.push(suspicious);
            }
        }

        // Admin changes require approval
        if self.config.require_approval_for_admin_changes && self.contains_admin_permissions(new_permissions) {
            requires_approval = true;
            warnings.push("Admin permissions detected - approval required".to_string());
        }

        PermissionValidationResult {
            is_valid: violations.is_empty(),
            violations,
            warnings,
            requires_approval,
        }
    }

    /// Check for permission escalation
    async fn detect_permission_escalation(
        &self,
        old_permissions: &[Permission],
        new_permissions: &[Permission],
    ) -> Option<SecurityViolation> {
        // Check if new permissions are significantly more powerful
        let old_admin_perms = self.count_admin_permissions(old_permissions);
        let new_admin_perms = self.count_admin_permissions(new_permissions);

        if new_admin_perms > old_admin_perms {
            return Some(SecurityViolation {
                violation_type: SecurityViolationType::PermissionEscalation,
                description: format!(
                    "Admin permissions increased from {} to {}",
                    old_admin_perms, new_admin_perms
                ),
                severity: SecuritySeverity::High,
                suggested_action: "Review and approve admin permission grants".to_string(),
            });
        }

        // Check for wildcard permission escalation
        let old_wildcards = self.count_wildcard_permissions(old_permissions);
        let new_wildcards = self.count_wildcard_permissions(new_permissions);

        if new_wildcards > old_wildcards + 2 {
            return Some(SecurityViolation {
                violation_type: SecurityViolationType::PermissionEscalation,
                description: "Significant increase in wildcard permissions detected".to_string(),
                severity: SecuritySeverity::Medium,
                suggested_action: "Review wildcard permission grants".to_string(),
            });
        }

        None
    }

    /// Check if requesting user has authority to make permission changes
    async fn has_authority_for_change(
        &self,
        _requesting_user_id: &UserId,
        requesting_user_tier: &PackageTier,
        new_permissions: &[Permission],
    ) -> bool {
        // SuperAdmin can do anything
        if requesting_user_tier == &PackageTier::SuperAdmin {
            return true;
        }

        // Admin can grant non-admin permissions
        if requesting_user_tier == &PackageTier::Admin {
            return !self.contains_admin_permissions(new_permissions);
        }

        // Regular users cannot grant permissions
        false
    }

    /// Detect suspicious permission patterns
    async fn detect_suspicious_permission_patterns(
        &self,
        user_id: &UserId,
        permissions: &[Permission],
    ) -> Option<String> {
        // Check for unusual permission combinations
        let admin_perms = self.count_admin_permissions(permissions);
        let wildcard_perms = self.count_wildcard_permissions(permissions);

        if admin_perms > 5 && wildcard_perms > 3 {
            return Some(format!(
                "Unusual combination: {} admin permissions and {} wildcard permissions",
                admin_perms, wildcard_perms
            ));
        }

        // Check against user's historical patterns
        let user_activities = self.user_activities.read().await;
        if let Some(activity) = user_activities.get(user_id) {
            if activity.suspicious_score > self.config.suspicious_activity_threshold {
                return Some("User has high suspicious activity score".to_string());
            }
        }

        None
    }

    /// Record security event
    pub async fn record_security_event(&self, event: SecurityEvent) {
        // Log the event
        tracing::warn!(
            "Security event: {:?} - {} for user {}",
            event.violation_type,
            event.description,
            event.user_id
        );

        // Store the event
        let mut events = self.security_events.write().await;
        events.push(event.clone());

        // Update user activity and check for blocking
        self.update_user_activity(&event).await;

        // Trigger alerts for critical events
        if event.severity == SecuritySeverity::Critical {
            self.trigger_security_alert(&event).await;
        }
    }

    /// Update user activity tracking
    async fn update_user_activity(&self, event: &SecurityEvent) {
        let mut activities = self.user_activities.write().await;
        let activity = activities.entry(event.user_id.clone()).or_insert_with(|| UserActivity {
            user_id: event.user_id.clone(),
            failed_attempts: 0,
            successful_accesses: 0,
            last_failed_attempt: None,
            last_successful_access: None,
            access_patterns: Vec::new(),
            suspicious_score: 0.0,
        });

        // Update based on event type
        match event.violation_type {
            SecurityViolationType::UnauthorizedAccess | SecurityViolationType::BruteForceAttempt => {
                activity.failed_attempts += 1;
                activity.last_failed_attempt = Some(event.timestamp);
                activity.suspicious_score += 1.5;
            }
            _ => {
                activity.suspicious_score += 0.5;
            }
        }

        // Add access pattern
        if let (Some(action), Some(resource)) = (&event.attempted_action, &event.attempted_resource) {
            activity.access_patterns.push(AccessPattern {
                timestamp: event.timestamp,
                resource: resource.clone(),
                action: action.clone(),
                source_ip: event.source_ip.clone(),
                success: false,
            });

            // Keep only recent patterns
            let cutoff = Utc::now() - Duration::hours(24);
            activity.access_patterns.retain(|p| p.timestamp > cutoff);
        }

        // Check if user should be blocked
        if activity.failed_attempts >= self.config.max_failed_attempts {
            let mut blocked_users = self.blocked_users.write().await;
            blocked_users.insert(event.user_id.clone());
            
            tracing::warn!("User {} blocked due to {} failed attempts", event.user_id, activity.failed_attempts);
        }
    }

    /// Check if user is currently blocked
    pub async fn is_user_blocked(&self, user_id: &UserId) -> bool {
        let blocked_users = self.blocked_users.read().await;
        blocked_users.contains(user_id)
    }

    /// Unblock user (manual intervention)
    pub async fn unblock_user(&self, user_id: &UserId) {
        let mut blocked_users = self.blocked_users.write().await;
        blocked_users.remove(user_id);

        // Reset user activity
        let mut activities = self.user_activities.write().await;
        if let Some(activity) = activities.get_mut(user_id) {
            activity.failed_attempts = 0;
            activity.suspicious_score = 0.0;
        }

        tracing::info!("User {} unblocked manually", user_id);
    }

    /// Check rate limiting
    pub async fn check_rate_limit(&self, identifier: &str) -> bool {
        let mut rate_limits = self.rate_limits.write().await;
        let now = Utc::now();

        let rate_limit = rate_limits.entry(identifier.to_string()).or_insert_with(|| RateLimitState {
            requests: 0,
            window_start: now,
        });

        // Reset window if expired
        if now - rate_limit.window_start > Duration::seconds(self.config.rate_limit_window_seconds as i64) {
            rate_limit.requests = 0;
            rate_limit.window_start = now;
        }

        rate_limit.requests += 1;

        if rate_limit.requests > self.config.max_requests_per_window {
            tracing::warn!("Rate limit exceeded for {}: {} requests in window", identifier, rate_limit.requests);
            return false;
        }

        true
    }

    /// Trigger security alert
    async fn trigger_security_alert(&self, event: &SecurityEvent) {
        // In a real implementation, this would send alerts to security team
        tracing::error!(
            "SECURITY ALERT: Critical security event - {} for user {} at {}",
            event.description,
            event.user_id,
            event.timestamp
        );

        // Could integrate with external alerting systems here
    }

    /// Count admin permissions
    fn count_admin_permissions(&self, permissions: &[Permission]) -> usize {
        permissions
            .iter()
            .filter(|p| {
                p.action().starts_with("admin:") ||
                p.resource().contains("admin") ||
                p.action().contains("manage") ||
                p.action().contains("delete")
            })
            .count()
    }

    /// Count wildcard permissions
    fn count_wildcard_permissions(&self, permissions: &[Permission]) -> usize {
        permissions
            .iter()
            .filter(|p| p.action().contains('*') || p.resource().contains('*'))
            .count()
    }

    /// Check if permissions contain admin privileges
    fn contains_admin_permissions(&self, permissions: &[Permission]) -> bool {
        self.count_admin_permissions(permissions) > 0
    }

    /// Get security events for user
    pub async fn get_security_events(&self, user_id: &UserId, limit: usize) -> Vec<SecurityEvent> {
        let events = self.security_events.read().await;
        events
            .iter()
            .filter(|e| &e.user_id == user_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get system security summary
    pub async fn get_security_summary(&self) -> SecuritySummary {
        let events = self.security_events.read().await;
        let blocked_users = self.blocked_users.read().await;
        let activities = self.user_activities.read().await;

        let recent_events: Vec<&SecurityEvent> = events
            .iter()
            .filter(|e| e.timestamp > Utc::now() - Duration::hours(24))
            .collect();

        let critical_events = recent_events.iter().filter(|e| e.severity == SecuritySeverity::Critical).count();
        let high_events = recent_events.iter().filter(|e| e.severity == SecuritySeverity::High).count();

        SecuritySummary {
            total_blocked_users: blocked_users.len() as u32,
            events_last_24h: recent_events.len() as u32,
            critical_events_last_24h: critical_events as u32,
            high_severity_events_last_24h: high_events as u32,
            users_with_suspicious_activity: activities.values().filter(|a| a.suspicious_score > 5.0).count() as u32,
            last_updated: Utc::now(),
        }
    }
}

/// Security summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub total_blocked_users: u32,
    pub events_last_24h: u32,
    pub critical_events_last_24h: u32,
    pub high_severity_events_last_24h: u32,
    pub users_with_suspicious_activity: u32,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::entities::iam::Permission;

    #[tokio::test]
    async fn test_permission_security_service_creation() {
        let service = PermissionSecurityService::new(None);
        let summary = service.get_security_summary().await;
        assert_eq!(summary.total_blocked_users, 0);
    }

    #[tokio::test]
    async fn test_permission_escalation_detection() {
        let service = PermissionSecurityService::new(None);
        
        let old_permissions = vec![
            Permission::new("read".to_string(), "posts".to_string()),
        ];
        
        let new_permissions = vec![
            Permission::new("read".to_string(), "posts".to_string()),
            Permission::new("admin:manage".to_string(), "users".to_string()),
        ];
        
        let user_id = UserId::new("test_user".to_string());
        let requesting_user_id = UserId::new("admin_user".to_string());
        
        let result = service.validate_permission_change(
            &user_id,
            &old_permissions,
            &new_permissions,
            &requesting_user_id,
            &PackageTier::Admin,
        ).await;
        
        assert!(!result.violations.is_empty());
        assert!(result.violations.iter().any(|v| matches!(v.violation_type, SecurityViolationType::PermissionEscalation)));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mut config = SecurityConfig::default();
        config.max_requests_per_window = 2;
        config.rate_limit_window_seconds = 60;
        
        let service = PermissionSecurityService::new(Some(config));
        
        // First two requests should succeed
        assert!(service.check_rate_limit("test_user").await);
        assert!(service.check_rate_limit("test_user").await);
        
        // Third request should fail
        assert!(!service.check_rate_limit("test_user").await);
    }

    #[tokio::test]
    async fn test_user_blocking() {
        let service = PermissionSecurityService::new(None);
        let user_id = UserId::new("test_user".to_string());
        
        // User should not be blocked initially
        assert!(!service.is_user_blocked(&user_id).await);
        
        // Record multiple security events
        for i in 0..6 {
            let event = SecurityEvent {
                id: format!("event_{}", i),
                violation_type: SecurityViolationType::UnauthorizedAccess,
                user_id: user_id.clone(),
                description: "Test unauthorized access".to_string(),
                severity: SecuritySeverity::Medium,
                timestamp: Utc::now(),
                source_ip: Some("127.0.0.1".to_string()),
                user_agent: None,
                attempted_action: Some("admin:delete".to_string()),
                attempted_resource: Some("users".to_string()),
                metadata: HashMap::new(),
            };
            
            service.record_security_event(event).await;
        }
        
        // User should be blocked after multiple failures
        assert!(service.is_user_blocked(&user_id).await);
        
        // Unblock user
        service.unblock_user(&user_id).await;
        assert!(!service.is_user_blocked(&user_id).await);
    }
}