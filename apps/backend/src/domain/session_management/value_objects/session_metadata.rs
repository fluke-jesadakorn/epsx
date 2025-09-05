// Session Metadata Value Object
// Contains session persistence and lifecycle information

use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::domain::authentication::{SessionId, AuthenticatedUserId, ProviderType};
use super::DeviceInfo;

/// Complete session metadata for persistence and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Core session identification
    pub session_id: SessionId,
    pub user_id: AuthenticatedUserId,
    
    /// Session lifecycle
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub terminated_at: Option<DateTime<Utc>>,
    
    /// Authentication context
    pub provider_type: ProviderType,
    pub authentication_method: String,
    pub initial_scopes: Vec<String>,
    
    /// Device and client information
    pub device_info: Option<DeviceInfo>,
    pub ip_addresses: Vec<IpAddressInfo>,
    
    /// Session properties
    pub is_persistent: bool,
    pub remember_me: bool,
    pub session_data: HashMap<String, String>,
    
    /// Security tracking
    pub security_flags: Vec<String>,
    pub suspicious_activity_count: u32,
    pub last_security_check: DateTime<Utc>,
    
    /// Performance metrics
    pub access_count: u64,
    pub data_transfer_kb: u64,
    pub api_calls_count: u64,
}

impl SessionMetadata {
    /// Create new session metadata
    pub fn new(
        session_id: SessionId,
        user_id: AuthenticatedUserId,
        provider_type: ProviderType,
        expires_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            session_id,
            user_id,
            status: SessionStatus::Active,
            created_at: now,
            last_accessed: now,
            expires_at,
            terminated_at: None,
            provider_type,
            authentication_method: "OIDC".to_string(),
            initial_scopes: vec![],
            device_info: None,
            ip_addresses: vec![],
            is_persistent: false,
            remember_me: false,
            session_data: HashMap::new(),
            security_flags: vec![],
            suspicious_activity_count: 0,
            last_security_check: now,
            access_count: 1,
            data_transfer_kb: 0,
            api_calls_count: 0,
        }
    }
    
    /// Check if session is currently active
    pub fn is_active(&self) -> bool {
        matches!(self.status, SessionStatus::Active) && 
        Utc::now() < self.expires_at &&
        self.terminated_at.is_none()
    }
    
    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at || matches!(self.status, SessionStatus::Expired)
    }
    
    /// Check if session is terminated
    pub fn is_terminated(&self) -> bool {
        self.terminated_at.is_some() || matches!(self.status, SessionStatus::Terminated)
    }
    
    /// Update last accessed time and increment access count
    pub fn record_access(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
        
        // Keep session alive if it's close to expiring (sliding expiration)
        let time_until_expiry = self.expires_at - Utc::now();
        if time_until_expiry < Duration::minutes(30) && self.is_persistent {
            self.expires_at = Utc::now() + Duration::hours(24);
        }
    }
    
    /// Add IP address to tracking
    pub fn add_ip_address(&mut self, ip: String, location: Option<String>) {
        let ip_info = IpAddressInfo {
            address: ip.clone(),
            location,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            access_count: 1,
        };
        
        // Check if IP already exists
        if let Some(existing) = self.ip_addresses.iter_mut().find(|info| info.address == ip) {
            existing.last_seen = Utc::now();
            existing.access_count += 1;
        } else {
            self.ip_addresses.push(ip_info);
        }
    }
    
    /// Update device information
    pub fn update_device_info(&mut self, device_info: DeviceInfo) {
        self.device_info = Some(device_info);
    }
    
    /// Add security flag
    pub fn add_security_flag(&mut self, flag: String) {
        if !self.security_flags.contains(&flag) {
            self.security_flags.push(flag);
        }
        self.last_security_check = Utc::now();
    }
    
    /// Record suspicious activity
    pub fn record_suspicious_activity(&mut self, description: String) {
        self.suspicious_activity_count += 1;
        self.add_security_flag(format!("suspicious:{}", description));
        
        // Auto-expire if too many suspicious activities
        if self.suspicious_activity_count >= 5 {
            self.status = SessionStatus::Suspicious;
        }
    }
    
    /// Terminate session with reason
    pub fn terminate(&mut self, reason: String) {
        self.status = SessionStatus::Terminated;
        self.terminated_at = Some(Utc::now());
        self.add_security_flag(format!("terminated:{}", reason));
    }
    
    /// Mark session as expired
    pub fn mark_expired(&mut self) {
        self.status = SessionStatus::Expired;
    }
    
    /// Get session duration in minutes
    pub fn session_duration_minutes(&self) -> i64 {
        let end_time = self.terminated_at.unwrap_or(Utc::now());
        (end_time - self.created_at).num_minutes()
    }
    
    /// Get session age in hours
    pub fn session_age_hours(&self) -> i64 {
        (Utc::now() - self.created_at).num_hours()
    }
    
    /// Check if session is suspicious
    pub fn is_suspicious(&self) -> bool {
        matches!(self.status, SessionStatus::Suspicious) ||
        self.suspicious_activity_count >= 3 ||
        self.ip_addresses.len() > 5 || // Too many different IPs
        self.session_age_hours() > 72 // Very long sessions
    }
    
    /// Get session summary for logging
    pub fn summary(&self) -> SessionSummary {
        SessionSummary {
            session_id: self.session_id.clone(),
            user_id: self.user_id.clone(),
            status: self.status.clone(),
            created_at: self.created_at,
            last_accessed: self.last_accessed,
            expires_at: self.expires_at,
            duration_minutes: self.session_duration_minutes(),
            access_count: self.access_count,
            ip_count: self.ip_addresses.len() as u32,
            is_suspicious: self.is_suspicious(),
            provider_type: self.provider_type.clone(),
        }
    }
}

