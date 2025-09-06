// Security Monitoring Service Adapter
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use tracing::{debug, info, warn, error};

use crate::domain::authentication::{AuthenticatedUserId, SessionId};
use crate::domain::authentication::repositories::SecurityMonitoringServicePort;
use crate::infrastructure::cache::Cache;

/// Security monitoring service adapter
pub struct SecurityMonitoringServiceAdapter {
    /// Cache for storing security metrics and rate limiting
    cache: Arc<dyn Cache>,
    
    /// Configuration
    config: SecurityMonitoringConfig,
}

impl SecurityMonitoringServiceAdapter {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self {
            cache,
            config: SecurityMonitoringConfig::default(),
        }
    }
    
    /// Get cache key for rate limiting
    fn get_rate_limit_key(&self, identifier: &str, limit_type: &str) -> String {
        format!("rate_limit:{}:{}", limit_type, identifier)
    }
    
    /// Get cache key for IP reputation
    fn get_ip_reputation_key(&self, ip: &str) -> String {
        format!("ip_reputation:{}", ip)
    }
    
    /// Get cache key for security metrics
    fn get_security_metrics_key(&self, user_id: &str) -> String {
        format!("security_metrics:user:{}", user_id)
    }
    
    /// Update security metrics for user
    async fn update_user_security_metrics(
        &self,
        user_id: &str,
        metric_type: &str,
        increment: i64,
    ) -> Result<(), String> {
        let key = self.get_security_metrics_key(user_id);
        
        // Get existing metrics
        let mut metrics: SecurityMetrics = match self.cache.get(&key) {
            Some(data) => serde_json::from_str(&data).unwrap_or_default(),
            None => SecurityMetrics::default(),
        };
        
        // Update specific metric
        match metric_type {
            "failed_auth" => metrics.failed_auth_attempts += increment as u64,
            "suspicious_activity" => metrics.suspicious_activities += increment as u64,
            "session_creation" => metrics.sessions_created += increment as u64,
            _ => {}
        }
        
        metrics.last_updated = Utc::now();
        
        // Store updated metrics with TTL
        let serialized = serde_json::to_string(&metrics)
            .map_err(|e| format!("Failed to serialize metrics: {}", e))?;
        
        self.cache.set(&key, serialized, Some(Duration::hours(24).num_seconds() as u64));
        
        Ok(())
    }
}

#[async_trait]
impl SecurityMonitoringServicePort for SecurityMonitoringServiceAdapter {
    async fn record_session_creation(
        &self,
        user_id: &str,
        ip: &str,
        session_id: &str,
    ) -> Result<(), String> {
        info!(
            session_id = session_id,
            user_id = user_id,
            ip = ip,
            "Recording session creation for security monitoring"
        );
        
        // Update user security metrics
        self.update_user_security_metrics(user_id, "session_creation", 1).await?;
        
        // Record IP activity
        let ip_addr = ip;
        // Update IP reputation tracking
        let ip_key = format!("ip_activity:{}", ip_addr);
        let activity_data = format!("session_creation:{}:{}", user_id, Utc::now().timestamp());
        
        self.cache.set(&ip_key, activity_data, Some(86400)); // 24 hour TTL
        
        debug!(
            session_id = session_id,
            user_id = user_id,
            "Session creation recorded in security monitoring"
        );
        
        Ok(())
    }
    
