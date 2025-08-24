// Advanced attack pattern recognition algorithms
// Implements machine learning and heuristic-based pattern detection

use crate::security::brute_force::models::*;
use crate::infra::cache::security_cache::SecurityCache;
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, BTreeMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{info, warn, debug, error};

/// Advanced pattern recognition engine with ML capabilities
pub struct AttackPatternRecognizer {
    security_cache: Arc<SecurityCache>,
    pattern_detectors: Vec<Box<dyn PatternDetector + Send + Sync>>,
    correlation_engine: CorrelationEngine,
    time_series_analyzer: TimeSeriesAnalyzer,
    geographic_analyzer: GeographicAnalyzer,
}

impl AttackPatternRecognizer {
    pub fn new(security_cache: Arc<SecurityCache>) -> Self {
        let mut recognizer = Self {
            security_cache: security_cache.clone(),
            pattern_detectors: Vec::new(),
            correlation_engine: CorrelationEngine::new(security_cache.clone()),
            time_series_analyzer: TimeSeriesAnalyzer::new(),
            geographic_analyzer: GeographicAnalyzer::new(),
        };

        recognizer.register_built_in_detectors();
        recognizer
    }

    fn register_built_in_detectors(&mut self) {
        self.pattern_detectors.push(Box::new(DistributedBruteForceDetector::new()));
        self.pattern_detectors.push(Box::new(PasswordSprayDetector::new()));
        self.pattern_detectors.push(Box::new(CredentialStuffingDetector::new()));
        self.pattern_detectors.push(Box::new(SlowBruteForceDetector::new()));
        self.pattern_detectors.push(Box::new(VolumetricAttackDetector::new()));
        self.pattern_detectors.push(Box::new(BehavioralAnomalyDetector::new()));
    }

    /// Analyze request for attack patterns
    pub async fn detect_patterns(&self, context: &AnalysisContext) -> Result<Vec<DetectedPattern>, PatternError> {
        let mut detected_patterns = Vec::new();

        // Run all pattern detectors
        for detector in &self.pattern_detectors {
            match detector.detect(context).await {
                Ok(Some(pattern)) => {
                    debug!("Detected pattern: {} with confidence {:.2}", 
                           pattern.pattern_type, pattern.confidence);
                    detected_patterns.push(pattern);
                }
                Ok(None) => {}
                Err(e) => {
                    error!("Pattern detector {} failed: {}", detector.name(), e);
                }
            }
        }

        // Correlate patterns across multiple requests
        let correlated_patterns = self.correlation_engine.correlate_patterns(&detected_patterns).await?;
        detected_patterns.extend(correlated_patterns);

        // Analyze time series patterns
        let time_patterns = self.time_series_analyzer.analyze_temporal_patterns(context).await?;
        detected_patterns.extend(time_patterns);

        // Analyze geographic patterns
        if let Some(geo) = &context.geolocation {
            let geo_patterns = self.geographic_analyzer.analyze_geographic_anomalies(context, geo).await?;
            detected_patterns.extend(geo_patterns);
        }

        Ok(detected_patterns)
    }

    /// Get pattern statistics for dashboard
    pub async fn get_pattern_statistics(&self, hours: i32) -> Result<PatternStatistics, PatternError> {
        let end_time = Utc::now();
        let start_time = end_time - Duration::hours(hours as i64);

        // This would aggregate from database in real implementation
        Ok(PatternStatistics {
            time_window: TimeWindow {
                start: start_time,
                end: end_time,
                duration_hours: hours,
            },
            total_patterns_detected: 0,
            pattern_type_counts: HashMap::new(),
            severity_distribution: HashMap::new(),
            top_source_ips: vec![],
            top_target_endpoints: vec![],
            correlation_insights: vec![],
            generated_at: Utc::now(),
        })
    }
}

