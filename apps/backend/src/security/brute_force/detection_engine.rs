// Advanced brute force detection engine with machine learning capabilities
// Provides real-time threat detection and adaptive response mechanisms

use crate::security::brute_force::models::*;
use crate::infra::cache::security_cache::SecurityCache;
use chrono::{Utc, Duration, Timelike};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, error};

/// Core brute force detection engine with ML-based threat analysis
pub struct BruteForceDetectionEngine {
    security_cache: Arc<SecurityCache>,
    thresholds: Arc<RwLock<DetectionThresholds>>,
    pattern_analyzers: Vec<Box<dyn PatternAnalyzer + Send + Sync>>,
    threat_intel: Arc<RwLock<HashMap<String, ThreatIntelligence>>>,
    ml_model: Option<Arc<ThreatScoringModel>>,
}

impl BruteForceDetectionEngine {
    pub fn new(security_cache: Arc<SecurityCache>) -> Self {
        let mut engine = Self {
            security_cache,
            thresholds: Arc::new(RwLock::new(DetectionThresholds::default())),
            pattern_analyzers: Vec::new(),
            threat_intel: Arc::new(RwLock::new(HashMap::new())),
            ml_model: None,
        };

        // Initialize built-in pattern analyzers
        engine.register_built_in_analyzers();
        
        engine
    }

    pub fn with_thresholds(mut self, thresholds: DetectionThresholds) -> Self {
        self.thresholds = Arc::new(RwLock::new(thresholds));
        self
    }

    pub fn with_ml_model(mut self, model: Arc<ThreatScoringModel>) -> Self {
        self.ml_model = Some(model);
        self
    }

    /// Register built-in pattern analyzers
    fn register_built_in_analyzers(&mut self) {
        self.pattern_analyzers.push(Box::new(BruteForceAnalyzer::new()));
        self.pattern_analyzers.push(Box::new(PasswordSprayAnalyzer::new()));
        self.pattern_analyzers.push(Box::new(CredentialStuffingAnalyzer::new()));
        self.pattern_analyzers.push(Box::new(SlowBruteAnalyzer::new()));
        self.pattern_analyzers.push(Box::new(DistributedAttackAnalyzer::new()));
        self.pattern_analyzers.push(Box::new(GeographicAnomalyAnalyzer::new()));
        self.pattern_analyzers.push(Box::new(DeviceAnomalyAnalyzer::new()));
        self.pattern_analyzers.push(Box::new(BehavioralAnomalyAnalyzer::new()));
    }