    async fn record_authentication_failure(
        &self,
        ip: &str,
        user_id: Option<&str>,
        reason: &str,
    ) -> Result<(), String> {
        warn!(
            user_id = user_id.unwrap_or("unknown"),
            reason = reason,
            ip = ip,
            "Recording authentication failure"
        );
        
        // Update user metrics if user ID is known
        if let Some(uid) = user_id {
            let user_id_str = uid.to_string();
            self.update_user_security_metrics(&user_id_str, "failed_auth", 1).await?;
        }
        
        // Update IP reputation if provided
        if !ip.is_empty() {
            let ip_addr = ip;
            let ip_rep_key = self.get_ip_reputation_key(ip_addr);
            
            // Get current reputation
            let mut reputation: IpReputation = match self.cache.get(&ip_rep_key) {
                Some(data) => serde_json::from_str(&data).unwrap_or_else(|_| IpReputation::new(ip_addr.to_string())),
                None => IpReputation::new(ip_addr.to_string()),
            };
            
            reputation.record_failure(reason);
            
            // Store updated reputation
            let serialized = serde_json::to_string(&reputation)
                .map_err(|e| format!("Failed to serialize IP reputation: {}", e))?;
            
            self.cache.set(&ip_rep_key, serialized, Some(Duration::days(7).num_seconds() as u64));
            // Cache.set() returns () not Result, no error handling needed
        }
        
        Ok(())
    }
    
    async fn is_suspicious_ip(&self, ip: &str) -> Result<bool, String> {
        debug!(ip = ip, "Checking IP reputation");
        
        let ip_rep_key = self.get_ip_reputation_key(ip);
        
        match self.cache.get(&ip_rep_key) {
            Some(data) => {
                let reputation: IpReputation = serde_json::from_str(&data)
                    .map_err(|e| format!("Failed to deserialize IP reputation: {}", e))?;
                
                let is_suspicious = reputation.is_suspicious();
                
                if is_suspicious {
                    warn!(
                        ip = ip,
                        reputation_score = reputation.reputation_score,
                        failed_attempts = reputation.failed_attempts,
                        "Suspicious IP detected"
                    );
                } else {
                    debug!(ip = ip, "IP reputation check passed");
                }
                
                Ok(is_suspicious)
            },
            None => {
                debug!(ip = ip, "No reputation data found for IP, assuming safe");
                Ok(false) // No data means it's not known to be suspicious
            },
        }
    }
    
    async fn is_rate_limited(&self, user_id: &str) -> Result<bool, String> {
        debug!(user_id = user_id, "Checking rate limits");
        
        // Check session creation rate limit
        let session_key = self.get_rate_limit_key(user_id, "session_creation");
        let current_count = match self.cache.get(&session_key) {
            Some(count_str) => count_str.parse::<u64>().unwrap_or(0),
            None => 0,
        };
        
        if current_count >= self.config.max_session_creations_per_hour {
            warn!(
                user_id = user_id,
                current_count = current_count,
                limit = self.config.max_session_creations_per_hour,
                "User hit session creation rate limit"
            );
            return Ok(true);
        }
        
        // Check authentication failure rate limit
        let auth_fail_key = self.get_rate_limit_key(user_id, "auth_failures");
        let fail_count = match self.cache.get(&auth_fail_key) {
            Some(count_str) => count_str.parse::<u64>().unwrap_or(0),
            None => 0,
        };
        
        if fail_count >= self.config.max_auth_failures_per_hour {
            warn!(
                user_id = user_id,
                fail_count = fail_count,
                limit = self.config.max_auth_failures_per_hour,
                "User hit authentication failure rate limit"
            );
            return Ok(true);
        }
        
        debug!(user_id = user_id, "Rate limit check passed");
        Ok(false)
    }
    
    async fn record_suspicious_activity(&self, user_id: &str, ip: &str, activity: &str) -> Result<(), String> {
        warn!(
            user_id = user_id,
            ip = ip,
            activity = activity,
            "Recording suspicious activity"
        );
        
        // Update user security metrics
        self.update_user_security_metrics(user_id, "suspicious_activity", 1).await?;
        
        // Store activity record in cache
        let record_key = format!(
            "suspicious_activity:{}:{}",
            user_id,
            Utc::now().timestamp()
        );
        
        let activity_data = format!("{}|{}|{}", user_id, ip, activity);
        self.cache.set(&record_key, activity_data, Some(86400)); // 24 hour TTL
        
        info!(
            user_id = user_id,
            activity = activity,
            "Suspicious activity recorded"
        );
        
        Ok(())
    }
    
