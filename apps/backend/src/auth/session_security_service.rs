use std::sync::Arc;
use std::net::IpAddr;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use tracing::info;

use sha2::{Sha256, Digest};

use base64::{engine::general_purpose, Engine};


use crate::core::errors::AppResult;


/// Enhanced device information with fingerprinting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub user_agent: Option<String>,
    pub screen_resolution: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub platform: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub device_type: Option<String>, // mobile, desktop, tablet
    pub fingerprint_hash: Option<String>, // Derived hash of all components
    pub canvas_fingerprint: Option<String>,
    pub webgl_fingerprint: Option<String>,
}

impl DeviceFingerprint {
    /// Generate a hash fingerprint from device characteristics
    pub fn generate_hash(&mut self) {
        let mut hasher = Sha256::new();
        
        // Add all available device characteristics to hash
        if let Some(ref ua) = self.user_agent {
            hasher.update(ua.as_bytes());
        }
        if let Some(ref screen) = self.screen_resolution {
            hasher.update(screen.as_bytes());
        }
        if let Some(ref tz) = self.timezone {
            hasher.update(tz.as_bytes());
        }
        if let Some(ref lang) = self.language {
            hasher.update(lang.as_bytes());
        }
        if let Some(ref platform) = self.platform {
            hasher.update(platform.as_bytes());
        }
        if let Some(ref canvas) = self.canvas_fingerprint {
            hasher.update(canvas.as_bytes());
        }
        if let Some(ref webgl) = self.webgl_fingerprint {
            hasher.update(webgl.as_bytes());
        }
        
        let hash = hasher.finalize();
        self.fingerprint_hash = Some(general_purpose::STANDARD.encode(hash));
    }

    /// Check if this fingerprint matches another (with tolerance)
    pub fn matches(&self, other: &DeviceFingerprint, tolerance: f32) -> bool {
        let mut matches = 0;
        let mut total_checks = 0;

        // Compare each available field
        if self.user_agent.is_some() && other.user_agent.is_some() {
            total_checks += 1;
            if self.user_agent == other.user_agent {
                matches += 1;
            }
        }

        if self.screen_resolution.is_some() && other.screen_resolution.is_some() {
            total_checks += 1;
            if self.screen_resolution == other.screen_resolution {
                matches += 1;
            }
        }

        if self.timezone.is_some() && other.timezone.is_some() {
            total_checks += 1;
            if self.timezone == other.timezone {
                matches += 1;
            }
        }

        if self.language.is_some() && other.language.is_some() {
            total_checks += 1;
            if self.language == other.language {
                matches += 1;
            }
        }

        if self.platform.is_some() && other.platform.is_some() {
            total_checks += 1;
            if self.platform == other.platform {
                matches += 1;
            }
        }

        if total_checks == 0 {
            return false; // No data to compare
        }

        let match_ratio = matches as f32 / total_checks as f32;
        match_ratio >= tolerance
    }
}

/// Geographic location information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub isp: Option<String>,
}

/// Security event types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SecurityEventType {
    NewDevice,
    SuspiciousLocation,
    IpAddressChange,
    DeviceChange,
    ConcurrentSessions,
    FailedLogin,
    TokenReuse,
    UnusualActivity,
}

/// Session security event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: String,
    pub user_id: String,
    pub event_type: SecurityEventType,
    pub ip_address: IpAddr,
    pub device_fingerprint: DeviceFingerprint,
    pub geo_location: Option<GeoLocation>,
    pub timestamp: DateTime<Utc>,
    pub risk_score: u8, // 0-100
    pub details: String,
    pub resolved: bool,
}

/// Session security analysis result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityAnalysisResult {
    pub risk_score: u8, // 0-100, higher is more risky
    pub risk_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub allow_session: bool,
    pub require_additional_auth: bool,
}

