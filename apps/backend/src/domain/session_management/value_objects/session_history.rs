use crate::domain::authentication::AuthenticatedUserId;
use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::shared_kernel::value_objects::SessionId;
use chrono::{DateTime, Utc};// Session History Value Object
// Maintains historical record of session lifecycle events

use serde::{Serialize, Deserialize};
use std::collections::HashMap;


/// Complete session history for audit and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistory {
    /// Session this history belongs to
    pub session_id: SessionId,
    pub user_id: AuthenticatedUserId,
    
    /// Historical events in chronological order
    pub events: Vec<HistoryEntry>,
    
    /// History metadata
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl SessionHistory {
    /// Create new session history
    pub fn new(session_id: SessionId, user_id: AuthenticatedUserId) -> Self {
        let now = Utc::now();
        
        Self {
            session_id,
            user_id,
            events: vec![],
            created_at: now,
            last_updated: now,
        }
    }
    
    /// Add history event
    pub fn add_event(&mut self, event_type: HistoryEventType, details: Option<String>) {
        let event = HistoryEntry {
            event_type,
            timestamp: Utc::now(),
            details,
            metadata: HistoryMetadata::default(),
        };
        
        self.events.push(event);
        self.last_updated = Utc::now();
    }
    
    /// Add event with metadata
    pub fn add_event_with_metadata(
        &mut self, 
        event_type: HistoryEventType, 
        details: Option<String>,
        metadata: HistoryMetadata
    ) {
        let event = HistoryEntry {
            event_type,
            timestamp: Utc::now(),
            details,
            metadata,
        };
        
        self.events.push(event);
        self.last_updated = Utc::now();
    }
    
    /// Get events of specific type
    pub fn get_events_by_type(&self, event_type: &HistoryEventType) -> Vec<&HistoryEntry> {
        self.events
            .iter()
            .filter(|event| &event.event_type == event_type)
            .collect()
    }
    
    /// Get events in date range
    pub fn get_events_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&HistoryEntry> {
        self.events
            .iter()
            .filter(|event| event.timestamp >= start && event.timestamp <= end)
            .collect()
    }
    
    /// Get most recent event
    pub fn get_latest_event(&self) -> Option<&HistoryEntry> {
        self.events.last()
    }
    
    /// Get first event
    pub fn get_first_event(&self) -> Option<&HistoryEntry> {
        self.events.first()
    }
    
    /// Get session lifecycle summary
    pub fn get_lifecycle_summary(&self) -> SessionLifecycleSummary {
        let created_event = self.events.iter()
            .find(|e| matches!(e.event_type, HistoryEventType::SessionCreated));
            
        let terminated_event = self.events.iter()
            .find(|e| matches!(e.event_type, HistoryEventType::SessionTerminated { .. }));
            
        let token_refreshes = self.events.iter()
            .filter(|e| matches!(e.event_type, HistoryEventType::TokenRefreshed))
            .count();
            
        let security_events = self.events.iter()
            .filter(|e| matches!(e.event_type, 
                HistoryEventType::SecurityAlert { .. } |
                HistoryEventType::SuspiciousActivityDetected { .. } |
                HistoryEventType::AuthenticationFailed { .. }
            ))
            .count();
            
        let authentication_events = self.events.iter()
            .filter(|e| matches!(e.event_type, 
                HistoryEventType::AuthenticationSucceeded { .. } |
                HistoryEventType::AuthenticationFailed { .. }
            ))
            .count();
            
        SessionLifecycleSummary {
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            created_at: created_event.map(|e| e.timestamp),
            terminated_at: terminated_event.map(|e| e.timestamp),
            total_events: self.events.len(),
            token_refreshes,
            security_events,
            authentication_events,
            session_duration_minutes: self.calculate_session_duration_minutes(),
            final_status: self.determine_final_status(),
        }
    }
    
    /// Calculate total session duration in minutes
    fn calculate_session_duration_minutes(&self) -> Option<i64> {
        let first = self.get_first_event()?;
        let last = self.get_latest_event()?;
        
        Some((last.timestamp - first.timestamp).num_minutes())
    }
    
    /// Determine final session status from events
    fn determine_final_status(&self) -> SessionFinalStatus {
        if let Some(latest) = self.get_latest_event() {
            match &latest.event_type {
                HistoryEventType::SessionTerminated { reason } => {
                    match reason.as_str() {
                        "user_logout" => SessionFinalStatus::UserLogout,
                        "admin_termination" => SessionFinalStatus::AdminTerminated,
                        "security_threat" => SessionFinalStatus::SecurityTerminated,
                        "session_expired" => SessionFinalStatus::Expired,
                        _ => SessionFinalStatus::Terminated,
                    }
                },
                HistoryEventType::SessionExpired => SessionFinalStatus::Expired,
                _ => {
                    // Check if there are any security issues
                    if self.events.iter().any(|e| matches!(e.event_type, 
                        HistoryEventType::SecurityAlert { .. } |
                        HistoryEventType::SuspiciousActivityDetected { .. }
                    )) {
                        SessionFinalStatus::SuspiciousActivity
                    } else {
                        SessionFinalStatus::Active
                    }
                }
            }
        } else {
            SessionFinalStatus::Unknown
        }
    }
    
    /// Generate audit trail
    pub fn generate_audit_trail(&self) -> AuditTrail {
        let mut audit_events = vec![];
        
        for event in &self.events {
            audit_events.push(AuditEvent {
                timestamp: event.timestamp,
                event_description: event.event_type.description(),
                details: event.details.clone(),
                severity: event.event_type.severity(),
                category: event.event_type.category(),
            });
        }
        
        AuditTrail {
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            events: audit_events,
            generated_at: Utc::now(),
        }
    }
}