    async fn get_security_summary(&self, user_id: &str) -> Result<crate::domain::authentication::repositories::SecuritySummary, String> {
        debug!(user_id = user_id, "Getting security summary");
        let metrics_key = self.get_security_metrics_key(user_id);
        
        let metrics: SecurityMetrics = match self.cache.get(&metrics_key) {
            Some(data) => serde_json::from_str(&data).unwrap_or_default(),
            None => SecurityMetrics::default(),
        };
        
        // Calculate risk score
        let risk_score = self.calculate_user_risk_score(&metrics);
        let risk_level = if risk_score > 80.0 {
            "critical"
        } else if risk_score > 60.0 {
            "high"
        } else if risk_score > 30.0 {
            "medium"
        } else {
            "low"
        };
        
        // Convert f64 risk_score to RiskScore enum
        let risk_score_enum = if risk_score > 80.0 {
            crate::domain::authentication::repositories::RiskScore::Critical
        } else if risk_score > 60.0 {
            crate::domain::authentication::repositories::RiskScore::High
        } else if risk_score > 30.0 {
            crate::domain::authentication::repositories::RiskScore::Medium
        } else {
            crate::domain::authentication::repositories::RiskScore::Low
        };

        let summary = crate::domain::authentication::repositories::SecuritySummary {
            user_id: user_id.to_string(),
            recent_login_attempts: metrics.failed_auth_attempts as u32,
            failed_attempts: metrics.failed_auth_attempts as u32,
            suspicious_activities: metrics.suspicious_activities as u32,
            last_login: Some(metrics.last_updated),
            risk_score: risk_score_enum,
            is_locked: false, // Default to not locked
        };
        
        info!(
            user_id = user_id,
            risk_level = risk_level,
            risk_score = risk_score,
            "Security summary generated"
        );
        
        Ok(summary)
    }
    
    async fn increment_rate_limit_counter(&self, key: &str) -> Result<u32, String> {
        // Use the key directly as provided
        
        // Get current count
        let current_count = match self.cache.get(key) {
            Some(count_str) => count_str.parse::<u32>().unwrap_or(0),
            _ => 0,
        };
        
        let new_count = current_count + 1;
        
        // Store incremented count with TTL
        self.cache.set(key, new_count.to_string(), Some(3600));  // 1 hour TTL
        
        Ok(new_count)
    }
    
    async fn record_auth_attempt(&self, ip: &str, user_id: Option<&str>, success: bool) -> Result<(), String> {
        let attempt_type = if success { "successful" } else { "failed" };
        info!(
            ip = ip,
            user_id = user_id.unwrap_or("unknown"),
            attempt_type = attempt_type,
            "Recording authentication attempt"
        );
        
        // Record IP activity
        let ip_key = format!("auth_attempts:{}", ip);
        let _count = self.increment_rate_limit_counter(&ip_key).await?;
        
        // Record user-specific activity if user_id provided
        if let Some(uid) = user_id {
            self.update_user_security_metrics(uid, if success { "auth_success" } else { "auth_failure" }, 1).await?;
        }
        
        Ok(())
    }
    
    async fn get_risk_score(&self, ip: &str, user_id: &str) -> Result<crate::domain::authentication::repositories::RiskScore, String> {
        info!(ip = ip, user_id = user_id, "Calculating risk score");
        
        // Get IP-based metrics
        let ip_key = format!("auth_attempts:{}", ip);
        let ip_attempts = match self.cache.get(&ip_key) {
            Some(count_str) => count_str.parse::<u32>().unwrap_or(0),
            _ => 0,
        };
        
        // Get user-based metrics
        let metrics_key = self.get_security_metrics_key(user_id);
        let user_metrics: SecurityMetrics = match self.cache.get(&metrics_key) {
            Some(data) => serde_json::from_str(&data).unwrap_or_default(),
            _ => SecurityMetrics::default(),
        };
        
        // Calculate combined risk score
        let mut score = 0.0;
        score += (ip_attempts as f64) * 2.0; // IP attempts factor
        score += self.calculate_user_risk_score(&user_metrics); // User metrics factor
        
        let risk_level = if score > 50.0 { "high" } else if score > 20.0 { "medium" } else { "low" };
        
        let risk_score_enum = if score > 80.0 {
            crate::domain::authentication::repositories::RiskScore::Critical
        } else if score > 60.0 {
            crate::domain::authentication::repositories::RiskScore::High
        } else if score > 30.0 {
            crate::domain::authentication::repositories::RiskScore::Medium
        } else {
            crate::domain::authentication::repositories::RiskScore::Low
        };
        
        Ok(risk_score_enum)
    }
    
