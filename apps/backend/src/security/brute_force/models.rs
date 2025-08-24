// Data models for brute force detection system

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::net::IpAddr;

/// Login attempt record for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttempt {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub device_fingerprint: Option<String>,
    pub session_id: Option<String>,
    pub request_path: Option<String>,
    pub http_method: Option<String>,
    pub request_size: Option<u32>,
    pub response_time_ms: Option<u32>,
}

/// Attack attempt tracking in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackAttempt {
    pub id: Uuid,
    pub ip_address: String,
    pub attack_type: String,
    pub severity_level: String,
    pub attempt_count: i32,
    pub first_attempt: DateTime<Utc>,
    pub last_attempt: DateTime<Utc>,
    pub blocked: bool,
    pub blocked_until: Option<DateTime<Utc>>,
    pub user_agents: Option<serde_json::Value>,
    pub target_usernames: Option<serde_json::Value>,
    pub geographic_data: Option<serde_json::Value>,
    pub pattern_signatures: Option<serde_json::Value>,
    pub ml_confidence_score: Option<f64>,
    pub response_actions: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// IP reputation and blocking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpReputation {
    pub id: Uuid,
    pub ip_address: String,
    pub reputation_score: f64,
    pub is_malicious: bool,
    pub is_vpn: bool,
    pub is_proxy: bool,
    pub is_tor: bool,
    pub country_code: Option<String>,
    pub asn: Option<i32>,
    pub organization: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub attack_history_count: i32,
    pub successful_attacks: i32,
    pub blocked_count: i32,
    pub threat_categories: Option<serde_json::Value>,
    pub confidence_level: f64,
    pub data_sources: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pattern analysis results for machine learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPattern {
    pub id: Uuid,
    pub pattern_type: String,
    pub signature_hash: String,
    pub description: String,
    pub detection_count: i32,
    pub success_rate: f64,
    pub avg_requests_per_minute: f64,
    pub common_user_agents: Option<serde_json::Value>,
    pub target_endpoints: Option<serde_json::Value>,
    pub timing_characteristics: Option<serde_json::Value>,
    pub geographic_distribution: Option<serde_json::Value>,
    pub severity_score: f64,
    pub false_positive_rate: f64,
    pub first_detected: DateTime<Utc>,
    pub last_detected: DateTime<Utc>,
    pub status: String, // active, deprecated, false_positive
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response action logging (database record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseActionRecord {
    pub id: Uuid,
    pub attack_attempt_id: Uuid,
    pub action_type: String,
    pub action_details: serde_json::Value,
    pub executed_at: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
    pub duration_minutes: Option<i32>,
    pub automated: bool,
    pub operator_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Machine learning training data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlTrainingData {
    pub id: Uuid,
    pub feature_vector: serde_json::Value,
    pub label: String, // attack, legitimate, suspicious
    pub confidence: f64,
    pub ip_address: String,
    pub timestamp: DateTime<Utc>,
    pub attack_type: Option<String>,
    pub verified: bool,
    pub feedback_score: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Real-time attack analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackAnalysis {
    pub attack_detected: bool,
    pub threat_level: ThreatLevel,
    pub attack_types: Vec<AttackType>,
    pub confidence_score: f64,
    pub algorithms_used: Vec<String>,
    pub risk_factors: Vec<RiskFactor>,
    pub temporal_analysis: TemporalAnalysis,
    pub geographic_analysis: Option<GeographicAnalysis>,
    pub behavioral_analysis: BehavioralAnalysis,
}

/// Pattern analysis with ML insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysisResult {
    pub ml_confidence: f64,
    pub signatures: Vec<String>,
    pub anomaly_score: f64,
    pub pattern_matches: Vec<PatternMatch>,
    pub feature_importance: HashMap<String, f64>,
    pub prediction_explanation: Vec<String>,
}

/// Response execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseExecutionResult {
    pub actions: Vec<ResponseActionResult>,
    pub blocked_until: Option<DateTime<Utc>>,
    pub recommendations: Vec<String>,
    pub strategy: ResponseStrategy,
    pub escalation_level: u32,
}

/// Response actions for threat mitigation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResponseAction {
    None,
    Monitor,
    RateLimit,
    RequireCaptcha,
    RequireMfa,
    Block,
    Alert,
    LockAccount,
}

/// Pattern types for detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternType {
    TraditionalBrute,
    DistributedBrute,
    PasswordSpray,
    CredentialStuffing,
    SlowBrute,
    TimeBasedAnomaly,
    GeographicAnomaly,
    DeviceAnomaly,
    BehavioralAnomaly,
}

