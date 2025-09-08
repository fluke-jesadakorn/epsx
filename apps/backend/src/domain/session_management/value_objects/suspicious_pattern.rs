use chrono::{DateTime, Utc};// Suspicious Pattern Value Object
// Represents patterns that indicate potentially suspicious session activity

use serde::{Serialize, Deserialize};

use crate::domain::shared_kernel::{ValueObject, value_object::ValueObjectError};

/// Suspicious activity pattern detected in user sessions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuspiciousPattern {
    pattern_type: PatternType,
    severity_level: SeverityLevel,
    description: String,
    detected_at: DateTime<Utc>,
    confidence_score: u8, // 0-100
    evidence: Vec<Evidence>,
}

/// Types of suspicious patterns that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Multiple concurrent sessions from different locations
    ConcurrentGeolocations,
    /// Rapid succession of login attempts
    RapidLoginAttempts,
    /// Unusual time of day access
    UnusualTimeAccess,
    /// Access from known malicious IP
    MaliciousIPAccess,
    /// Device fingerprint mismatch
    DeviceFingerprintMismatch,
    /// Suspicious user agent
    SuspiciousUserAgent,
    /// Velocity-based anomaly (too many actions too quickly)
    VelocityAnomaly,
    /// Session hijacking indicators
    SessionHijackingIndicators,
    /// Brute force attack pattern
    BruteForcePattern,
    /// Custom pattern defined by security rules
    CustomPattern(String),
}

/// Severity levels for suspicious patterns
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SeverityLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Evidence supporting the suspicious pattern detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    evidence_type: EvidenceType,
    value: String,
    timestamp: DateTime<Utc>,
}

/// Types of evidence that can support pattern detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceType {
    IPAddress,
    UserAgent,
    Geolocation,
    Timestamp,
    SessionId,
    DeviceFingerprint,
    ActionCount,
    Custom(String),
}

impl SuspiciousPattern {
    /// Create a new suspicious pattern
    pub fn new(
        pattern_type: PatternType,
        severity_level: SeverityLevel,
        description: String,
        confidence_score: u8,
    ) -> Result<Self, SuspiciousPatternError> {
        if confidence_score > 100 {
            return Err(SuspiciousPatternError::InvalidConfidenceScore);
        }
        
        if description.trim().is_empty() {
            return Err(SuspiciousPatternError::EmptyDescription);
        }
        
        let pattern = Self {
            pattern_type,
            severity_level,
            description,
            detected_at: Utc::now(),
            confidence_score,
            evidence: Vec::new(),
        };
        
        pattern.validate()?;
        Ok(pattern)
    }
    
    /// Add evidence to support this pattern
    pub fn add_evidence(&mut self, evidence_type: EvidenceType, value: String) -> Result<(), SuspiciousPatternError> {
        if value.trim().is_empty() {
            return Err(SuspiciousPatternError::EmptyEvidenceValue);
        }
        
        let evidence = Evidence {
            evidence_type,
            value,
            timestamp: Utc::now(),
        };
        
        self.evidence.push(evidence);
        Ok(())
    }
    
    /// Check if this pattern should trigger an immediate security response
    pub fn requires_immediate_action(&self) -> bool {
        match self.severity_level {
            SeverityLevel::Critical => true,
            SeverityLevel::High => self.confidence_score >= 80,
            _ => false,
        }
    }
    
    /// Get risk score (combines severity and confidence)
    pub fn risk_score(&self) -> u16 {
        let severity_multiplier = match self.severity_level {
            SeverityLevel::Low => 1,
            SeverityLevel::Medium => 2,
            SeverityLevel::High => 3,
            SeverityLevel::Critical => 4,
        };
        
        (self.confidence_score as u16) * severity_multiplier
    }
    
    // Getters
    pub fn pattern_type(&self) -> &PatternType {
        &self.pattern_type
    }
    
    pub fn severity_level(&self) -> &SeverityLevel {
        &self.severity_level
    }
    
    pub fn description(&self) -> &str {
        &self.description
    }
    
    pub fn detected_at(&self) -> DateTime<Utc> {
        self.detected_at
    }
    
    pub fn confidence_score(&self) -> u8 {
        self.confidence_score
    }
    
    pub fn evidence(&self) -> &[Evidence] {
        &self.evidence
    }
    