/// Individual history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub event_type: HistoryEventType,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
    pub metadata: HistoryMetadata,
}

/// Types of events that can be recorded in session history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryEventType {
    /// Session lifecycle
    SessionCreated,
    SessionExpired,
    SessionTerminated { reason: String },
    SessionSuspended { reason: String },
    
    /// Authentication events
    AuthenticationSucceeded { provider: String },
    AuthenticationFailed { reason: String },
    TokensIssued { scopes: Vec<String> },
    TokenRefreshed,
    TokenRevoked { reason: String },
    
    /// Security events
    SecurityAlert { alert_type: String },
    SuspiciousActivityDetected { activity: String },
    IpAddressChanged { old_ip: String, new_ip: String },
    DeviceChanged { device_fingerprint: String },
    
    /// Permission events
    PermissionsGranted { permissions: Vec<String> },
    PermissionsRevoked { permissions: Vec<String> },
    
    /// Activity events
    DataAccessed { resource: String },
    ApiCallMade { endpoint: String },
    AdminActionPerformed { action: String },
    
    /// System events
    SecurityCheckPassed,
    SecurityCheckFailed { reason: String },
    RateLimitHit { limit_type: String },
    
    /// Custom events
    Custom { event_name: String },
}

impl HistoryEventType {
    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            HistoryEventType::SessionCreated => "Session created".to_string(),
            HistoryEventType::SessionExpired => "Session expired".to_string(),
            HistoryEventType::SessionTerminated { reason } => format!("Session terminated: {}", reason),
            HistoryEventType::SessionSuspended { reason } => format!("Session suspended: {}", reason),
            HistoryEventType::AuthenticationSucceeded { provider } => format!("Authentication succeeded via {}", provider),
            HistoryEventType::AuthenticationFailed { reason } => format!("Authentication failed: {}", reason),
            HistoryEventType::TokensIssued { scopes } => format!("Tokens issued with scopes: {}", scopes.join(", ")),
            HistoryEventType::TokenRefreshed => "Tokens refreshed".to_string(),
            HistoryEventType::TokenRevoked { reason } => format!("Tokens revoked: {}", reason),
            HistoryEventType::SecurityAlert { alert_type } => format!("Security alert: {}", alert_type),
            HistoryEventType::SuspiciousActivityDetected { activity } => format!("Suspicious activity detected: {}", activity),
            HistoryEventType::IpAddressChanged { old_ip, new_ip } => format!("IP address changed from {} to {}", old_ip, new_ip),
            HistoryEventType::DeviceChanged { device_fingerprint } => format!("Device changed: {}", device_fingerprint),
            HistoryEventType::PermissionsGranted { permissions } => format!("Permissions granted: {}", permissions.join(", ")),
            HistoryEventType::PermissionsRevoked { permissions } => format!("Permissions revoked: {}", permissions.join(", ")),
            HistoryEventType::DataAccessed { resource } => format!("Data accessed: {}", resource),
            HistoryEventType::ApiCallMade { endpoint } => format!("API call made to: {}", endpoint),
            HistoryEventType::AdminActionPerformed { action } => format!("Admin action performed: {}", action),
            HistoryEventType::SecurityCheckPassed => "Security check passed".to_string(),
            HistoryEventType::SecurityCheckFailed { reason } => format!("Security check failed: {}", reason),
            HistoryEventType::RateLimitHit { limit_type } => format!("Rate limit hit: {}", limit_type),
            HistoryEventType::Custom { event_name } => format!("Custom event: {}", event_name),
        }
    }
    
    /// Get event severity level
    pub fn severity(&self) -> EventSeverity {
        match self {
            HistoryEventType::SecurityAlert { .. } => EventSeverity::High,
            HistoryEventType::SuspiciousActivityDetected { .. } => EventSeverity::High,
            HistoryEventType::AuthenticationFailed { .. } => EventSeverity::Medium,
            HistoryEventType::SessionTerminated { .. } => EventSeverity::Medium,
            HistoryEventType::TokenRevoked { .. } => EventSeverity::Medium,
            HistoryEventType::PermissionsRevoked { .. } => EventSeverity::Medium,
            HistoryEventType::SecurityCheckFailed { .. } => EventSeverity::Medium,
            HistoryEventType::RateLimitHit { .. } => EventSeverity::Low,
            _ => EventSeverity::Info,
        }
    }
    
    /// Get event category
    pub fn category(&self) -> EventCategory {
        match self {
            HistoryEventType::SessionCreated | HistoryEventType::SessionExpired | 
            HistoryEventType::SessionTerminated { .. } | HistoryEventType::SessionSuspended { .. } => EventCategory::Session,
            
            HistoryEventType::AuthenticationSucceeded { .. } | HistoryEventType::AuthenticationFailed { .. } |
            HistoryEventType::TokensIssued { .. } | HistoryEventType::TokenRefreshed | 
            HistoryEventType::TokenRevoked { .. } => EventCategory::Authentication,
            
            HistoryEventType::SecurityAlert { .. } | HistoryEventType::SuspiciousActivityDetected { .. } |
            HistoryEventType::SecurityCheckPassed | HistoryEventType::SecurityCheckFailed { .. } => EventCategory::Security,
            
            HistoryEventType::PermissionsGranted { .. } | HistoryEventType::PermissionsRevoked { .. } => EventCategory::Authorization,
            
            HistoryEventType::DataAccessed { .. } | HistoryEventType::ApiCallMade { .. } => EventCategory::Activity,
            
            _ => EventCategory::System,
        }
    }
}