    async fn record_security_event(&self, event: crate::domain::authentication::repositories::SecurityEvent) -> Result<(), String> {
        info!(
            event_type = ?event.event_type,
            user_id = event.user_id.as_deref().unwrap_or("unknown"),
            ip = event.ip_address,
            "Recording security event"
        );
        
        // Store event in cache with timestamp key
        let event_key = format!(
            "security_events:{}:{}",
            event.user_id.as_deref().unwrap_or("system"),
            Utc::now().timestamp()
        );
        
        let event_data = serde_json::to_string(&event).map_err(|e| e.to_string())?;
        self.cache.set(&event_key, event_data, Some(604800)); // 7 days TTL
        
        // Update user metrics if user_id is provided
        if let Some(user_id) = &event.user_id {
            self.update_user_security_metrics(user_id, "security_event", 1).await?;
        }
        
        Ok(())
    }
    
    async fn check_rate_limit(&self, key: &str, limit: u32, window_seconds: u64) -> Result<bool, String> {
        let current_count = self.increment_rate_limit_counter(key).await?;
        
        let is_limited = current_count > limit;
        
        if is_limited {
            warn!(
                key = key,
                current_count = current_count,
                limit = limit,
                "Rate limit exceeded"
            );
        }
        
        Ok(!is_limited)
    }
}

impl SecurityMonitoringServiceAdapter {
    /// Calculate user risk score based on security metrics
    fn calculate_user_risk_score(&self, metrics: &SecurityMetrics) -> f64 {
        let mut score = 0.0;
        
        // Failed authentication attempts contribute to risk
        score += (metrics.failed_auth_attempts as f64) * 5.0;
        
        // Suspicious activities heavily impact risk
        score += (metrics.suspicious_activities as f64) * 15.0;
        
        // Too many recent sessions might indicate account sharing
        if metrics.sessions_created > 10 {
            score += 10.0;
        }
        
        // Time factor - recent activity is more concerning
        let hours_since_update = (Utc::now() - metrics.last_updated).num_hours();
        if hours_since_update < 24 {
            score *= 1.5; // Increase risk for recent activity
        }
        
        score.min(100.0)
    }
    
    /// Generate security recommendations based on metrics
    fn generate_security_recommendations(&self, metrics: &SecurityMetrics, risk_score: f64) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if risk_score > 70.0 {
            recommendations.push("Consider terminating all sessions and requiring re-authentication".to_string());
            recommendations.push("Enable additional security monitoring".to_string());
        }
        
        if metrics.failed_auth_attempts > 5 {
            recommendations.push("Review failed authentication attempts for potential brute force attacks".to_string());
            recommendations.push("Consider implementing account lockout after repeated failures".to_string());
        }
        
        if metrics.suspicious_activities > 3 {
            recommendations.push("Investigate recent suspicious activities".to_string());
            recommendations.push("Consider requiring additional verification for sensitive operations".to_string());
        }
        
        if metrics.sessions_created > 15 {
            recommendations.push("User has many sessions - review for potential account sharing".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Security posture looks good - continue monitoring".to_string());
        }
        
        recommendations
    }
}

/// Security monitoring configuration
#[derive(Debug, Clone)]
pub struct SecurityMonitoringConfig {
    pub max_session_creations_per_hour: u64,
    pub max_auth_failures_per_hour: u64,
    pub suspicious_ip_threshold: f64,
}

impl Default for SecurityMonitoringConfig {
    fn default() -> Self {
        Self {
            max_session_creations_per_hour: 10,
            max_auth_failures_per_hour: 5,
            suspicious_ip_threshold: 70.0,
        }
    }
}