/// Detection thresholds for engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionThresholds {
    pub max_failed_attempts: i64,
    pub time_window_minutes: i64,
    pub risk_score_block_threshold: f64,
    pub risk_score_captcha_threshold: f64,
    pub risk_score_mfa_threshold: f64,
    pub confidence_threshold: f64,
}

impl Default for DetectionThresholds {
    fn default() -> Self {
        Self {
            max_failed_attempts: 10,
            time_window_minutes: 15,
            risk_score_block_threshold: 8.0,
            risk_score_captcha_threshold: 5.0,
            risk_score_mfa_threshold: 6.5,
            confidence_threshold: 0.7,
        }
    }
}

/// Threat intelligence data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelligence {
    pub source: String,
    pub confidence: f64,
    pub threat_types: Vec<String>,
    pub indicators: HashMap<String, serde_json::Value>,
    pub last_updated: DateTime<Utc>,
    pub ttl_seconds: Option<i64>,
}

/// Analysis context for threat detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    pub ip_address: IpAddr,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub user_agent: Option<String>,
    pub endpoint: String,
    pub method: String,
    pub geolocation: Option<GeolocationData>,
    pub device_fingerprint: Option<String>,
    pub request_headers: HashMap<String, String>,
    pub request_body_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub response_code: Option<u16>,
    pub response_time_ms: Option<u64>,
    pub correlation_id: Option<String>,
}

/// Geolocation data for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeolocationData {
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub is_vpn: bool,
    pub is_proxy: bool,
    pub is_tor: bool,
}

/// Detection result from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub attack_detected: bool,
    pub attack_type: Option<AttackType>,
    pub confidence_score: f64,
    pub risk_score: f64,
    pub threat_level: ThreatLevel,
    pub recommended_actions: Vec<ResponseAction>,
    pub detected_patterns: Vec<DetectedPattern>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Detected pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_type: PatternType,
    pub pattern_id: String,
    pub confidence: f64,
    pub severity: ThreatLevel,
    pub indicators: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Security summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub time_window: TimeWindow,
    pub total_attacks_blocked: i64,
    pub total_ips_blocked: i64,
    pub total_patterns_detected: i64,
    pub threat_level_distribution: HashMap<String, i64>,
    pub attack_type_distribution: HashMap<String, i64>,
    pub geographic_distribution: HashMap<String, i64>,
    pub top_attacking_ips: Vec<String>,
    pub top_targeted_endpoints: Vec<String>,
    pub average_risk_score: f64,
    pub generated_at: DateTime<Utc>,
}

/// Time window for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_hours: i32,
}

/// Detection error types
#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Brute force protection error types
#[derive(Debug, thiserror::Error)]
pub enum BruteForceError {
    #[error("Cache error: {0}")]
    CacheError(#[from] crate::infra::cache::CacheError),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Detection error: {0}")]
    DetectionError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Analysis error: {0}")]
    AnalysisError(String),
}

/// Individual response action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseActionResult {
    pub action_type: ResponseActionType,
    pub success: bool,
    pub message: String,
    pub duration: Option<chrono::Duration>,
}