    /// Main detection entry point - analyze incoming request for threats
    pub async fn analyze_request(&self, context: &AnalysisContext) -> Result<DetectionResult, DetectionError> {
        debug!("Analyzing request from IP: {} to endpoint: {}", context.ip_address, context.endpoint);

        // Check if IP is already blocked
        if self.security_cache.is_ip_blocked(&context.ip_address.to_string()).await
            .map_err(|e| DetectionError::CacheError(e.to_string()))? {
            return Ok(DetectionResult {
                attack_detected: true,
                attack_type: Some(AttackType::BruteForce),
                confidence_score: 10.0,
                risk_score: 10.0,
                threat_level: ThreatLevel::Critical,
                recommended_actions: vec![ResponseAction::Block],
                detected_patterns: vec![],
                metadata: HashMap::new(),
            });
        }

        // Gather threat intelligence and historical data
        let ip_reputation = self.security_cache.analyze_ip_reputation(&context.ip_address.to_string()).await
            .map_err(|e| DetectionError::CacheError(e.to_string()))?;

        let failed_attempts = self.security_cache.get_brute_force_attempts(&context.ip_address.to_string()).await
            .map_err(|e| DetectionError::CacheError(e.to_string()))?;

        // Run pattern analysis
        let mut detected_patterns = Vec::new();
        let mut max_confidence = 0.0;
        let mut primary_attack_type = None;

        for analyzer in &self.pattern_analyzers {
            match analyzer.analyze(context, &ip_reputation, failed_attempts).await {
                Ok(Some(pattern)) => {
                    if pattern.confidence > max_confidence {
                        max_confidence = pattern.confidence;
                        primary_attack_type = Some(pattern.attack_type.clone());
                    }
                    detected_patterns.push(pattern);
                }
                Ok(None) => {},
                Err(e) => {
                    error!("Pattern analysis failed: {}", e);
                }
            }
        }

        // Calculate risk score using ML model if available
        let risk_score = if let Some(ml_model) = &self.ml_model {
            match ml_model.calculate_risk_score(context, &ip_reputation, failed_attempts).await {
                Ok(score) => score,
                Err(_) => self.calculate_baseline_risk_score(context, &ip_reputation, failed_attempts).await
            }
        } else {
            self.calculate_baseline_risk_score(context, &ip_reputation, failed_attempts).await
        };

        // Determine threat level and recommended actions
        let threat_level = self.calculate_threat_level(risk_score, max_confidence).await;
        let recommended_actions = self.determine_response_actions(&threat_level, risk_score, &detected_patterns).await?;

        // Update threat intelligence with new data
        self.update_threat_intelligence(context, &ip_reputation, risk_score).await?;

        let result = DetectionResult {
            attack_detected: max_confidence > 0.5 || risk_score > 5.0,
            attack_type: primary_attack_type,
            confidence_score: max_confidence,
            risk_score,
            threat_level,
            recommended_actions,
            detected_patterns: detected_patterns.into_iter().map(|p| DetectedPattern {
                pattern_type: p.pattern_type,
                pattern_id: p.pattern_id,
                confidence: p.confidence,
                severity: p.threat_level,
                indicators: p.indicators,
                metadata: p.metadata,
            }).collect(),
            metadata: HashMap::new(),
        };

        if result.attack_detected {
            info!("Attack detected from IP: {} - Type: {:?}, Risk: {:.2}, Confidence: {:.2}", 
                  context.ip_address, result.attack_type, risk_score, max_confidence);
        }

        Ok(result)
    }

    /// Calculate baseline risk score using heuristic algorithms
    async fn calculate_baseline_risk_score(&self, context: &AnalysisContext, ip_reputation: &crate::infra::cache::security_cache::IpThreatData, failed_attempts: i64) -> f64 {
        let mut risk_score = 0.0;

        // IP reputation factor (0-3 points)
        if ip_reputation.is_malicious {
            risk_score += 3.0;
        } else if ip_reputation.is_vpn || ip_reputation.is_proxy {
            risk_score += 1.5;
        }
        risk_score += (1.0 - ip_reputation.reputation_score) * 2.0;

        // Failed attempts factor (0-4 points)
        risk_score += match failed_attempts {
            0..=2 => 0.0,
            3..=5 => 1.0,
            6..=10 => 2.0,
            11..=20 => 3.0,
            _ => 4.0,
        };

        // Endpoint sensitivity factor (0-2 points)
        risk_score += match context.endpoint.as_str() {
            "/auth/login" | "/api/auth/session" => 2.0,
            "/auth/register" | "/auth/forgot-password" => 1.5,
            "/api/admin" => 2.0,
            _ if context.endpoint.contains("/admin") => 1.5,
            _ => 0.5,
        };

        // Time-based factor (0-1 point)
        let hour = context.timestamp.hour();
        if (0..=6).contains(&hour) || (22..=23).contains(&hour) {
            risk_score += 1.0; // Suspicious during off-hours
        }

        // Response code factor (0-1 point)
        if let Some(code) = context.response_code {
            match code {
                401 | 403 => risk_score += 1.0,
                429 => risk_score += 0.5,
                _ => {}
            }
        }

        // Geographic factor (0-1 point)
        if let Some(geo) = &context.geolocation {
            if let Some(country) = &geo.country_code {
                // High-risk countries (this would be configurable)
                if ["CN", "RU", "KP", "IR"].contains(&country.as_str()) {
                    risk_score += 1.0;
                }
            }
        }

        risk_score.min(10.0)
    }

    /// Calculate threat level based on risk score and confidence
    async fn calculate_threat_level(&self, risk_score: f64, confidence: f64) -> ThreatLevel {
        match (risk_score, confidence) {
            (r, c) if r >= 8.0 || c >= 0.9 => ThreatLevel::Critical,
            (r, c) if r >= 6.0 || c >= 0.7 => ThreatLevel::High,
            (r, c) if r >= 4.0 || c >= 0.5 => ThreatLevel::Medium,
            _ => ThreatLevel::Low,
        }
    }

