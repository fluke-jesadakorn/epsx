// Tracks security-related information and threat detection

use crate::domain::authentication::ClientInformation;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Security context for authentication sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    // IP and location tracking
    ip_addresses: Vec<IpAddress>,
    geographic_locations: Vec<GeographicLocation>,
    
    // Device and client tracking  
    user_agents: Vec<UserAgent>,
    device_fingerprints: Vec<DeviceFingerprint>,
    
    // Security metrics
    failed_attempts: u32,
    suspicious_activity_score: f64,
    last_security_check: DateTime<Utc>,
    
    // Risk assessment
    risk_level: RiskLevel,
    security_flags: Vec<SecurityFlag>,
}

impl SecurityContext {
    /// Create new security context
    pub fn new() -> Self {
        Self {
            ip_addresses: Vec::new(),
            geographic_locations: Vec::new(),
            user_agents: Vec::new(),
            device_fingerprints: Vec::new(),
            failed_attempts: 0,
            suspicious_activity_score: 0.0,
            last_security_check: Utc::now(),
            risk_level: RiskLevel::Low,
            security_flags: Vec::new(),
        }
    }
    
    /// Detect if new client information represents anomalous activity
    pub fn detect_anomaly(&mut self, _client_info: &ClientInformation) -> bool {
        let mut anomaly_detected = false;
        let mut score_increase = 0.0;
        
        // Check for IP address changes
        if let Some(_current_ip) = self.ip_addresses.last() {
            // In real implementation, extract IP from client_info
            // For now, simulate IP comparison
            if self.is_suspicious_ip_change() {
                score_increase += 25.0;
                anomaly_detected = true;
                self.security_flags.push(SecurityFlag::SuspiciousIpChange);
            }
        }
        
        // Check for geographic location anomalies
        if self.is_suspicious_location_change() {
            score_increase += 30.0;
            anomaly_detected = true;
            self.security_flags.push(SecurityFlag::ImpossibleTravel);
        }
        
        // Check for device fingerprint changes
        if self.is_suspicious_device_change() {
            score_increase += 20.0;
            anomaly_detected = true;
            self.security_flags.push(SecurityFlag::DeviceChange);
        }
        
        // Update suspicious activity score
        self.suspicious_activity_score += score_increase;
        self.suspicious_activity_score = self.suspicious_activity_score.min(100.0);
        
        // Update risk level based on score
        self.update_risk_level();
        
        self.last_security_check = Utc::now();
        anomaly_detected
    }
    
    /// Check if current context represents high risk
    pub fn is_high_risk(&self) -> bool {
        matches!(self.risk_level, RiskLevel::High | RiskLevel::Critical)
    }
    
    /// Record failed authentication attempt
    pub fn record_failed_attempt(&mut self) {
        self.failed_attempts += 1;
        self.suspicious_activity_score += 5.0;
        
        if self.failed_attempts >= 5 {
            self.security_flags.push(SecurityFlag::MultipleFailures);
        }
        
        self.update_risk_level();
    }
    
    /// Reset failed attempts (on successful authentication)
    pub fn reset_failed_attempts(&mut self) {
        self.failed_attempts = 0;
        // Reduce suspicion score slightly on successful auth
        self.suspicious_activity_score = (self.suspicious_activity_score - 10.0).max(0.0);
        self.update_risk_level();
    }
    
    /// Add security flag
    pub fn add_security_flag(&mut self, flag: SecurityFlag) {
        if !self.security_flags.contains(&flag) {
            self.security_flags.push(flag);
        }
    }
    
    /// Clear all security flags
    pub fn clear_security_flags(&mut self) {
        self.security_flags.clear();
        self.suspicious_activity_score = 0.0;
        self.update_risk_level();
    }
    
    /// Get security summary for logging/monitoring
    pub fn security_summary(&self) -> SecuritySummary {
        SecuritySummary {
            risk_level: self.risk_level.clone(),
            suspicious_score: self.suspicious_activity_score,
            failed_attempts: self.failed_attempts,
            active_flags: self.security_flags.clone(),
            last_check: self.last_security_check,
            unique_ips: self.ip_addresses.len(),
            unique_locations: self.geographic_locations.len(),
        }
    }
    
    // Getters
    pub fn risk_level(&self) -> &RiskLevel { &self.risk_level }
    pub fn suspicious_score(&self) -> f64 { self.suspicious_activity_score }
    pub fn failed_attempts(&self) -> u32 { self.failed_attempts }
    pub fn security_flags(&self) -> &[SecurityFlag] { &self.security_flags }
    