/// Enums for type safety and categorization

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AttackType {
    BruteForce,
    TraditionalBruteForce,
    SlowBruteForce,
    CredentialStuffing,
    PasswordSpraying,
    DistributedAttack,
    BotnetAttack,
    HybridAttack,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseStrategy {
    Progressive,
    Immediate,
    Adaptive,
    ManualReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseActionType {
    IpBlock,
    RateLimit,
    CaptchaChallenge,
    AccountLock,
    GeofenceBlock,
    UserAgentBlock,
    HoneypotRedirect,
    TarPit,
}

/// Supporting analysis structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: String,
    pub weight: f64,
    pub value: f64,
    pub description: String,
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysis {
    pub requests_per_minute: f64,
    pub requests_per_hour: f64,
    pub time_between_attempts_ms: Vec<u64>,
    pub peak_activity_hours: Vec<u8>,
    pub consistency_score: f64,
    pub burst_detection: BurstPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstPattern {
    pub detected: bool,
    pub intensity: f64,
    pub duration_seconds: u32,
    pub request_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicAnalysis {
    pub country_code: String,
    pub city: Option<String>,
    pub is_suspicious_location: bool,
    pub distance_from_previous_km: Option<f64>,
    pub impossible_travel: bool,
    pub vpn_probability: f64,
    pub proxy_probability: f64,
    pub tor_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralAnalysis {
    pub user_agent_consistency: f64,
    pub session_behavior_score: f64,
    pub request_pattern_anomaly: f64,
    pub timing_fingerprint: TimingFingerprint,
    pub automation_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingFingerprint {
    pub human_like_variance: f64,
    pub mechanical_precision: f64,
    pub typing_speed_estimate: Option<f64>,
    pub think_time_patterns: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub pattern_type: String,
    pub confidence: f64,
    pub similarity_score: f64,
    pub matched_features: Vec<String>,
}

/// Brute force data for compatibility (stub for Diesel migration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BruteForceData {
    pub ip_address: String,
    pub attempt_count: i32,
    pub first_attempt: DateTime<Utc>,
    pub last_attempt: DateTime<Utc>,
    pub blocked: bool,
}

/// Database query structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub attack_type: Option<AttackType>,
    pub threat_level: Option<ThreatLevel>,
    pub blocked_only: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpReputationQuery {
    pub ip_address: Option<String>,
    pub reputation_threshold: Option<f64>,
    pub is_malicious: Option<bool>,
    pub country_codes: Option<Vec<String>>,
    pub min_attack_count: Option<i32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Statistics and reporting structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackStatistics {
    pub total_attacks: i64,
    pub attacks_by_type: HashMap<AttackType, i64>,
    pub attacks_by_hour: HashMap<u8, i64>,
    pub top_attack_ips: Vec<IpAttackSummary>,
    pub geographic_distribution: HashMap<String, i64>,
    pub success_rate: f64,
    pub average_duration_minutes: f64,
    pub blocked_attempts: i64,
    pub false_positives: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAttackSummary {
    pub ip_address: String,
    pub attack_count: i64,
    pub success_count: i64,
    pub last_attack: DateTime<Utc>,
    pub threat_level: ThreatLevel,
    pub blocked: bool,
}

impl LoginAttempt {
    pub fn new(
        ip_address: String,
        success: bool,
        username: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            ip_address,
            user_agent,
            username,
            email: None,
            success,
            failure_reason: if success { None } else { Some("Authentication failed".to_string()) },
            country_code: None,
            city: None,
            device_fingerprint: None,
            session_id: None,
            request_path: None,
            http_method: None,
            request_size: None,
            response_time_ms: None,
        }
    }

    pub fn with_geographic_data(mut self, country_code: String, city: Option<String>) -> Self {
        self.country_code = Some(country_code);
        self.city = city;
        self
    }

    pub fn with_request_details(
        mut self,
        path: String,
        method: String,
        size: u32,
        response_time: u32,
    ) -> Self {
        self.request_path = Some(path);
        self.http_method = Some(method);
        self.request_size = Some(size);
        self.response_time_ms = Some(response_time);
        self
    }
}

impl ThreatLevel {
    pub fn to_score(&self) -> u8 {
        match self {
            ThreatLevel::Low => 1,
            ThreatLevel::Medium => 2,
            ThreatLevel::High => 3,
            ThreatLevel::Critical => 4,
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score >= 8.0 {
            ThreatLevel::Critical
        } else if score >= 6.0 {
            ThreatLevel::High
        } else if score >= 3.0 {
            ThreatLevel::Medium
        } else {
            ThreatLevel::Low
        }
    }
}

/// Request pattern for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPattern {
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub path: String,
    pub method: String,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
}

impl AttackType {
    pub fn description(&self) -> &'static str {
        match self {
            AttackType::BruteForce => "Basic brute force attack",
            AttackType::TraditionalBruteForce => "High-frequency automated password guessing",
            AttackType::SlowBruteForce => "Low-frequency stealth password attacks",
            AttackType::CredentialStuffing => "Using leaked credentials across multiple services",
            AttackType::PasswordSpraying => "Common passwords against multiple accounts",
            AttackType::DistributedAttack => "Coordinated attack from multiple IP addresses",
            AttackType::BotnetAttack => "Large-scale automated attack network",
            AttackType::HybridAttack => "Multiple attack techniques combined",
        }
    }

    pub fn severity_multiplier(&self) -> f64 {
        match self {
            AttackType::BruteForce => 1.0,
            AttackType::TraditionalBruteForce => 1.0,
            AttackType::SlowBruteForce => 1.2,
            AttackType::CredentialStuffing => 1.5,
            AttackType::PasswordSpraying => 1.3,
            AttackType::DistributedAttack => 1.8,
            AttackType::BotnetAttack => 2.0,
            AttackType::HybridAttack => 1.9,
        }
    }
}