/// Session status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is active and valid
    Active,
    /// Session has expired naturally
    Expired,
    /// Session was terminated by user/admin/system
    Terminated,
    /// Session is flagged as suspicious
    Suspicious,
    /// Session is temporarily suspended
    Suspended,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Expired => write!(f, "expired"),
            SessionStatus::Terminated => write!(f, "terminated"),
            SessionStatus::Suspicious => write!(f, "suspicious"),
            SessionStatus::Suspended => write!(f, "suspended"),
        }
    }
}

/// IP address information for session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddressInfo {
    pub address: String,
    pub location: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub access_count: u32,
}

/// Session summary for monitoring and logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub user_id: AuthenticatedUserId,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub duration_minutes: i64,
    pub access_count: u64,
    pub ip_count: u32,
    pub is_suspicious: bool,
    pub provider_type: ProviderType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_management::value_objects::UserId;
    
    #[test]
    fn create_session_metadata() {
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let expires_at = Utc::now() + Duration::hours(8);
        
        let metadata = SessionMetadata::new(
            session_id.clone(),
            user_id.clone(),
            ProviderType::Firebase,
            expires_at,
        );
        
        assert_eq!(metadata.session_id, session_id);
        assert_eq!(metadata.user_id, user_id);
        assert_eq!(metadata.status, SessionStatus::Active);
        assert!(metadata.is_active());
        assert!(!metadata.is_expired());
    }
    
    #[test]
    fn session_access_tracking() {
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let expires_at = Utc::now() + Duration::hours(8);
        
        let mut metadata = SessionMetadata::new(session_id, user_id, ProviderType::Firebase, expires_at);
        
        let initial_access_count = metadata.access_count;
        metadata.record_access();
        
        assert_eq!(metadata.access_count, initial_access_count + 1);
    }
    
    #[test]
    fn ip_address_tracking() {
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let expires_at = Utc::now() + Duration::hours(8);
        
        let mut metadata = SessionMetadata::new(session_id, user_id, ProviderType::Firebase, expires_at);
        
        metadata.add_ip_address("192.168.1.1".to_string(), Some("New York".to_string()));
        metadata.add_ip_address("192.168.1.1".to_string(), Some("New York".to_string())); // Same IP
        metadata.add_ip_address("10.0.0.1".to_string(), None); // Different IP
        
        assert_eq!(metadata.ip_addresses.len(), 2);
        assert_eq!(metadata.ip_addresses[0].access_count, 2); // Same IP accessed twice
        assert_eq!(metadata.ip_addresses[1].access_count, 1); // New IP accessed once
    }
    
    #[test]
    fn suspicious_activity_tracking() {
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let expires_at = Utc::now() + Duration::hours(8);
        
        let mut metadata = SessionMetadata::new(session_id, user_id, ProviderType::Firebase, expires_at);
        
        // Record multiple suspicious activities
        for i in 0..6 {
            metadata.record_suspicious_activity(format!("activity_{}", i));
        }
        
        assert_eq!(metadata.suspicious_activity_count, 6);
        assert_eq!(metadata.status, SessionStatus::Suspicious);
        assert!(metadata.is_suspicious());
    }
    
    #[test]
    fn session_termination() {
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(UserId::new().unwrap());
        let expires_at = Utc::now() + Duration::hours(8);
        
        let mut metadata = SessionMetadata::new(session_id, user_id, ProviderType::Firebase, expires_at);
        
        assert!(metadata.is_active());
        
        metadata.terminate("user_logout".to_string());
        
        assert!(!metadata.is_active());
        assert!(metadata.is_terminated());
        assert!(metadata.terminated_at.is_some());
        assert_eq!(metadata.status, SessionStatus::Terminated);
    }
}