/// Session security service configuration
#[derive(Clone, Debug)]
pub struct SessionSecurityConfig {
    pub max_concurrent_sessions: usize,
    pub location_change_threshold_km: f64,
    pub device_fingerprint_tolerance: f32, // 0.0-1.0
    pub high_risk_threshold: u8, // 0-100
    pub moderate_risk_threshold: u8, // 0-100
    pub enable_geo_blocking: bool,
    pub allowed_countries: Vec<String>,
    pub block_tor_vpn: bool,
}

impl Default for SessionSecurityConfig {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: 5,
            location_change_threshold_km: 100.0, // 100km
            device_fingerprint_tolerance: 0.8, // 80% match
            high_risk_threshold: 80,
            moderate_risk_threshold: 50,
            enable_geo_blocking: false,
            allowed_countries: vec![], // Empty means allow all
            block_tor_vpn: false,
        }
    }
}

/// Enhanced session security service
pub struct SessionSecurityService {
    config: SessionSecurityConfig,
    // In-memory storage for demo - in production would use database
    security_events: Arc<tokio::sync::RwLock<Vec<SecurityEvent>>>,
    user_sessions: Arc<tokio::sync::RwLock<HashMap<String, Vec<UserSessionInfo>>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSessionInfo {
    pub session_id: String,
    pub user_id: String,
    pub ip_address: IpAddr,
    pub device_fingerprint: DeviceFingerprint,
    pub geo_location: Option<GeoLocation>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
}

impl SessionSecurityService {
    pub fn new(config: SessionSecurityConfig) -> Self {
        Self {
            config,
            security_events: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            user_sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Analyze session security for a login attempt
    pub async fn analyze_session_security(
        &self,
        user_id: &str,
        ip_address: IpAddr,
        device_fingerprint: DeviceFingerprint,
        geo_location: Option<GeoLocation>,
    ) -> AppResult<SecurityAnalysisResult> {
        let mut risk_score = 0u8;
        let mut risk_factors = Vec::new();
        let mut recommended_actions = Vec::new();

        // Check for new device
        let is_new_device = self.is_new_device(user_id, &device_fingerprint).await?;
        if is_new_device {
            risk_score += 20;
            risk_factors.push("New device detected".to_string());
            recommended_actions.push("Consider requiring 2FA verification".to_string());
        }

        // Check IP address change
        let ip_change_risk = self.analyze_ip_change(user_id, ip_address).await?;
        risk_score += ip_change_risk;
        if ip_change_risk > 0 {
            risk_factors.push("IP address change detected".to_string());
        }

        // Check geographic location change
        if let Some(ref geo) = geo_location {
            let geo_risk = self.analyze_location_change(user_id, geo).await?;
            risk_score += geo_risk;
            if geo_risk > 30 {
                risk_factors.push("Suspicious location change".to_string());
                recommended_actions.push("Block login from unusual location".to_string());
            }
        }

        // Check concurrent sessions
        let concurrent_sessions = self.count_concurrent_sessions(user_id).await?;
        if concurrent_sessions >= self.config.max_concurrent_sessions {
            risk_score += 25;
            risk_factors.push("Maximum concurrent sessions reached".to_string());
            recommended_actions.push("Terminate oldest session".to_string());
        }

        // Geo-blocking check
        if self.config.enable_geo_blocking {
            if let Some(ref geo) = geo_location {
                if let Some(ref country) = geo.country {
                    if !self.config.allowed_countries.is_empty() 
                        && !self.config.allowed_countries.contains(country) {
                        risk_score = 100; // Immediately block
                        risk_factors.push(format!("Login from blocked country: {}", country));
                        recommended_actions.push("Block login entirely".to_string());
                    }
                }
            }
        }

        // Cap risk score at 100
        risk_score = risk_score.min(100);

        let allow_session = risk_score < self.config.high_risk_threshold;
        let require_additional_auth = risk_score >= self.config.moderate_risk_threshold;

        Ok(SecurityAnalysisResult {
            risk_score,
            risk_factors,
            recommended_actions,
            allow_session,
            require_additional_auth,
        })
    }

    /// Record a new session
    pub async fn record_session(
        &self,
        session_id: &str,
        user_id: &str,
        ip_address: IpAddr,
        device_fingerprint: DeviceFingerprint,
        geo_location: Option<GeoLocation>,
    ) -> AppResult<()> {
        let session_info = UserSessionInfo {
            session_id: session_id.to_string(),
            user_id: user_id.to_string(),
            ip_address,
            device_fingerprint,
            geo_location,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            is_active: true,
        };

        let mut sessions = self.user_sessions.write().await;
        sessions.entry(user_id.to_string()).or_insert_with(Vec::new).push(session_info);

        info!("Recorded new session {} for user {}", session_id, user_id);
        Ok(())
    }

    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str, user_id: &str) -> AppResult<()> {
        let mut sessions = self.user_sessions.write().await;
        if let Some(user_sessions) = sessions.get_mut(user_id) {
            if let Some(session) = user_sessions.iter_mut().find(|s| s.session_id == session_id) {
                session.last_activity = Utc::now();
            }
        }
        Ok(())
    }

    /// Terminate a session
    pub async fn terminate_session(&self, session_id: &str, user_id: &str) -> AppResult<()> {
        let mut sessions = self.user_sessions.write().await;
        if let Some(user_sessions) = sessions.get_mut(user_id) {
            if let Some(session) = user_sessions.iter_mut().find(|s| s.session_id == session_id) {
                session.is_active = false;
            }
        }

        info!("Terminated session {} for user {}", session_id, user_id);
        Ok(())
    }

    /// Record a security event
    pub async fn record_security_event(
        &self,
        user_id: &str,
        event_type: SecurityEventType,
        ip_address: IpAddr,
        device_fingerprint: DeviceFingerprint,
        geo_location: Option<GeoLocation>,
        risk_score: u8,
        details: &str,
    ) -> AppResult<String> {
        let event_id = uuid::Uuid::new_v4().to_string();
        
        let event = SecurityEvent {
            event_id: event_id.clone(),
            user_id: user_id.to_string(),
            event_type,
            ip_address,
            device_fingerprint,
            geo_location,
            timestamp: Utc::now(),
            risk_score,
            details: details.to_string(),
            resolved: false,
        };

        let mut events = self.security_events.write().await;
        events.push(event);

        info!("Recorded security event {} for user {} with risk score {}", 
              event_id, user_id, risk_score);

        Ok(event_id)
    }

    /// Get security events for a user
    pub async fn get_user_security_events(&self, user_id: &str, limit: usize) -> AppResult<Vec<SecurityEvent>> {
        let events = self.security_events.read().await;
        let user_events: Vec<SecurityEvent> = events
            .iter()
            .filter(|e| e.user_id == user_id)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev() // Most recent first
            .take(limit)
            .collect();

        Ok(user_events)
    }

    /// Clean up old events and sessions
    pub async fn cleanup_old_data(&self, older_than_days: i64) -> AppResult<usize> {
        let cutoff = Utc::now() - Duration::days(older_than_days);
        let mut total_cleaned = 0;

        // Clean old events
        let mut events = self.security_events.write().await;
        let initial_count = events.len();
        events.retain(|e| e.timestamp > cutoff);
        let events_cleaned = initial_count - events.len();
        total_cleaned += events_cleaned;

        // Clean old inactive sessions
        let mut sessions = self.user_sessions.write().await;
        for user_sessions in sessions.values_mut() {
            let initial_count = user_sessions.len();
            user_sessions.retain(|s| s.is_active || s.last_activity > cutoff);
            total_cleaned += initial_count - user_sessions.len();
        }

        info!("Cleaned {} old security records", total_cleaned);
        Ok(total_cleaned)
    }

    // Private helper methods

    async fn is_new_device(&self, user_id: &str, device_fingerprint: &DeviceFingerprint) -> AppResult<bool> {
        let sessions = self.user_sessions.read().await;
        if let Some(user_sessions) = sessions.get(user_id) {
            for session in user_sessions {
                if session.device_fingerprint.matches(device_fingerprint, self.config.device_fingerprint_tolerance) {
                    return Ok(false); // Found matching device
                }
            }
        }
        Ok(true) // No matching device found
    }

    async fn analyze_ip_change(&self, user_id: &str, new_ip: IpAddr) -> AppResult<u8> {
        let sessions = self.user_sessions.read().await;
        if let Some(user_sessions) = sessions.get(user_id) {
            if let Some(last_session) = user_sessions.last() {
                if last_session.ip_address != new_ip {
                    // Simple IP change detection - could be enhanced with IP geolocation
                    return Ok(15); // Moderate risk for IP change
                }
            }
        }
        Ok(0)
    }

    async fn analyze_location_change(&self, user_id: &str, new_location: &GeoLocation) -> AppResult<u8> {
        let sessions = self.user_sessions.read().await;
        if let Some(user_sessions) = sessions.get(user_id) {
            for session in user_sessions.iter().rev().take(3) { // Check last 3 sessions
                if let Some(ref old_location) = session.geo_location {
                    // Simple location change detection
                    if let (Some(old_country), Some(new_country)) = (&old_location.country, &new_location.country) {
                        if old_country != new_country {
                            // Different country is higher risk
                            return Ok(40);
                        }
                    }
                    if let (Some(old_region), Some(new_region)) = (&old_location.region, &new_location.region) {
                        if old_region != new_region {
                            // Different region/state is moderate risk
                            return Ok(20);
                        }
                    }
                }
            }
        }
        Ok(0)
    }

    async fn count_concurrent_sessions(&self, user_id: &str) -> AppResult<usize> {
        let sessions = self.user_sessions.read().await;
        if let Some(user_sessions) = sessions.get(user_id) {
            return Ok(user_sessions.iter().filter(|s| s.is_active).count());
        }
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_device_fingerprint_matching() {
        let mut fp1 = DeviceFingerprint {
            user_agent: Some("Mozilla/5.0 Chrome".to_string()),
            screen_resolution: Some("1920x1080".to_string()),
            timezone: Some("UTC".to_string()),
            language: Some("en-US".to_string()),
            platform: Some("Win32".to_string()),
            browser: None,
            os: None,
            device_type: None,
            fingerprint_hash: None,
            canvas_fingerprint: None,
            webgl_fingerprint: None,
        };

        let mut fp2 = fp1.clone();
        fp1.generate_hash();
        fp2.generate_hash();

        assert!(fp1.matches(&fp2, 0.8));
    }

    #[tokio::test]
    async fn test_security_service_creation() {
        let config = SessionSecurityConfig::default();
        let service = SessionSecurityService::new(config);
        
        // Test service creation
        assert_eq!(service.config.max_concurrent_sessions, 5);
    }

    #[tokio::test]
    async fn test_session_analysis() {
        let config = SessionSecurityConfig::default();
        let service = SessionSecurityService::new(config);

        let ip = IpAddr::from_str("192.168.1.1").unwrap();
        let device_fp = DeviceFingerprint {
            user_agent: Some("Test".to_string()),
            screen_resolution: None,
            timezone: None,
            language: None,
            platform: None,
            browser: None,
            os: None,
            device_type: None,
            fingerprint_hash: None,
            canvas_fingerprint: None,
            webgl_fingerprint: None,
        };

        let result = service.analyze_session_security("test_user", ip, device_fp, None).await.unwrap();
        
        // New device should have some risk
        assert!(result.risk_score > 0);
        assert!(result.risk_factors.contains(&"New device detected".to_string()));
    }
}