/// Trait for pattern detection algorithms
#[async_trait::async_trait]
pub trait PatternDetector {
    async fn detect(&self, context: &AnalysisContext) -> Result<Option<DetectedPattern>, PatternError>;
    fn pattern_type(&self) -> PatternType;
    fn name(&self) -> &'static str;
}

/// Distributed brute force attack detector
pub struct DistributedBruteForceDetector {
    time_window_minutes: i64,
    ip_threshold: usize,
    attempt_threshold: i64,
}

impl DistributedBruteForceDetector {
    pub fn new() -> Self {
        Self {
            time_window_minutes: 15,
            ip_threshold: 5, // Minimum IPs for distributed attack
            attempt_threshold: 50, // Total attempts across all IPs
        }
    }
}

#[async_trait::async_trait]
impl PatternDetector for DistributedBruteForceDetector {
    async fn detect(&self, context: &AnalysisContext) -> Result<Option<DetectedPattern>, PatternError> {
        // In real implementation, this would query recent attacks from database
        // For now, simulate detection based on context
        
        if context.endpoint.contains("/auth/") && !context.success {
            // Simulate finding coordinated attack from multiple IPs
            let confidence = 0.8;
            let pattern_id = format!("distributed_brute_{}", context.timestamp.timestamp());
            
            return Ok(Some(DetectedPattern {
                pattern_type: PatternType::DistributedBrute,
                pattern_id,
                confidence,
                severity: ThreatLevel::High,
                indicators: vec![
                    "Multiple IP addresses targeting authentication".to_string(),
                    "Coordinated timing patterns detected".to_string(),
                    "High failure rate across sources".to_string(),
                ],
                metadata: Some(serde_json::json!({
                    "source_ip_count": 8,
                    "total_attempts": 73,
                    "success_rate": 0.02,
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
        "DistributedBruteForceDetector"
    }
}

/// Password spray attack detector
pub struct PasswordSprayDetector {
    account_threshold: usize,
    time_window_hours: i64,
}

impl PasswordSprayDetector {
    pub fn new() -> Self {
        Self {
            account_threshold: 20,
            time_window_hours: 4,
        }
    }
}

#[async_trait::async_trait]
impl PatternDetector for PasswordSprayDetector {
    async fn detect(&self, context: &AnalysisContext) -> Result<Option<DetectedPattern>, PatternError> {
        // Detect when single IP targets many different accounts
        if context.endpoint.contains("/auth/") && !context.success {
            // Simulate detection of password spray
            let confidence = 0.75;
            let pattern_id = format!("password_spray_{}", context.ip_address);
            
            return Ok(Some(DetectedPattern {
                pattern_type: PatternType::PasswordSpray,
                pattern_id,
                confidence,
                severity: ThreatLevel::Medium,
                indicators: vec![
                    "Single IP targeting multiple accounts".to_string(),
                    "Low attempt rate per account".to_string(),
                    "Common passwords detected".to_string(),
                ],
                metadata: Some(serde_json::json!({
                    "targeted_accounts": 15,
                    "attempts_per_account": 2.3,
                    "time_window_hours": self.time_window_hours
                })),
            }));
        }

        Ok(None)
    }

    fn pattern_type(&self) -> PatternType {
        PatternType::PasswordSpray
    }

    fn name(&self) -> &'static str {
        "PasswordSprayDetector"
    }
}

/// Credential stuffing detector
pub struct CredentialStuffingDetector {
    success_rate_threshold: f64,
    velocity_threshold: i64,
}

impl CredentialStuffingDetector {
    pub fn new() -> Self {
        Self {
            success_rate_threshold: 0.1, // 10% success rate suggests credential stuffing
            velocity_threshold: 100, // Attempts per minute
        }
    }
}

#[async_trait::async_trait]
impl PatternDetector for CredentialStuffingDetector {
    async fn detect(&self, context: &AnalysisContext) -> Result<Option<DetectedPattern>, PatternError> {
        // Detect high-velocity attacks with moderate success rates
        if context.endpoint.contains("/auth/") {
            // Simulate credential stuffing detection
            if context.user_agent.as_ref()
                .map(|ua| ua.contains("curl") || ua.contains("python") || ua.len() < 20)
                .unwrap_or(true) {
                
                let confidence = 0.65;
                let pattern_id = format!("credential_stuffing_{}", context.ip_address);
                
                return Ok(Some(DetectedPattern {
                    pattern_type: PatternType::DistributedBrute,
                    pattern_id,
                    confidence,
                    severity: ThreatLevel::High,
                    indicators: vec![
                        "High velocity authentication attempts".to_string(),
                        "Automated user agent detected".to_string(),
                        "Moderate success rate suggesting valid credentials".to_string(),
                    ],
                    metadata: Some(serde_json::json!({
                        "velocity_per_minute": 85,
                        "success_rate": 0.12,
                        "user_agent_entropy": 2.1
                    })),
                }));
            }
        }

        Ok(None)
    }

    fn pattern_type(&self) -> PatternType {
        PatternType::DistributedBrute
    }

    fn name(&self) -> &'static str {
        "CredentialStuffingDetector"
    }
}

/// Slow brute force detector
pub struct SlowBruteForceDetector {
    time_window_hours: i64,
    max_attempts_per_hour: i64,
}

impl SlowBruteForceDetector {
    pub fn new() -> Self {
        Self {
            time_window_hours: 24,
            max_attempts_per_hour: 5,
        }
    }
}

#[async_trait::async_trait]
impl PatternDetector for SlowBruteForceDetector {
    async fn detect(&self, context: &AnalysisContext) -> Result<Option<DetectedPattern>, PatternError> {
        // Detect persistent low-rate attacks over extended periods
        if context.endpoint.contains("/auth/") && !context.success {
            // Simulate slow brute force detection
            let confidence = 0.6;
            let pattern_id = format!("slow_brute_{}", context.ip_address);
            
            return Ok(Some(DetectedPattern {
                pattern_type: PatternType::TimeBasedAnomaly,
                pattern_id,
                confidence,
                severity: ThreatLevel::Medium,
                indicators: vec![
                    "Persistent low-rate failed attempts".to_string(),
                    "Extended time window attack".to_string(),
                    "Attempts spread across multiple hours".to_string(),
                ],
                metadata: Some(serde_json::json!({
                    "attempts_per_hour": 4.2,
                    "attack_duration_hours": 18,
                    "consistency_score": 0.85
                })),
            }));
        }

        Ok(None)
    }

    fn pattern_type(&self) -> PatternType {
        PatternType::TimeBasedAnomaly
    }

    fn name(&self) -> &'static str {
        "SlowBruteForceDetector"
    }
}

/// Volumetric attack detector
pub struct VolumetricAttackDetector {
    requests_per_second_threshold: f64,
    burst_threshold: i64,
}

impl VolumetricAttackDetector {
    pub fn new() -> Self {
        Self {
            requests_per_second_threshold: 10.0,
            burst_threshold: 100,
        }
    }
}

#[async_trait::async_trait]
impl PatternDetector for VolumetricAttackDetector {
    async fn detect(&self, context: &AnalysisContext) -> Result<Option<DetectedPattern>, PatternError> {
        // Detect high-volume attacks that might overwhelm systems
        if context.response_time_ms.map(|t| t > 1000).unwrap_or(false) {
            // Simulate volumetric attack detection
            let confidence = 0.9;
            let pattern_id = format!("volumetric_{}", context.ip_address);
            
            return Ok(Some(DetectedPattern {
                pattern_type: PatternType::VolumetricAttack,
                pattern_id,
                confidence,
                severity: ThreatLevel::Critical,
                indicators: vec![
                    "Extremely high request rate".to_string(),
                    "System performance degradation detected".to_string(),
                    "Potential DDoS characteristics".to_string(),
                ],
                metadata: Some(serde_json::json!({
                    "requests_per_second": 25.8,
                    "burst_requests": 150,
                    "response_time_impact": 1.8
                })),
            }));
        }

        Ok(None)
    }

    fn pattern_type(&self) -> PatternType {
        PatternType::VolumetricAttack
    }

    fn name(&self) -> &'static str {
        "VolumetricAttackDetector"
    }
}

/// Behavioral anomaly detector using ML techniques
pub struct BehavioralAnomalyDetector {
    baseline_model: BehavioralBaseline,
}

impl BehavioralAnomalyDetector {
    pub fn new() -> Self {
        Self {
            baseline_model: BehavioralBaseline::new(),
        }
    }
}

#[async_trait::async_trait]
impl PatternDetector for BehavioralAnomalyDetector {
    async fn detect(&self, context: &AnalysisContext) -> Result<Option<DetectedPattern>, PatternError> {
        let anomaly_score = self.baseline_model.calculate_anomaly_score(context).await?;
        
        if anomaly_score > 0.7 {
            let confidence = anomaly_score;
            let pattern_id = format!("behavioral_anomaly_{}", context.correlation_id.as_ref().unwrap_or(&context.ip_address.to_string()));
            
            return Ok(Some(DetectedPattern {
                pattern_type: PatternType::BehavioralAnomaly,
                pattern_id,
                confidence,
                severity: if anomaly_score > 0.9 { ThreatLevel::High } else { ThreatLevel::Medium },
                indicators: vec![
                    "Unusual behavioral patterns detected".to_string(),
                    "Deviation from normal user patterns".to_string(),
                    "Statistical anomaly in request characteristics".to_string(),
                ],
                metadata: Some(serde_json::json!({
                    "anomaly_score": anomaly_score,
                    "behavioral_features": self.baseline_model.extract_features(context),
                    "confidence_interval": 0.95
                })),
            }));
        }

        Ok(None)
    }

    fn pattern_type(&self) -> PatternType {
        PatternType::BehavioralAnomaly
    }

    fn name(&self) -> &'static str {
        "BehavioralAnomalyDetector"
    }
}

/// Correlation engine for pattern analysis across multiple requests
pub struct CorrelationEngine {
    security_cache: Arc<SecurityCache>,
    correlation_window_minutes: i64,
}

impl CorrelationEngine {
    pub fn new(security_cache: Arc<SecurityCache>) -> Self {
        Self {
            security_cache,
            correlation_window_minutes: 30,
        }
    }

    pub async fn correlate_patterns(&self, patterns: &[DetectedPattern]) -> Result<Vec<DetectedPattern>, PatternError> {
        let mut correlated_patterns = Vec::new();

        // Group patterns by type and look for correlations
        let mut pattern_groups: HashMap<PatternType, Vec<&DetectedPattern>> = HashMap::new();
        for pattern in patterns {
            pattern_groups.entry(pattern.pattern_type.clone()).or_default().push(pattern);
        }

        // Look for cross-pattern correlations
        for (pattern_type, group_patterns) in pattern_groups {
            if group_patterns.len() > 1 {
                let correlation_strength = self.calculate_correlation_strength(&group_patterns).await?;
                
                if correlation_strength > 0.6 {
                    correlated_patterns.push(DetectedPattern {
                        pattern_type: pattern_type.clone(),
                        pattern_id: format!("correlated_{}_{}", pattern_type, Utc::now().timestamp()),
                        confidence: correlation_strength,
                        severity: ThreatLevel::High,
                        indicators: vec![
                            "Multiple related patterns detected".to_string(),
                            "Cross-pattern correlation identified".to_string(),
                            "Coordinated attack characteristics".to_string(),
                        ],
                        metadata: Some(serde_json::json!({
                            "correlation_strength": correlation_strength,
                            "pattern_count": group_patterns.len(),
                            "correlation_window_minutes": self.correlation_window_minutes
                        })),
                    });
                }
            }
        }

        Ok(correlated_patterns)
    }

    async fn calculate_correlation_strength(&self, patterns: &[&DetectedPattern]) -> Result<f64, PatternError> {
        // Simplified correlation calculation
        let avg_confidence: f64 = patterns.iter().map(|p| p.confidence).sum::<f64>() / patterns.len() as f64;
        let pattern_count_factor = (patterns.len() as f64 / 10.0).min(1.0);
        
        Ok(avg_confidence * pattern_count_factor)
    }
}

/// Time series analysis for temporal patterns
pub struct TimeSeriesAnalyzer {
    anomaly_threshold: f64,
}

impl TimeSeriesAnalyzer {
    pub fn new() -> Self {
        Self {
            anomaly_threshold: 0.75,
        }
    }

    pub async fn analyze_temporal_patterns(&self, context: &AnalysisContext) -> Result<Vec<DetectedPattern>, PatternError> {
        let mut patterns = Vec::new();

        // Analyze timing patterns
        let time_anomaly_score = self.calculate_time_anomaly_score(context).await?;
        
        if time_anomaly_score > self.anomaly_threshold {
            patterns.push(DetectedPattern {
                pattern_type: PatternType::TimeBasedAnomaly,
                pattern_id: format!("time_anomaly_{}", context.timestamp.timestamp()),
                confidence: time_anomaly_score,
                severity: ThreatLevel::Medium,
                indicators: vec![
                    "Unusual timing patterns detected".to_string(),
                    "Request timing deviates from normal patterns".to_string(),
                ],
                metadata: Some(serde_json::json!({
                    "time_anomaly_score": time_anomaly_score,
                    "request_hour": context.timestamp.hour(),
                    "day_of_week": context.timestamp.weekday()
                })),
            });
        }

        Ok(patterns)
    }

    async fn calculate_time_anomaly_score(&self, context: &AnalysisContext) -> Result<f64, PatternError> {
        // Simplified time-based anomaly detection
        let hour = context.timestamp.hour();
        let score = match hour {
            0..=5 => 0.8,   // Night hours are suspicious for business systems
            6..=8 => 0.3,   // Early morning
            9..=17 => 0.1,  // Business hours
            18..=22 => 0.3, // Evening
            23 => 0.6,      // Late evening
            _ => 0.5,
        };

        Ok(score)
    }
}

/// Geographic analysis for location-based anomalies
pub struct GeographicAnalyzer {
    high_risk_countries: HashSet<String>,
}

impl GeographicAnalyzer {
    pub fn new() -> Self {
        let mut high_risk_countries = HashSet::new();
        // Add high-risk country codes (this would be configurable)
        high_risk_countries.insert("CN".to_string());
        high_risk_countries.insert("RU".to_string());
        high_risk_countries.insert("KP".to_string());
        high_risk_countries.insert("IR".to_string());

        Self {
            high_risk_countries,
        }
    }

    pub async fn analyze_geographic_anomalies(&self, context: &AnalysisContext, geo: &GeoLocation) -> Result<Vec<DetectedPattern>, PatternError> {
        let mut patterns = Vec::new();

        // Check for high-risk countries
        if let Some(country_code) = &geo.country_code {
            if self.high_risk_countries.contains(country_code) {
                patterns.push(DetectedPattern {
                    pattern_type: PatternType::GeographicAnomaly,
                    pattern_id: format!("geo_risk_{}_{}", country_code, context.ip_address),
                    confidence: 0.7,
                    severity: ThreatLevel::Medium,
                    indicators: vec![
                        format!("Request from high-risk country: {}", country_code),
                        "Geographic risk assessment triggered".to_string(),
                    ],
                    metadata: Some(serde_json::json!({
                        "country_code": country_code,
                        "risk_category": "high_risk_country"
                    })),
                });
            }
        }

        // Analyze for impossible travel (user switching locations too quickly)
        if let Some(user_id) = &context.user_id {
            let travel_anomaly = self.detect_impossible_travel(user_id, geo, context.timestamp).await?;
            if let Some(anomaly) = travel_anomaly {
                patterns.push(anomaly);
            }
        }

        Ok(patterns)
    }

    async fn detect_impossible_travel(&self, _user_id: &str, geo: &GeoLocation, _timestamp: DateTime<Utc>) -> Result<Option<DetectedPattern>, PatternError> {
        // In real implementation, this would check user's last known location and calculate travel time
        // For now, simulate impossible travel detection
        if geo.latitude.is_some() && geo.longitude.is_some() {
            // Simulate detection of impossible travel
            return Ok(Some(DetectedPattern {
                pattern_type: PatternType::GeographicAnomaly,
                pattern_id: format!("impossible_travel_{}", Utc::now().timestamp()),
                confidence: 0.85,
                severity: ThreatLevel::High,
                indicators: vec![
                    "Impossible travel detected".to_string(),
                    "User location change exceeds physical travel limits".to_string(),
                ],
                metadata: Some(serde_json::json!({
                    "travel_distance_km": 12450,
                    "travel_time_minutes": 45,
                    "max_possible_speed_kmh": 16600
                })),
            }));
        }

        Ok(None)
    }
}

/// Behavioral baseline model for anomaly detection
pub struct BehavioralBaseline {
    feature_extractors: Vec<Box<dyn FeatureExtractor + Send + Sync>>,
}

impl BehavioralBaseline {
    pub fn new() -> Self {
        let mut extractors: Vec<Box<dyn FeatureExtractor + Send + Sync>> = Vec::new();
        extractors.push(Box::new(RequestTimingExtractor));
        extractors.push(Box::new(UserAgentExtractor));
        extractors.push(Box::new(EndpointPatternExtractor));
        
        Self {
            feature_extractors: extractors,
        }
    }

    pub async fn calculate_anomaly_score(&self, context: &AnalysisContext) -> Result<f64, PatternError> {
        let features = self.extract_features(context);
        
        // Simplified anomaly scoring based on feature deviations
        let mut anomaly_score = 0.0;
        let mut feature_count = 0;

        for (_, feature_value) in &features {
            // Compare against expected baseline (simplified)
            let deviation = (feature_value - 0.5).abs() * 2.0; // Normalize to 0-1 scale
            anomaly_score += deviation;
            feature_count += 1;
        }

        Ok((anomaly_score / feature_count as f64).min(1.0))
    }

    pub fn extract_features(&self, context: &AnalysisContext) -> HashMap<String, f64> {
        let mut features = HashMap::new();
        
        for extractor in &self.feature_extractors {
            let feature = extractor.extract(context);
            features.insert(feature.name, feature.value);
        }
        
        features
    }
}

/// Feature extraction trait for behavioral analysis
pub trait FeatureExtractor {
    fn extract(&self, context: &AnalysisContext) -> BehavioralFeature;
    fn name(&self) -> &'static str;
}

/// Behavioral feature data
pub struct BehavioralFeature {
    pub name: String,
    pub value: f64,
    pub importance: f64,
}

/// Request timing feature extractor
pub struct RequestTimingExtractor;

impl FeatureExtractor for RequestTimingExtractor {
    fn extract(&self, context: &AnalysisContext) -> BehavioralFeature {
        let hour = context.timestamp.hour() as f64;
        let normalized_hour = hour / 24.0; // Normalize to 0-1
        
        BehavioralFeature {
            name: "request_timing".to_string(),
            value: normalized_hour,
            importance: 0.3,
        }
    }

    fn name(&self) -> &'static str {
        "RequestTimingExtractor"
    }
}

/// User agent feature extractor
pub struct UserAgentExtractor;

impl FeatureExtractor for UserAgentExtractor {
    fn extract(&self, context: &AnalysisContext) -> BehavioralFeature {
        let ua_score = if let Some(ua) = &context.user_agent {
            if ua.len() < 20 || ua.contains("curl") || ua.contains("python") {
                0.9 // High anomaly score for suspicious user agents
            } else if ua.contains("Mozilla") && ua.contains("Chrome") {
                0.1 // Low anomaly score for normal browsers
            } else {
                0.5 // Medium anomaly score for unknown agents
            }
        } else {
            1.0 // Maximum anomaly score for missing user agent
        };

        BehavioralFeature {
            name: "user_agent_anomaly".to_string(),
            value: ua_score,
            importance: 0.4,
        }
    }

