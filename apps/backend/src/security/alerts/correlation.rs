// Alert Correlation Engine
// Intelligent correlation and grouping of related security alerts

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use tracing::{debug, info};

use super::models::*;

/// Alert engine configuration (stub)
#[derive(Debug, Clone)]
pub struct AlertEngineConfig {
    pub correlation_window_minutes: u32,
    pub max_correlations_per_alert: usize,
    pub enable_ml_correlation: bool,
    pub max_correlation_window_minutes: u32,
}

/// Correlation types for grouping related alerts
#[derive(Debug, Clone)]
pub enum CorrelationType {
    TimeBased,
    IpBased,
    UserBased,
    SessionBased,
    PatternBased,
    Geolocation,
    ThreatIntelligence,
}

impl CorrelationType {
    pub fn to_string(&self) -> String {
        match self {
            CorrelationType::TimeBased => "TIME_BASED".to_string(),
            CorrelationType::IpBased => "IP_BASED".to_string(),
            CorrelationType::UserBased => "USER_BASED".to_string(),
            CorrelationType::SessionBased => "SESSION_BASED".to_string(),
            CorrelationType::PatternBased => "PATTERN_BASED".to_string(),
            CorrelationType::Geolocation => "GEOLOCATION".to_string(),
            CorrelationType::ThreatIntelligence => "THREAT_INTELLIGENCE".to_string(),
        }
    }
}

/// Alert correlation data
#[derive(Debug, Clone)]
pub struct AlertCorrelation {
    pub id: Uuid,
    pub correlation_id: String,
    pub correlation_type: CorrelationType,
    pub related_alerts: Vec<Uuid>,
    pub confidence_score: f64,
    pub root_cause_alert: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Correlation Engine for intelligent alert grouping
pub struct CorrelationEngine {
    config: AlertEngineConfig,
    active_correlations: std::sync::RwLock<HashMap<String, AlertCorrelation>>,
    correlation_cache: std::sync::RwLock<CorrelationCache>,
}

impl CorrelationEngine {
    pub fn new(config: AlertEngineConfig) -> Self {
        Self {
            config,
            active_correlations: std::sync::RwLock::new(HashMap::new()),
            correlation_cache: std::sync::RwLock::new(CorrelationCache::new()),
        }
    }

    /// Find correlations for a new alert
    pub async fn find_correlations(&self, alert: &SecurityAlert) -> AlertResult<Vec<AlertCorrelation>> {
        let mut correlations = Vec::new();

        // Time-based correlation
        if let Some(correlation) = self.find_time_based_correlation(alert).await? {
            correlations.push(correlation);
        }

        // IP-based correlation
        if let Some(correlation) = self.find_ip_based_correlation(alert).await? {
            correlations.push(correlation);
        }

        // User-based correlation
        if let Some(correlation) = self.find_user_based_correlation(alert).await? {
            correlations.push(correlation);
        }

        // Session-based correlation
        if let Some(correlation) = self.find_session_based_correlation(alert).await? {
            correlations.push(correlation);
        }

        // Pattern-based correlation
        if let Some(correlation) = self.find_pattern_based_correlation(alert).await? {
            correlations.push(correlation);
        }

        // Update correlation cache
        self.update_correlation_cache(alert, &correlations).await?;

        debug!("Found {} correlations for alert {}", correlations.len(), alert.id);
        Ok(correlations)
    }

    /// Find time-based correlations (alerts within time window)
    async fn find_time_based_correlation(&self, alert: &SecurityAlert) -> AlertResult<Option<AlertCorrelation>> {
        let time_window = Duration::minutes(self.config.max_correlation_window_minutes as i64);
        let window_start = alert.created_at - time_window;
        let window_end = alert.created_at + time_window;

        let cache = self.correlation_cache.read().unwrap();
        let related_alerts = cache.find_alerts_in_time_window(window_start, window_end);

        if related_alerts.len() > 1 {
            let correlation_id = format!("time_{}", alert.created_at.format("%Y%m%d_%H%M"));
            let confidence_score = self.calculate_time_correlation_score(&related_alerts);
            let correlation = AlertCorrelation {
                id: Uuid::new_v4(),
                correlation_id,
                correlation_type: CorrelationType::TimeBased,
                related_alerts,
                confidence_score,
                root_cause_alert: None,
                created_at: Utc::now(),
            };

            return Ok(Some(correlation));
        }

        Ok(None)
    }