    pub fn evidence_count(&self) -> usize {
        self.evidence.len()
    }
}

impl ValueObject for SuspiciousPattern {
    type Error = SuspiciousPatternError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.description.trim().is_empty() {
            return Err(SuspiciousPatternError::EmptyDescription);
        }
        
        if self.confidence_score > 100 {
            return Err(SuspiciousPatternError::InvalidConfidenceScore);
        }
        
        // Validate evidence
        for evidence in &self.evidence {
            if evidence.value.trim().is_empty() {
                return Err(SuspiciousPatternError::EmptyEvidenceValue);
            }
        }
        
        Ok(())
    }
}

/// Errors that can occur when working with suspicious patterns
#[derive(Debug, thiserror::Error)]
pub enum SuspiciousPatternError {
    #[error("Confidence score must be between 0 and 100")]
    InvalidConfidenceScore,
    
    #[error("Pattern description cannot be empty")]
    EmptyDescription,
    
    #[error("Evidence value cannot be empty")]
    EmptyEvidenceValue,
    
    #[error("Pattern validation failed: {0}")]
    ValidationFailed(String),
}

impl From<SuspiciousPatternError> for ValueObjectError {
    fn from(error: SuspiciousPatternError) -> Self {
        ValueObjectError::ValidationFailed(error.to_string())
    }
}

// Common pattern constructors for convenience
impl SuspiciousPattern {
    /// Create a concurrent geolocation pattern
    pub fn concurrent_geolocations(
        locations: Vec<String>,
        confidence: u8,
    ) -> Result<Self, SuspiciousPatternError> {
        let mut pattern = Self::new(
            PatternType::ConcurrentGeolocations,
            SeverityLevel::High,
            format!("Concurrent sessions from {} different locations", locations.len()),
            confidence,
        )?;
        
        for location in locations {
            pattern.add_evidence(EvidenceType::Geolocation, location)?;
        }
        
        Ok(pattern)
    }
    
    /// Create a rapid login attempts pattern
    pub fn rapid_login_attempts(
        attempt_count: u32,
        time_window_minutes: u32,
        confidence: u8,
    ) -> Result<Self, SuspiciousPatternError> {
        Self::new(
            PatternType::RapidLoginAttempts,
            SeverityLevel::Medium,
            format!("{} login attempts in {} minutes", attempt_count, time_window_minutes),
            confidence,
        )
    }
    
    /// Create a malicious IP pattern
    pub fn malicious_ip_access(
        ip_address: String,
        threat_level: &str,
        confidence: u8,
    ) -> Result<Self, SuspiciousPatternError> {
        let mut pattern = Self::new(
            PatternType::MaliciousIPAccess,
            SeverityLevel::Critical,
            format!("Access from malicious IP ({} threat level)", threat_level),
            confidence,
        )?;
        
        pattern.add_evidence(EvidenceType::IPAddress, ip_address)?;
        pattern.add_evidence(EvidenceType::Custom("threat_level".to_string()), threat_level.to_string())?;
        
        Ok(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_suspicious_pattern() {
        let pattern = SuspiciousPattern::new(
            PatternType::ConcurrentGeolocations,
            SeverityLevel::High,
            "Multiple concurrent sessions".to_string(),
            85,
        ).unwrap();
        
        assert_eq!(pattern.confidence_score(), 85);
        assert_eq!(pattern.severity_level(), &SeverityLevel::High);
        assert!(pattern.requires_immediate_action());
    }
    
    #[test]
    fn test_risk_score_calculation() {
        let pattern = SuspiciousPattern::new(
            PatternType::ConcurrentGeolocations,
            SeverityLevel::High,
            "Test pattern".to_string(),
            80,
        ).unwrap();
        
        // High severity (3) * 80 confidence = 240
        assert_eq!(pattern.risk_score(), 240);
    }
    
    #[test]
    fn test_add_evidence() {
        let mut pattern = SuspiciousPattern::new(
            PatternType::MaliciousIPAccess,
            SeverityLevel::Critical,
            "Malicious IP detected".to_string(),
            95,
        ).unwrap();
        
        pattern.add_evidence(EvidenceType::IPAddress, "192.168.1.100".to_string()).unwrap();
        pattern.add_evidence(EvidenceType::Geolocation, "Unknown".to_string()).unwrap();
        
        assert_eq!(pattern.evidence_count(), 2);
    }
}