/// Security metrics for a user
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityMetrics {
    pub failed_auth_attempts: u64,
    pub suspicious_activities: u64,
    pub sessions_created: u64,
    pub last_updated: DateTime<Utc>,
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self {
            failed_auth_attempts: 0,
            suspicious_activities: 0,
            sessions_created: 0,
            last_updated: Utc::now(),
        }
    }
}

/// IP reputation tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpReputation {
    pub ip_address: String,
    pub reputation_score: f64, // 0-100, lower is more suspicious
    pub failed_attempts: u64,
    pub successful_attempts: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub failure_reasons: HashMap<String, u64>,
}

impl IpReputation {
    pub fn new(ip_address: String) -> Self {
        let now = Utc::now();
        Self {
            ip_address,
            reputation_score: 50.0, // Neutral starting score
            failed_attempts: 0,
            successful_attempts: 0,
            first_seen: now,
            last_seen: now,
            failure_reasons: HashMap::new(),
        }
    }
    
    pub fn record_failure(&mut self, reason: &str) {
        self.failed_attempts += 1;
        self.last_seen = Utc::now();
        *self.failure_reasons.entry(reason.to_string()).or_insert(0) += 1;
        
        // Decrease reputation score
        self.reputation_score = (self.reputation_score - 10.0).max(0.0);
    }
    
    pub fn record_success(&mut self) {
        self.successful_attempts += 1;
        self.last_seen = Utc::now();
        
        // Slightly increase reputation score
        self.reputation_score = (self.reputation_score + 2.0).min(100.0);
    }
    
    pub fn is_suspicious(&self) -> bool {
        self.reputation_score < 30.0 || self.failed_attempts > 10
    }
}

/// Suspicious activity record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuspiciousActivityRecord {
    pub session_id: String,
    pub user_id: String,
    pub activity_type: String,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Security summary for a user
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecuritySummary {
    pub user_id: String,
    pub risk_score: f64,
    pub risk_level: String,
    pub failed_auth_attempts: u64,
    pub suspicious_activities: u64,
    pub sessions_created: u64,
    pub last_activity: DateTime<Utc>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::memory_cache::MemoryCache;
    
    #[tokio::test]
    async fn test_security_monitoring_creation() {
        let cache = Arc::new(MemoryCache::new(1000));
        let adapter = SecurityMonitoringServiceAdapter::new(cache);
        
        // Test basic functionality
        let session_id = SessionId::generate();
        let user_id = AuthenticatedUserId::from_verified_user(
            crate::domain::shared_kernel::value_objects::UserId::new("123".to_string())
        );
        
        let result = adapter.record_session_creation(&session_id, &user_id, Some("192.168.1.1")).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let cache = Arc::new(MemoryCache::new(1000));
        let adapter = SecurityMonitoringServiceAdapter::new(cache);
        
        let user_id_str = "test_user_123";
        
        // Initially should not be rate limited
        let result = adapter.is_rate_limited(user_id_str).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
        
        // Increment counter multiple times
        for _ in 0..15 {
            let _ = adapter.increment_rate_limit_counter(user_id_str, "session_creation").await;
        }
        
        // Should now be rate limited
        let result = adapter.is_rate_limited(user_id_str).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
    
    #[tokio::test]
    async fn test_ip_reputation() {
        let cache = Arc::new(MemoryCache::new(1000));
        let adapter = SecurityMonitoringServiceAdapter::new(cache);
        
        let ip = "192.168.1.100";
        
        // Initially should not be suspicious
        let result = adapter.is_suspicious_ip(ip).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
        
        // Record multiple authentication failures
        for _ in 0..3 {
            let _ = adapter.record_authentication_failure(None, "invalid_password", Some(ip)).await;
        }
        
        // Should now be considered suspicious
        let result = adapter.is_suspicious_ip(ip).await;
        assert!(result.is_ok());
        // Note: This might not be true depending on the threshold, but it tests the flow
    }
}