    /// Find IP-based correlations (same IP address)
    async fn find_ip_based_correlation(&self, alert: &SecurityAlert) -> AlertResult<Option<AlertCorrelation>> {
        if let Some(ip_address) = &alert.source_ip {
            let cache = self.correlation_cache.read().unwrap();
            let related_alerts = cache.find_alerts_by_ip(ip_address);

            if related_alerts.len() > 1 {
                let correlation_id = format!("ip_{}", ip_address.replace('.', "_"));
                let correlation = AlertCorrelation {
                    id: Uuid::new_v4(),
                    correlation_id,
                    correlation_type: CorrelationType::IpBased,
                    related_alerts,
                    confidence_score: self.calculate_ip_correlation_score(ip_address),
                    root_cause_alert: None,
                    created_at: Utc::now(),
                };

                return Ok(Some(correlation));
            }
        }

        Ok(None)
    }

    /// Find user-based correlations (same user ID)
    async fn find_user_based_correlation(&self, alert: &SecurityAlert) -> AlertResult<Option<AlertCorrelation>> {
        if let Some(user_id) = &alert.user_id {
            let cache = self.correlation_cache.read().unwrap();
            let related_alerts = cache.find_alerts_by_user(user_id);

            if related_alerts.len() > 1 {
                let correlation_id = format!("user_{}", user_id);
                let correlation = AlertCorrelation {
                    id: Uuid::new_v4(),
                    correlation_id,
                    correlation_type: CorrelationType::UserBased,
                    related_alerts,
                    confidence_score: self.calculate_user_correlation_score(user_id),
                    root_cause_alert: None,
                    created_at: Utc::now(),
                };

                return Ok(Some(correlation));
            }
        }

        Ok(None)
    }

    /// Find session-based correlations (same session ID)
    async fn find_session_based_correlation(&self, alert: &SecurityAlert) -> AlertResult<Option<AlertCorrelation>> {
        if let Some(session_id) = &alert.session_id {
            let cache = self.correlation_cache.read().unwrap();
            let related_alerts = cache.find_alerts_by_session(session_id);

            if related_alerts.len() > 1 {
                let correlation_id = format!("session_{}", session_id);
                let correlation = AlertCorrelation {
                    id: Uuid::new_v4(),
                    correlation_id,
                    correlation_type: CorrelationType::SessionBased,
                    related_alerts,
                    confidence_score: 0.9, // High confidence for session-based correlation
                    root_cause_alert: None,
                    created_at: Utc::now(),
                };

                return Ok(Some(correlation));
            }
        }

        Ok(None)
    }

    /// Find pattern-based correlations (similar alert patterns)
    async fn find_pattern_based_correlation(&self, alert: &SecurityAlert) -> AlertResult<Option<AlertCorrelation>> {
        let cache = self.correlation_cache.read().unwrap();
        let similar_alerts = cache.find_similar_patterns(alert);

        if similar_alerts.len() > 1 {
            let correlation_id = format!("pattern_{}_{}", alert.category, alert.severity);
            let correlation = AlertCorrelation {
                id: Uuid::new_v4(),
                correlation_id,
                correlation_type: CorrelationType::PatternBased,
                related_alerts: similar_alerts,
                confidence_score: self.calculate_pattern_correlation_score(alert),
                root_cause_alert: None,
                created_at: Utc::now(),
            };

            return Ok(Some(correlation));
        }

        Ok(None)
    }

    /// Calculate correlation score for time-based correlation
    fn calculate_time_correlation_score(&self, related_alerts: &[Uuid]) -> f64 {
        let base_score = 0.3;
        let count_multiplier = (related_alerts.len() as f64 * 0.1).min(0.5);
        (base_score + count_multiplier).min(1.0)
    }

    /// Calculate correlation score for IP-based correlation
    fn calculate_ip_correlation_score(&self, _ip_address: &str) -> f64 {
        // In a real implementation, this could consider IP reputation
        0.7
    }