    /// Determine appropriate response actions based on threat assessment
    async fn determine_response_actions(&self, threat_level: &ThreatLevel, risk_score: f64, patterns: &[PatternDetection]) -> Result<Vec<ResponseAction>, DetectionError> {
        let thresholds = self.thresholds.read().await;
        let mut actions = Vec::new();

        match threat_level {
            ThreatLevel::Critical => {
                actions.push(ResponseAction::Block);
                actions.push(ResponseAction::Alert);
            }
            ThreatLevel::High => {
                if risk_score >= thresholds.risk_score_block_threshold {
                    actions.push(ResponseAction::Block);
                } else {
                    actions.push(ResponseAction::RateLimit);
                    actions.push(ResponseAction::RequireMfa);
                }
                actions.push(ResponseAction::Alert);
            }
            ThreatLevel::Medium => {
                if risk_score >= thresholds.risk_score_captcha_threshold {
                    actions.push(ResponseAction::RequireCaptcha);
                }
                if risk_score >= thresholds.risk_score_mfa_threshold {
                    actions.push(ResponseAction::RequireMfa);
                }
                actions.push(ResponseAction::RateLimit);
                actions.push(ResponseAction::Monitor);
            }
            ThreatLevel::Low => {
                actions.push(ResponseAction::Monitor);
            }
        }

        // Pattern-specific actions
        for pattern in patterns {
            match pattern.pattern_type {
                PatternType::DistributedBrute => {
                    if !actions.contains(&ResponseAction::Block) {
                        actions.push(ResponseAction::Block);
                    }
                }
                PatternType::PasswordSpray => {
                    actions.push(ResponseAction::LockAccount);
                }
                PatternType::GeographicAnomaly => {
                    actions.push(ResponseAction::RequireMfa);
                }
                _ => {}
            }
        }

        if actions.is_empty() {
            actions.push(ResponseAction::None);
        }

        Ok(actions)
    }

    /// Update threat intelligence with new observations
    async fn update_threat_intelligence(&self, context: &AnalysisContext, ip_reputation: &crate::infra::cache::security_cache::IpThreatData, risk_score: f64) -> Result<(), DetectionError> {
        let mut threat_intel = self.threat_intel.write().await;
        
        let key = format!("ip:{}", context.ip_address);
        let intel = threat_intel.entry(key).or_insert_with(|| ThreatIntelligence {
            source: "internal".to_string(),
            confidence: 0.5,
            threat_types: vec![],
            indicators: HashMap::new(),
            last_updated: Utc::now(),
            ttl_seconds: Some(86400),
        });

        // Update threat intelligence based on current analysis
        intel.confidence = (intel.confidence + (risk_score / 10.0)) / 2.0;
        intel.last_updated = Utc::now();

        if risk_score > 7.0 {
            intel.threat_types.push("high_risk_activity".to_string());
        }

        if ip_reputation.is_malicious {
            intel.threat_types.push("known_malicious".to_string());
        }

        debug!("Updated threat intelligence for IP: {} - Confidence: {:.2}", 
               context.ip_address, intel.confidence);

        Ok(())
    }

    /// Get current detection statistics
    pub async fn get_detection_stats(&self, hours: i32) -> Result<SecuritySummary, DetectionError> {
        let end_time = Utc::now();
        let start_time = end_time - Duration::hours(hours as i64);

        // This would typically aggregate from database
        // For now, return a basic summary
        Ok(SecuritySummary {
            time_window: TimeWindow {
                start: start_time,
                end: end_time,
                duration_hours: hours,
            },
            total_attacks_blocked: 0,
            total_ips_blocked: 0,
            total_patterns_detected: 0,
            threat_level_distribution: HashMap::new(),
            attack_type_distribution: HashMap::new(),
            geographic_distribution: HashMap::new(),
            top_attacking_ips: vec![],
            top_targeted_endpoints: vec![],
            average_risk_score: 0.0,
            generated_at: Utc::now(),
        })
    }