/// Event severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Event categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    Session,
    Authentication,
    Security,
    Authorization,
    Activity,
    System,
}

/// History event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMetadata {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub source_system: Option<String>,
    pub correlation_id: Option<String>,
    pub additional_data: Option<serde_json::Value>,
}

impl Default for HistoryMetadata {
    fn default() -> Self {
        Self {
            ip_address: None,
            user_agent: None,
            source_system: None,
            correlation_id: None,
            additional_data: None,
        }
    }
}

/// Session lifecycle summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLifecycleSummary {
    pub session_id: SessionId,
    pub user_id: AuthenticatedUserId,
    pub created_at: Option<DateTime<Utc>>,
    pub terminated_at: Option<DateTime<Utc>>,
    pub total_events: usize,
    pub token_refreshes: usize,
    pub security_events: usize,
    pub authentication_events: usize,
    pub session_duration_minutes: Option<i64>,
    pub final_status: SessionFinalStatus,
}

/// Final status of a session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionFinalStatus {
    Active,
    UserLogout,
    AdminTerminated,
    SecurityTerminated,
    Expired,
    SuspiciousActivity,
    Terminated,
    Unknown,
}

/// Audit trail for compliance and investigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub session_id: SessionId,
    pub user_id: AuthenticatedUserId,
    pub events: Vec<AuditEvent>,
    pub generated_at: DateTime<Utc>,
}