    /// Calculate correlation score for user-based correlation
    fn calculate_user_correlation_score(&self, _user_id: &str) -> f64 {
        // Could consider user behavior patterns
        0.8
    }

    /// Calculate correlation score for pattern-based correlation
    fn calculate_pattern_correlation_score(&self, alert: &SecurityAlert) -> f64 {
        match alert.severity.as_str() {
            "CRITICAL" => 0.9,
            "HIGH" => 0.8,
            "MEDIUM" => 0.6,
            "LOW" => 0.4,
            _ => 0.5,
        }
    }

    /// Update correlation cache with new alert
    async fn update_correlation_cache(
        &self,
        alert: &SecurityAlert,
        correlations: &[AlertCorrelation],
    ) -> AlertResult<()> {
        let mut cache = self.correlation_cache.write().unwrap();
        cache.add_alert(alert);

        for correlation in correlations {
            cache.add_correlation(correlation);
        }

        Ok(())
    }

    /// Get active correlations
    pub async fn get_active_correlations(&self) -> HashMap<String, AlertCorrelation> {
        self.active_correlations.read().unwrap().clone()
    }

    /// Clean up expired correlations
    pub async fn cleanup_expired_correlations(&self) -> AlertResult<()> {
        let cutoff_time = Utc::now() - Duration::hours(24);
        
        let mut correlations = self.active_correlations.write().unwrap();
        correlations.retain(|_, correlation| correlation.created_at > cutoff_time);

        let mut cache = self.correlation_cache.write().unwrap();
        cache.cleanup_expired(cutoff_time);

        info!("Cleaned up expired correlations, {} active", correlations.len());
        Ok(())
    }
}

/// Cache for correlation data to improve performance
pub struct CorrelationCache {
    alerts_by_ip: HashMap<String, Vec<Uuid>>,
    alerts_by_user: HashMap<String, Vec<Uuid>>,
    alerts_by_session: HashMap<String, Vec<Uuid>>,
    alerts_by_time: Vec<(DateTime<Utc>, Uuid)>,
    alert_metadata: HashMap<Uuid, AlertCacheEntry>,
}

#[derive(Debug, Clone)]
pub struct AlertCacheEntry {
    pub id: Uuid,
    pub severity: String,
    pub category: String,
    pub triggered_at: DateTime<Utc>,
    pub source_ip: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

impl CorrelationCache {
    pub fn new() -> Self {
        Self {
            alerts_by_ip: HashMap::new(),
            alerts_by_user: HashMap::new(),
            alerts_by_session: HashMap::new(),
            alerts_by_time: Vec::new(),
            alert_metadata: HashMap::new(),
        }
    }

    pub fn add_alert(&mut self, alert: &SecurityAlert) {
        let entry = AlertCacheEntry {
            id: alert.id,
            severity: alert.severity.clone(),
            category: alert.category.clone(),
            triggered_at: alert.triggered_at,
            source_ip: alert.source_ip.clone(),
            user_id: alert.user_id.clone(),
            session_id: alert.session_id.clone(),
        };

        // Index by IP
        if let Some(ip) = &alert.source_ip {
            self.alerts_by_ip.entry(ip.clone()).or_default().push(alert.id);
        }

        // Index by user
        if let Some(user_id) = &alert.user_id {
            self.alerts_by_user.entry(user_id.clone()).or_default().push(alert.id);
        }

        // Index by session
        if let Some(session_id) = &alert.session_id {
            self.alerts_by_session.entry(session_id.clone()).or_default().push(alert.id);
        }

        // Index by time
        self.alerts_by_time.push((alert.triggered_at, alert.id));
        self.alerts_by_time.sort_by_key(|&(time, _)| time);

        // Store metadata
        self.alert_metadata.insert(alert.id, entry);
    }

    pub fn add_correlation(&mut self, _correlation: &AlertCorrelation) {
        // Store correlation for future reference
    }

    pub fn find_alerts_in_time_window(
        &self,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    ) -> Vec<Uuid> {
        self.alerts_by_time
            .iter()
            .filter(|&&(time, _)| time >= window_start && time <= window_end)
            .map(|&(_, id)| id)
            .collect()
    }