    /// Update detection thresholds dynamically
    pub async fn update_thresholds(&self, new_thresholds: DetectionThresholds) {
        let mut thresholds = self.thresholds.write().await;
        *thresholds = new_thresholds;
        info!("Updated detection thresholds");
    }

    /// Add external threat intelligence
    pub async fn add_threat_intelligence(&self, key: String, intel: ThreatIntelligence) {
        let mut threat_intel = self.threat_intel.write().await;
        threat_intel.insert(key, intel);
        debug!("Added threat intelligence entry");
    }
}

/// Trait for implementing custom pattern analyzers
#[async_trait::async_trait]
pub trait PatternAnalyzer {
    async fn analyze(
        &self,
        context: &AnalysisContext,
        ip_reputation: &crate::infra::cache::security_cache::IpThreatData,
        failed_attempts: i64,
    ) -> Result<Option<PatternDetection>, DetectionError>;

    fn pattern_type(&self) -> PatternType;
    fn name(&self) -> &'static str;
}

/// Pattern detection result
#[derive(Debug, Clone)]
pub struct PatternDetection {
    pub pattern_type: PatternType,
    pub pattern_id: String,
    pub attack_type: AttackType,
    pub confidence: f64,
    pub threat_level: ThreatLevel,
    pub indicators: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Traditional brute force analyzer
pub struct BruteForceAnalyzer {
    time_window_minutes: i64,
    threshold_attempts: i64,
}

impl BruteForceAnalyzer {
    pub fn new() -> Self {
        Self {
            time_window_minutes: 5,
            threshold_attempts: 10,
        }
    }
}

#[async_trait::async_trait]
impl PatternAnalyzer for BruteForceAnalyzer {
    async fn analyze(
        &self,
        context: &AnalysisContext,
        _ip_reputation: &crate::infra::cache::security_cache::IpThreatData,
        failed_attempts: i64,
    ) -> Result<Option<PatternDetection>, DetectionError> {
        if failed_attempts >= self.threshold_attempts {
            let confidence = (failed_attempts as f64 / (self.threshold_attempts as f64 * 2.0)).min(1.0);
            
            return Ok(Some(PatternDetection {
                pattern_type: PatternType::DistributedBrute,
                pattern_id: format!("brute_force_{}", context.ip_address),
                attack_type: AttackType::BruteForce,
                confidence,
                threat_level: if failed_attempts > 20 { ThreatLevel::High } else { ThreatLevel::Medium },
                indicators: vec![
                    format!("Failed attempts: {}", failed_attempts),
                    format!("Time window: {} minutes", self.time_window_minutes),
                ],
                metadata: Some(serde_json::json!({
                    "failed_attempts": failed_attempts,
                    "threshold": self.threshold_attempts,
                    "time_window_minutes": self.time_window_minutes
                })),
            }));
        }

        Ok(None)
    }

    fn pattern_type(&self) -> PatternType {
        PatternType::DistributedBrute
    }

    fn name(&self) -> &'static str {
        "BruteForceAnalyzer"
    }
}

