use async_trait::async_trait;
use crate::domain::shared_kernel::value_objects::SessionId;
use crate::domain::authentication::AuthenticatedUserId;
use chrono::{DateTime, Utc};
// Session Management Repository Ports
// Defines repository interfaces for Session Management bounded context


use super::{
    UserSessionManager, SessionMetadata, SessionStatus, DeviceInfo
};
use super::value_objects::{IpAddressInfo};

/// Port for session manager repository operations
#[async_trait]
pub trait SessionManagerRepositoryPort: Send + Sync {
    /// Save session manager state
    async fn save_session_manager(&self, manager: &UserSessionManager) -> Result<(), String>;
    
    /// Find session manager for user
    async fn find_session_manager(&self, user_id: &AuthenticatedUserId) -> Result<Option<UserSessionManager>, String>;
    
    /// Delete session manager (removes all user sessions)
    async fn delete_session_manager(&self, user_id: &AuthenticatedUserId) -> Result<(), String>;
    
    /// Get session managers with high risk scores
    async fn find_high_risk_managers(&self, risk_threshold: u8) -> Result<Vec<UserSessionManager>, String>;
}

/// Port for session metadata repository operations
#[async_trait]
pub trait SessionMetadataRepositoryPort: Send + Sync {
    /// Save session metadata
    async fn save_metadata(&self, metadata: &SessionMetadata) -> Result<(), String>;
    
    /// Find metadata by session ID
    async fn find_by_session_id(&self, session_id: &SessionId) -> Result<Option<SessionMetadata>, String>;
    
    /// Find metadata by user ID
    async fn find_by_user_id(&self, user_id: &AuthenticatedUserId) -> Result<Vec<SessionMetadata>, String>;
    
    /// Update session status
    async fn update_status(&self, session_id: &SessionId, status: SessionStatus) -> Result<(), String>;
    
    /// Update last activity
    async fn update_last_activity(&self, session_id: &SessionId, timestamp: DateTime<Utc>) -> Result<(), String>;
    
    /// Delete metadata
    async fn delete_metadata(&self, session_id: &SessionId) -> Result<(), String>;
    
    /// Find expired sessions
    async fn find_expired_sessions(&self, before: DateTime<Utc>) -> Result<Vec<SessionMetadata>, String>;
    
    /// Find sessions by status
    async fn find_by_status(&self, status: SessionStatus) -> Result<Vec<SessionMetadata>, String>;
}

/// Port for session persistence operations
#[async_trait]
pub trait SessionPersistenceServicePort: Send + Sync {
    /// Store session data with TTL
    async fn store_session_data(&self, session_id: &SessionId, data: SessionData, ttl_seconds: u64) -> Result<(), String>;
    
    /// Retrieve session data
    async fn get_session_data(&self, session_id: &SessionId) -> Result<Option<SessionData>, String>;
    
    /// Update session TTL
    async fn extend_session_ttl(&self, session_id: &SessionId, additional_seconds: u64) -> Result<(), String>;
    
    /// Delete session data
    async fn delete_session_data(&self, session_id: &SessionId) -> Result<(), String>;
    
    /// Find sessions expiring soon
    async fn find_expiring_sessions(&self, within_seconds: u64) -> Result<Vec<SessionId>, String>;
    
    /// Cleanup expired session data
    async fn cleanup_expired_sessions(&self) -> Result<u32, String>;
}

/// Port for session activity monitoring
#[async_trait]
pub trait SessionActivityMonitoringPort: Send + Sync {
    /// Record session activity
    async fn record_activity(&self, session_id: &SessionId, activity: ActivityRecord) -> Result<(), String>;
    
    /// Get activity history for session
    async fn get_activity_history(&self, session_id: &SessionId, limit: u32) -> Result<Vec<ActivityRecord>, String>;
    
    /// Detect suspicious activity patterns
    async fn detect_suspicious_patterns(&self, session_id: &SessionId) -> Result<Vec<SuspiciousPattern>, String>;
    
    /// Get session analytics
    async fn get_session_analytics(&self, user_id: &AuthenticatedUserId, 
                                  period_hours: u32) -> Result<SessionAnalytics, String>;
    
    /// Record suspicious activity
    async fn record_suspicious_activity(&self, session_id: &SessionId, 
                                      details: SuspiciousActivityDetails) -> Result<(), String>;
}

/// Port for device information management
#[async_trait]
pub trait DeviceManagementServicePort: Send + Sync {
    /// Register new device
    async fn register_device(&self, user_id: &AuthenticatedUserId, 
                            device_info: DeviceInfo) -> Result<DeviceId, String>;
    