    pub fn find_alerts_by_ip(&self, ip_address: &str) -> Vec<Uuid> {
        self.alerts_by_ip
            .get(ip_address)
            .cloned()
            .unwrap_or_default()
    }

    pub fn find_alerts_by_user(&self, user_id: &str) -> Vec<Uuid> {
        self.alerts_by_user
            .get(user_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn find_alerts_by_session(&self, session_id: &str) -> Vec<Uuid> {
        self.alerts_by_session
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn find_similar_patterns(&self, alert: &SecurityAlert) -> Vec<Uuid> {
        let mut similar_alerts = Vec::new();

        for (alert_id, entry) in &self.alert_metadata {
            if *alert_id == alert.id {
                continue;
            }

            let mut similarity_score = 0.0;

            // Category match
            if entry.category == alert.category {
                similarity_score += 0.4;
            }

            // Severity match
            if entry.severity == alert.severity {
                similarity_score += 0.3;
            }

            // Time proximity (within 1 hour)
            let time_diff = (entry.triggered_at - alert.triggered_at).abs();
            if time_diff <= Duration::hours(1) {
                similarity_score += 0.3;
            }

            // Consider similar if score > 0.6
            if similarity_score > 0.6 {
                similar_alerts.push(*alert_id);
            }
        }

        similar_alerts
    }

    pub fn cleanup_expired(&mut self, cutoff_time: DateTime<Utc>) {
        // Clean up time-indexed alerts
        self.alerts_by_time.retain(|&(time, _)| time > cutoff_time);

        // Get expired alert IDs
        let expired_ids: HashSet<Uuid> = self.alert_metadata
            .iter()
            .filter(|(_, entry)| entry.triggered_at <= cutoff_time)
            .map(|(&id, _)| id)
            .collect();

        // Clean up metadata
        for id in &expired_ids {
            self.alert_metadata.remove(id);
        }

        // Clean up other indexes
        for alerts in self.alerts_by_ip.values_mut() {
            alerts.retain(|id| !expired_ids.contains(id));
        }
        
        for alerts in self.alerts_by_user.values_mut() {
            alerts.retain(|id| !expired_ids.contains(id));
        }
        
        for alerts in self.alerts_by_session.values_mut() {
            alerts.retain(|id| !expired_ids.contains(id));
        }

        // Remove empty entries
        self.alerts_by_ip.retain(|_, alerts| !alerts.is_empty());
        self.alerts_by_user.retain(|_, alerts| !alerts.is_empty());
        self.alerts_by_session.retain(|_, alerts| !alerts.is_empty());
    }
}

/// Correlation analytics for dashboard and reporting
pub struct CorrelationAnalytics;

impl CorrelationAnalytics {
    /// Analyze correlation patterns for insights
    pub fn analyze_patterns(correlations: &[AlertCorrelation]) -> CorrelationInsights {
        let mut insights = CorrelationInsights::default();

        for correlation in correlations {
            insights.total_correlations += 1;
            
            match correlation.correlation_type {
                CorrelationType::TimeBased => insights.time_based_count += 1,
                CorrelationType::IpBased => insights.ip_based_count += 1,
                CorrelationType::UserBased => insights.user_based_count += 1,
                CorrelationType::SessionBased => insights.session_based_count += 1,
                CorrelationType::PatternBased => insights.pattern_based_count += 1,
                CorrelationType::Geolocation => insights.geo_based_count += 1,
                CorrelationType::ThreatIntelligence => insights.threat_intel_count += 1,
            }

            if correlation.confidence_score > insights.highest_confidence {
                insights.highest_confidence = correlation.confidence_score;
            }

            insights.average_confidence += correlation.confidence_score;
        }

        if insights.total_correlations > 0 {
            insights.average_confidence /= insights.total_correlations as f64;
        }

        insights
    }
}

#[derive(Debug, Default)]
pub struct CorrelationInsights {
    pub total_correlations: usize,
    pub time_based_count: usize,
    pub ip_based_count: usize,
    pub user_based_count: usize,
    pub session_based_count: usize,
    pub pattern_based_count: usize,
    pub geo_based_count: usize,
    pub threat_intel_count: usize,
    pub highest_confidence: f64,
    pub average_confidence: f64,
}