// Implement other analyzers similarly...
pub struct PasswordSprayAnalyzer;
impl PasswordSprayAnalyzer { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl PatternAnalyzer for PasswordSprayAnalyzer {
    async fn analyze(&self, _context: &AnalysisContext, _ip_reputation: &crate::infra::cache::security_cache::IpThreatData, _failed_attempts: i64) -> Result<Option<PatternDetection>, DetectionError> {
        // Implementation would analyze for password spray patterns
        Ok(None)
    }
    fn pattern_type(&self) -> PatternType { PatternType::PasswordSpray }
    fn name(&self) -> &'static str { "PasswordSprayAnalyzer" }
}

pub struct CredentialStuffingAnalyzer;
impl CredentialStuffingAnalyzer { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl PatternAnalyzer for CredentialStuffingAnalyzer {
    async fn analyze(&self, _context: &AnalysisContext, _ip_reputation: &crate::infra::cache::security_cache::IpThreatData, _failed_attempts: i64) -> Result<Option<PatternDetection>, DetectionError> {
        Ok(None)
    }
    fn pattern_type(&self) -> PatternType { PatternType::DistributedBrute }
    fn name(&self) -> &'static str { "CredentialStuffingAnalyzer" }
}

pub struct SlowBruteAnalyzer;
impl SlowBruteAnalyzer { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl PatternAnalyzer for SlowBruteAnalyzer {
    async fn analyze(&self, _context: &AnalysisContext, _ip_reputation: &crate::infra::cache::security_cache::IpThreatData, _failed_attempts: i64) -> Result<Option<PatternDetection>, DetectionError> {
        Ok(None)
    }
    fn pattern_type(&self) -> PatternType { PatternType::TimeBasedAnomaly }
    fn name(&self) -> &'static str { "SlowBruteAnalyzer" }
}

pub struct DistributedAttackAnalyzer;
impl DistributedAttackAnalyzer { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl PatternAnalyzer for DistributedAttackAnalyzer {
    async fn analyze(&self, _context: &AnalysisContext, _ip_reputation: &crate::infra::cache::security_cache::IpThreatData, _failed_attempts: i64) -> Result<Option<PatternDetection>, DetectionError> {
        Ok(None)
    }
    fn pattern_type(&self) -> PatternType { PatternType::DistributedBrute }
    fn name(&self) -> &'static str { "DistributedAttackAnalyzer" }
}

pub struct GeographicAnomalyAnalyzer;
impl GeographicAnomalyAnalyzer { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl PatternAnalyzer for GeographicAnomalyAnalyzer {
    async fn analyze(&self, _context: &AnalysisContext, _ip_reputation: &crate::infra::cache::security_cache::IpThreatData, _failed_attempts: i64) -> Result<Option<PatternDetection>, DetectionError> {
        Ok(None)
    }
    fn pattern_type(&self) -> PatternType { PatternType::GeographicAnomaly }
    fn name(&self) -> &'static str { "GeographicAnomalyAnalyzer" }
}

pub struct DeviceAnomalyAnalyzer;
impl DeviceAnomalyAnalyzer { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl PatternAnalyzer for DeviceAnomalyAnalyzer {
    async fn analyze(&self, _context: &AnalysisContext, _ip_reputation: &crate::infra::cache::security_cache::IpThreatData, _failed_attempts: i64) -> Result<Option<PatternDetection>, DetectionError> {
        Ok(None)
    }
    fn pattern_type(&self) -> PatternType { PatternType::DeviceAnomaly }
    fn name(&self) -> &'static str { "DeviceAnomalyAnalyzer" }
}

pub struct BehavioralAnomalyAnalyzer;
impl BehavioralAnomalyAnalyzer { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl PatternAnalyzer for BehavioralAnomalyAnalyzer {
    async fn analyze(&self, _context: &AnalysisContext, _ip_reputation: &crate::infra::cache::security_cache::IpThreatData, _failed_attempts: i64) -> Result<Option<PatternDetection>, DetectionError> {
        Ok(None)
    }
    fn pattern_type(&self) -> PatternType { PatternType::BehavioralAnomaly }
    fn name(&self) -> &'static str { "BehavioralAnomalyAnalyzer" }
}

/// ML-based threat scoring model
pub struct ThreatScoringModel {
    // Model parameters would be loaded from trained ML model
    feature_weights: HashMap<String, f64>,
}

impl ThreatScoringModel {
    pub fn new() -> Self {
        let mut feature_weights = HashMap::new();
        
        // Example feature weights (would be learned from training data)
        feature_weights.insert("request_frequency".to_string(), 0.15);
        feature_weights.insert("time_between_requests".to_string(), 0.12);
        feature_weights.insert("user_agent_entropy".to_string(), 0.08);
        feature_weights.insert("geographic_distance".to_string(), 0.10);
        feature_weights.insert("device_consistency_score".to_string(), 0.11);
        feature_weights.insert("behavioral_deviation_score".to_string(), 0.13);
        feature_weights.insert("ip_reputation_score".to_string(), 0.18);
        feature_weights.insert("time_of_day_anomaly".to_string(), 0.07);
        feature_weights.insert("endpoint_diversity".to_string(), 0.06);

        Self { feature_weights }
    }