    // Private helpers
    fn is_suspicious_ip_change(&self) -> bool {
        // Simulate IP change detection logic
        // In real implementation, would compare with previous IPs
        self.ip_addresses.len() > 3
    }
    
    fn is_suspicious_location_change(&self) -> bool {
        // Simulate impossible travel detection
        // In real implementation, would calculate geographic distance and time
        self.geographic_locations.len() > 2
    }
    
    fn is_suspicious_device_change(&self) -> bool {
        // Simulate device fingerprint analysis
        // In real implementation, would analyze browser/device characteristics
        self.device_fingerprints.len() > 1
    }
    
    fn update_risk_level(&mut self) {
        self.risk_level = match self.suspicious_activity_score {
            score if score >= 80.0 => RiskLevel::Critical,
            score if score >= 60.0 => RiskLevel::High,
            score if score >= 30.0 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        };
        
        // High-priority flags automatically elevate risk
        for flag in &self.security_flags {
            match flag {
                SecurityFlag::ImpossibleTravel | SecurityFlag::KnownThreat => {
                    self.risk_level = RiskLevel::Critical;
                    break;
                },
                SecurityFlag::MultipleFailures | SecurityFlag::SuspiciousIpChange => {
                    if matches!(self.risk_level, RiskLevel::Low) {
                        self.risk_level = RiskLevel::Medium;
                    }
                },
                _ => {}
            }
        }
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Risk assessment levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Security flags indicating potential threats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityFlag {
    SuspiciousIpChange,
    ImpossibleTravel,
    DeviceChange,
    MultipleFailures,
    BruteForceAttempt,
    KnownThreat,
    RateLimitExceeded,
    InvalidUserAgent,
}

/// IP address tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpAddress {
    address: String,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    is_trusted: bool,
}

impl IpAddress {
    pub fn new(address: String) -> Self {
        let now = Utc::now();
        Self {
            address,
            first_seen: now,
            last_seen: now,
            is_trusted: false,
        }
    }
    
    pub fn address(&self) -> &str { &self.address }
    pub fn is_trusted(&self) -> bool { self.is_trusted }
}

/// Geographic location tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeographicLocation {
    country_code: String,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
}

impl GeographicLocation {
    pub fn new(country_code: String, city: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            country_code,
            city,
            latitude: None,
            longitude: None,
            first_seen: now,
            last_seen: now,
        }
    }
}

/// User agent tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAgent {
    agent_string: String,
    parsed_info: UserAgentInfo,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAgentInfo {
    browser: Option<String>,
    browser_version: Option<String>,
    os: Option<String>,
    os_version: Option<String>,
    device_type: Option<String>,
}

/// Device fingerprint for tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    fingerprint_hash: String,
    characteristics: HashMap<String, String>,
    first_seen: DateTime<Utc>,
    last_seen: DateTime<Utc>,
    confidence_score: f64,
}

/// Security summary for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub risk_level: RiskLevel,
    pub suspicious_score: f64,
    pub failed_attempts: u32,
    pub active_flags: Vec<SecurityFlag>,
    pub last_check: DateTime<Utc>,
    pub unique_ips: usize,
    pub unique_locations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn new_security_context_is_low_risk() {
        let context = SecurityContext::new();
        assert_eq!(context.risk_level(), &RiskLevel::Low);
        assert_eq!(context.suspicious_score(), 0.0);
        assert_eq!(context.failed_attempts(), 0);
    }
    
    #[test]
    fn failed_attempts_increase_risk() {
        let context = SecurityContext::new();
        
        // Record multiple failures
        for _ in 0..6 {
            context.record_failed_attempt();
        }
        
        assert!(context.failed_attempts() >= 5);
        assert!(context.suspicious_score() > 0.0);
        assert!(context.security_flags().contains(&SecurityFlag::MultipleFailures));
    }
    
    #[test]
    fn successful_auth_reduces_risk() {
        let context = SecurityContext::new();
        context.record_failed_attempt();
        context.record_failed_attempt();
        
        let score_before = context.suspicious_score();
        context.reset_failed_attempts();
        
        assert_eq!(context.failed_attempts(), 0);
        assert!(context.suspicious_score() < score_before);
    }
    
    #[test]
    fn security_summary_contains_key_metrics() {
        let context = SecurityContext::new();
        context.add_security_flag(SecurityFlag::SuspiciousIpChange);
        
        let summary = context.security_summary();
        assert_eq!(summary.risk_level, RiskLevel::Medium);
        assert!(summary.active_flags.contains(&SecurityFlag::SuspiciousIpChange));
    }
}