    /// Get device information
    async fn get_device_info(&self, device_id: &DeviceId) -> Result<Option<DeviceInfo>, String>;
    
    /// Update device last seen
    async fn update_device_last_seen(&self, device_id: &DeviceId, 
                                    timestamp: DateTime<Utc>) -> Result<(), String>;
    
    /// Get user devices
    async fn get_user_devices(&self, user_id: &AuthenticatedUserId) -> Result<Vec<DeviceInfo>, String>;
    
    /// Revoke device access
    async fn revoke_device(&self, device_id: &DeviceId) -> Result<(), String>;
    
    /// Detect device anomalies
    async fn detect_device_anomalies(&self, user_id: &AuthenticatedUserId) -> Result<Vec<DeviceAnomaly>, String>;
}

/// Session data for persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: SessionStatus,
    pub device_info: Option<DeviceInfo>,
    pub ip_addresses: Vec<IpAddressInfo>,
    pub security_context: SecurityContext,
    pub access_tokens: Vec<String>,
    pub scopes: Vec<String>,
}

/// Activity record for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActivityRecord {
    pub timestamp: DateTime<Utc>,
    pub activity_type: ActivityType,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub endpoint: Option<String>,
    pub success: bool,
    pub metadata: serde_json::Value,
}

/// Types of session activities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ActivityType {
    Login,
    TokenRefresh,
    ApiCall,
    Logout,
    PermissionCheck,
    SuspiciousAction,
}

/// Suspicious activity patterns
#[derive(Debug, Clone)]
pub struct SuspiciousPattern {
    pub pattern_type: SuspiciousPatternType,
    pub confidence_score: f64,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SuspiciousPatternType {
    UnusualLoginTime,
    UnusualLocation,
    RapidRequests,
    MultipleFailedAttempts,
    UnexpectedUserAgent,
    GeographicalAnomaly,
    BehaviorAnomaly,
}

/// Session analytics data
#[derive(Debug, Clone)]
pub struct SessionAnalytics {
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub average_session_duration: chrono::Duration,
    pub unique_devices: u32,
    pub unique_ip_addresses: u32,
    pub suspicious_activity_count: u32,
    pub most_active_hours: Vec<u8>,
    pub geographic_distribution: std::collections::HashMap<String, u32>,
}

/// Suspicious activity details
#[derive(Debug, Clone)]
pub struct SuspiciousActivityDetails {
    pub activity_type: SuspiciousActivityType,
    pub description: String,
    pub severity: SuspiciousSeverity,
    pub evidence: serde_json::Value,
    pub auto_response_taken: Option<AutoResponse>,
}

#[derive(Debug, Clone)]
pub enum SuspiciousActivityType {
    UnauthorizedAccess,
    TokenMisuse,
    UnusualBehavior,
    SecurityViolation,
    RateLimitAbuse,
    SessionHijacking,
}

#[derive(Debug, Clone)]
pub enum SuspiciousSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum AutoResponse {
    LoggedOnly,
    SessionSuspended,
    SessionTerminated,
    UserBlocked,
    IpBlocked,
}

/// Device identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Device anomaly detection
#[derive(Debug, Clone)]
pub struct DeviceAnomaly {
    pub device_id: DeviceId,
    pub anomaly_type: DeviceAnomalyType,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub severity: AnomalySeverity,
}

#[derive(Debug, Clone)]
pub enum DeviceAnomalyType {
    NewDevice,
    UnusualLocation,
    UnexpectedFingerprint,
    SuspiciousAgent,
    CompromisedDevice,
}

#[derive(Debug, Clone)]
pub enum AnomalySeverity {
    Info,
    Warning,
    Alert,
    Critical,
}

/// Security context for sessions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityContext {
    pub risk_score: u8,
    pub trust_level: TrustLevel,
    pub security_flags: Vec<SecurityFlag>,
    pub last_security_check: DateTime<Utc>,
    pub mfa_verified: bool,
    pub ip_reputation: IpReputation,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TrustLevel {
    Untrusted,
    Low,
    Medium,
    High,
    Verified,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SecurityFlag {
    SuspiciousLogin,
    NewDevice,
    LocationAnomaly,
    TimeAnomaly,
    BehaviorAnomaly,
    PotentialBot,
    HighRisk,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum IpReputation {
    Clean,
    Suspicious,
    Malicious,
    Tor,
    VPN,
    Proxy,
    Unknown,
}