    pub async fn calculate_risk_score(
        &self,
        context: &AnalysisContext,
        ip_reputation: &crate::infra::cache::security_cache::IpThreatData,
        failed_attempts: i64,
    ) -> Result<f64, DetectionError> {
        let features = self.extract_features(context, ip_reputation, failed_attempts).await?;
        
        let mut risk_score = 0.0;
        for (feature_name, feature_value) in &features {
            if let Some(weight) = self.feature_weights.get(feature_name) {
                risk_score += feature_value * weight;
            }
        }

        // Apply sigmoid function to normalize to 0-10 scale
        let normalized_score = 10.0 / (1.0 + (-risk_score).exp());
        
        Ok(normalized_score)
    }

    async fn extract_features(
        &self,
        context: &AnalysisContext,
        ip_reputation: &crate::infra::cache::security_cache::IpThreatData,
        failed_attempts: i64,
    ) -> Result<HashMap<String, f64>, DetectionError> {
        let mut features = HashMap::new();

        // Request frequency (normalized)
        features.insert("request_frequency".to_string(), (failed_attempts as f64 / 60.0).min(10.0));

        // IP reputation score
        features.insert("ip_reputation_score".to_string(), 10.0 - ip_reputation.reputation_score);

        // Time of day anomaly
        let hour = context.timestamp.hour() as f64;
        let time_anomaly = if (2.0..=6.0).contains(&hour) { 8.0 } else { 2.0 };
        features.insert("time_of_day_anomaly".to_string(), time_anomaly);

        // User agent entropy (simplified)
        if let Some(ua) = &context.user_agent {
            let entropy = self.calculate_entropy(ua);
            features.insert("user_agent_entropy".to_string(), entropy);
        } else {
            features.insert("user_agent_entropy".to_string(), 8.0); // No user agent is suspicious
        }

        // Additional features would be calculated here...

        Ok(features)
    }

    fn calculate_entropy(&self, text: &str) -> f64 {
        // Simplified entropy calculation
        let mut char_freq = HashMap::new();
        for c in text.chars() {
            *char_freq.entry(c).or_insert(0) += 1;
        }

        let text_len = text.len() as f64;
        let mut entropy = 0.0;
        for freq in char_freq.values() {
            let p = *freq as f64 / text_len;
            entropy -= p * p.log2();
        }

        entropy.min(10.0)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    fn create_test_context() -> AnalysisContext {
        AnalysisContext {
            ip_address: "192.168.1.100".parse().unwrap(),
            user_id: Some("test-user".to_string()),
            session_id: Some("test-session".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            endpoint: "/auth/login".to_string(),
            method: "POST".to_string(),
            geolocation: None,
            device_fingerprint: None,
            request_headers: HashMap::new(),
            request_body_hash: None,
            timestamp: Utc::now(),
            success: false,
            response_code: Some(401),
            response_time_ms: Some(150),
            correlation_id: None,
        }
    }

    #[tokio::test]
    async fn test_brute_force_analyzer() {
        let analyzer = BruteForceAnalyzer::new();
        let context = create_test_context();
        let ip_reputation = crate::infra::cache::security_cache::IpThreatData {
            ip_address: "192.168.1.100".to_string(),
            is_malicious: false,
            is_vpn: false,
            is_proxy: false,
            reputation_score: 0.5,
            country_code: None,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            threat_types: vec![],
            blocked: false,
        };

        // Test with low attempts - should not detect
        let result = analyzer.analyze(&context, &ip_reputation, 5).await.unwrap();
        assert!(result.is_none());

        // Test with high attempts - should detect
        let result = analyzer.analyze(&context, &ip_reputation, 15).await.unwrap();
        assert!(result.is_some());
        
        let detection = result.unwrap();
        assert_eq!(detection.attack_type, AttackType::BruteForce);
        assert!(detection.confidence > 0.5);
    }

    #[tokio::test]
    async fn test_threat_scoring_model() {
        let model = ThreatScoringModel::new();
        let context = create_test_context();
        let ip_reputation = crate::infra::cache::security_cache::IpThreatData {
            ip_address: "192.168.1.100".to_string(),
            is_malicious: false,
            is_vpn: false,
            is_proxy: false,
            reputation_score: 0.5,
            country_code: None,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            threat_types: vec![],
            blocked: false,
        };

        let score = model.calculate_risk_score(&context, &ip_reputation, 10).await.unwrap();
        assert!(score >= 0.0 && score <= 10.0);
    }
}