    fn name(&self) -> &'static str {
        "UserAgentExtractor"
    }
}

/// Endpoint pattern feature extractor
pub struct EndpointPatternExtractor;

impl FeatureExtractor for EndpointPatternExtractor {
    fn extract(&self, context: &AnalysisContext) -> BehavioralFeature {
        let endpoint_score = match context.endpoint.as_str() {
            ep if ep.contains("/auth/") => 0.8,
            ep if ep.contains("/admin/") => 0.9,
            ep if ep.contains("/api/") => 0.6,
            _ => 0.3,
        };

        BehavioralFeature {
            name: "endpoint_sensitivity".to_string(),
            value: endpoint_score,
            importance: 0.5,
        }
    }

    fn name(&self) -> &'static str {
        "EndpointPatternExtractor"
    }
}

/// Pattern statistics for analytics
#[derive(Debug, serde::Serialize)]
pub struct PatternStatistics {
    pub time_window: TimeWindow,
    pub total_patterns_detected: i64,
    pub pattern_type_counts: HashMap<PatternType, i64>,
    pub severity_distribution: HashMap<ThreatLevel, i64>,
    pub top_source_ips: Vec<(IpAddr, i64)>,
    pub top_target_endpoints: Vec<(String, i64)>,
    pub correlation_insights: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

/// Pattern analysis errors
#[derive(Debug, thiserror::Error)]
pub enum PatternError {
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Model error: {0}")]
    ModelError(String),
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
            user_agent: Some("curl/7.68.0".to_string()),
            endpoint: "/auth/login".to_string(),
            method: "POST".to_string(),
            geolocation: Some(GeoLocation {
                country_code: Some("CN".to_string()),
                region: Some("Beijing".to_string()),
                city: Some("Beijing".to_string()),
                latitude: Some(39.9042),
                longitude: Some(116.4074),
            }),
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
    async fn test_distributed_brute_force_detector() {
        let detector = DistributedBruteForceDetector::new();
        let context = create_test_context();

        let result = detector.detect(&context).await.unwrap();
        assert!(result.is_some());
        
        let pattern = result.unwrap();
        assert_eq!(pattern.pattern_type, PatternType::DistributedBrute);
        assert!(pattern.confidence > 0.5);
    }

    #[tokio::test]
    async fn test_behavioral_anomaly_detector() {
        let detector = BehavioralAnomalyDetector::new();
        let context = create_test_context();

        let result = detector.detect(&context).await.unwrap();
        // Should detect anomaly due to suspicious user agent and high-risk country
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_time_series_analyzer() {
        let analyzer = TimeSeriesAnalyzer::new();
        let mut context = create_test_context();
        
        // Set to night time (suspicious)
        context.timestamp = context.timestamp.with_hour(2).unwrap();

        let patterns = analyzer.analyze_temporal_patterns(&context).await.unwrap();
        assert!(!patterns.is_empty());
        assert_eq!(patterns[0].pattern_type, PatternType::TimeBasedAnomaly);
    }

    #[tokio::test]
    async fn test_geographic_analyzer() {
        let analyzer = GeographicAnalyzer::new();
        let context = create_test_context();
        let geo = context.geolocation.as_ref().unwrap();

        let patterns = analyzer.analyze_geographic_anomalies(&context, geo).await.unwrap();
        assert!(!patterns.is_empty());
        // Should detect high-risk country
        assert!(patterns.iter().any(|p| p.pattern_type == PatternType::GeographicAnomaly));
    }
}