/// Individual audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_description: String,
    pub details: Option<String>,
    pub severity: EventSeverity,
    pub category: EventCategory,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_management::value_objects::UserId;
    
    #[test]
    fn create_session_history() {
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let history = SessionHistory::new(session_id.clone(), user_id.clone());
        
        assert_eq!(history.session_id, session_id);
        assert_eq!(history.user_id, user_id);
        assert_eq!(history.events.len(), 0);
    }
    
    #[test]
    fn add_history_events() {
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut history = SessionHistory::new(session_id, user_id);
        
        history.add_event(HistoryEventType::SessionCreated, None);
        history.add_event(HistoryEventType::AuthenticationSucceeded { provider: "Firebase".to_string() }, None);
        history.add_event(HistoryEventType::TokensIssued { scopes: vec!["openid".to_string(), "profile".to_string()] }, None);
        
        assert_eq!(history.events.len(), 3);
        
        let auth_events = history.get_events_by_type(&HistoryEventType::AuthenticationSucceeded { provider: "Firebase".to_string() });
        assert_eq!(auth_events.len(), 1);
    }
    
    #[test]
    fn lifecycle_summary() {
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut history = SessionHistory::new(session_id.clone(), user_id.clone());
        
        // Add session lifecycle events
        history.add_event(HistoryEventType::SessionCreated, None);
        history.add_event(HistoryEventType::AuthenticationSucceeded { provider: "Firebase".to_string() }, None);
        history.add_event(HistoryEventType::TokenRefreshed, None);
        history.add_event(HistoryEventType::SecurityAlert { alert_type: "suspicious_ip".to_string() }, None);
        history.add_event(HistoryEventType::SessionTerminated { reason: "user_logout".to_string() }, None);
        
        let summary = history.get_lifecycle_summary();
        
        assert_eq!(summary.session_id, session_id);
        assert_eq!(summary.total_events, 5);
        assert_eq!(summary.token_refreshes, 1);
        assert_eq!(summary.security_events, 1);
        assert_eq!(summary.authentication_events, 1);
        assert_eq!(summary.final_status, SessionFinalStatus::UserLogout);
    }
    
    #[test]
    fn audit_trail_generation() {
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let mut history = SessionHistory::new(session_id.clone(), user_id);
        
        history.add_event(HistoryEventType::SessionCreated, Some("Initial login".to_string()));
        history.add_event(HistoryEventType::SecurityAlert { alert_type: "multiple_failures".to_string() }, None);
        
        let audit_trail = history.generate_audit_trail();
        
        assert_eq!(audit_trail.session_id, session_id);
        assert_eq!(audit_trail.events.len(), 2);
        
        // Check severity levels
        assert_eq!(audit_trail.events[0].severity, EventSeverity::Info);
        assert_eq!(audit_trail.events[1].severity, EventSeverity::High);
        
        // Check categories
        assert_eq!(audit_trail.events[0].category, EventCategory::Session);
        assert_eq!(audit_trail.events[1].category, EventCategory::Security);
    }
}