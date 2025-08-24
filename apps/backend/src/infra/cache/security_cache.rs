// Security-focused Redis cache implementation for middleware operations
// Provides specialized caching for session validation, permission checking, and security events

use super::{Cache, CacheExt, CacheError};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tracing::{info, warn, debug};
use crate::web::security::models::CreateSecurityEventRequest;

/// Cached session data for middleware validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSession {
    pub authenticated: bool,
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub subscription_tier: String,
    pub session_type: String,
    pub expires_at: DateTime<Utc>,
}

/// Security-specific cache keys for middleware operations and threat detection
pub struct SecurityCacheKeys;

impl SecurityCacheKeys {
    // Session validation cache keys
    pub fn admin_session(session_id: &str) -> String {
        format!("security:admin_session:{}", session_id)
    }
    
    pub fn user_session(session_id: &str) -> String {
        format!("security:user_session:{}", session_id)
    }
    
    // Permission cache keys
    pub fn admin_permissions(user_id: &str) -> String {
        format!("security:admin_permissions:{}", user_id)
    }
    
    pub fn user_permissions(user_id: &str) -> String {
        format!("security:user_permissions:{}", user_id)
    }
    
    pub fn admin_modules(user_id: &str) -> String {
        format!("security:admin_modules:{}", user_id)
    }
    
    pub fn package_tier(user_id: &str) -> String {
        format!("security:package_tier:{}", user_id)
    }
    
    // Security event cache keys
    pub fn security_events() -> String {
        "security:events".to_string()
    }
    
    pub fn brute_force_attempts(ip: &str) -> String {
        format!("security:brute_force:{}", ip)
    }
    
    pub fn suspicious_activity() -> String {
        "security:suspicious_activity".to_string()
    }
    
    pub fn performance_metrics(date: &str) -> String {
        format!("security:performance:{}", date)
    }
    
    // Daily statistics
    pub fn daily_stats(date: &str) -> String {
        format!("security:stats:{}", date)
    }
    
    // Real-time threat detection keys
    pub fn ip_reputation(ip: &str) -> String {
        format!("security:ip_reputation:{}", ip)
    }
    
    pub fn blocked_ips() -> String {
        "security:blocked_ips".to_string()
    }
    
    pub fn suspicious_patterns() -> String {
        "security:suspicious_patterns".to_string()
    }
    
    pub fn rate_limit_tracker(ip: &str, endpoint: &str) -> String {
        format!("security:rate_limit:{}:{}", ip, endpoint)
    }
    
    pub fn threat_detection_state() -> String {
        "security:threat_state".to_string()
    }
    
    pub fn anomaly_scores(user_id: &str) -> String {
        format!("security:anomaly:{}", user_id)
    }
    
    pub fn geographic_anomalies(user_id: &str) -> String {
        format!("security:geo_anomaly:{}", user_id)
    }
}

/// Cached session data for admin users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAdminSession {
    pub user_id: String,
    pub email: String,
    pub admin_modules: Vec<String>,
    pub effective_permissions: Vec<String>,
    pub session_id: String,
    pub is_active: bool,
    pub last_activity: DateTime<Utc>,
    pub cached_at: DateTime<Utc>,
}

/// Cached session data for regular users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUserSession {
    pub user_id: String,
    pub email: String,
    pub package_tier: String,
    pub effective_permissions: Vec<String>,
    pub subscription_status: String,
    pub session_id: String,
    pub is_active: bool,
    pub last_activity: DateTime<Utc>,
    pub cached_at: DateTime<Utc>,
}

/// Enhanced security event for logging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub path: String,
    pub method: Option<String>,
    pub ip: String,
    pub user_agent: String,
    pub severity: String,
    pub details: serde_json::Value,
    pub risk_score: Option<f64>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub device_fingerprint: Option<String>,
    pub correlation_id: Option<String>,
}

/// IP reputation and threat intelligence data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpThreatData {
    pub ip_address: String,
    pub is_malicious: bool,
    pub is_vpn: bool,
    pub is_proxy: bool,
    pub reputation_score: f64,
    pub country_code: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub threat_types: Vec<String>,
    pub blocked: bool,
}

/// Rate limiting and pattern detection data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatPattern {
    pub pattern_id: String,
    pub pattern_type: String,
    pub source_ips: Vec<String>,
    pub detection_count: i64,
    pub first_detected: DateTime<Utc>,
    pub last_detected: DateTime<Utc>,
    pub severity: String,
    pub auto_blocked: bool,
}

/// Performance metrics for middleware operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub middleware_execution_time: f64,
    pub cache_hit_rate: f64,
    pub session_validation_time: f64,
    pub permission_check_time: f64,
    pub total_request_time: f64,
    pub timestamp: DateTime<Utc>,
}

/// Security-focused cache wrapper providing specialized middleware operations and threat detection
pub struct SecurityCache {
    cache: Arc<dyn Cache>,
    security_event_ttl: i64,
    session_validation_ttl: i64,
    permission_ttl: i64,
    admin_module_ttl: i64,
    ip_reputation_ttl: i64,
    threat_pattern_ttl: i64,
    rate_limit_window: i64,
}

impl SecurityCache {
    pub fn new(
        cache: Arc<dyn Cache>,
        security_event_ttl: i64,
        session_validation_ttl: i64,
        permission_ttl: i64,
        admin_module_ttl: i64,
    ) -> Self {
        Self {
            cache,
            security_event_ttl,
            session_validation_ttl,
            permission_ttl,
            admin_module_ttl,
            ip_reputation_ttl: 86400, // 24 hours
            threat_pattern_ttl: 3600,  // 1 hour
            rate_limit_window: 300,    // 5 minutes
        }
    }
    
    pub fn with_threat_config(
        cache: Arc<dyn Cache>,
        security_event_ttl: i64,
        session_validation_ttl: i64,
        permission_ttl: i64,
        admin_module_ttl: i64,
        ip_reputation_ttl: i64,
        threat_pattern_ttl: i64,
        rate_limit_window: i64,
    ) -> Self {
        Self {
            cache,
            security_event_ttl,
            session_validation_ttl,
            permission_ttl,
            admin_module_ttl,
            ip_reputation_ttl,
            threat_pattern_ttl,
            rate_limit_window,
        }
    }

    // Session validation operations
    pub async fn cache_admin_session(
        &self,
        session_id: &str,
        session_data: &CachedAdminSession,
    ) -> Result<(), CacheError> {
        let key = SecurityCacheKeys::admin_session(session_id);
        self.cache.set(&key, session_data, Some(self.session_validation_ttl)).await?;
        
        debug!("Cached admin session for user: {}", session_data.user_id);
        Ok(())
    }
    
    pub async fn get_admin_session(
        &self,
        session_id: &str,
    ) -> Result<Option<CachedAdminSession>, CacheError> {
        let key = SecurityCacheKeys::admin_session(session_id);
        let session = self.cache.get::<CachedAdminSession>(&key).await?;
        
        if session.is_some() {
            debug!("Cache hit for admin session: {}", session_id);
        } else {
            debug!("Cache miss for admin session: {}", session_id);
        }
        
        Ok(session)
    }
    
    pub async fn cache_user_session(
        &self,
        session_id: &str,
        session_data: &CachedUserSession,
    ) -> Result<(), CacheError> {
        let key = SecurityCacheKeys::user_session(session_id);
        self.cache.set(&key, session_data, Some(self.session_validation_ttl)).await?;
        
        debug!("Cached user session for user: {}", session_data.user_id);
        Ok(())
    }
    
    pub async fn get_user_session(
        &self,
        session_id: &str,
    ) -> Result<Option<CachedUserSession>, CacheError> {
        let key = SecurityCacheKeys::user_session(session_id);
        let session = self.cache.get::<CachedUserSession>(&key).await?;
        
        if session.is_some() {
            debug!("Cache hit for user session: {}", session_id);
        } else {
            debug!("Cache miss for user session: {}", session_id);
        }
        
        Ok(session)
    }
    
    pub async fn invalidate_session(&self, session_id: &str, is_admin: bool) -> Result<(), CacheError> {
        let key = if is_admin {
            SecurityCacheKeys::admin_session(session_id)
        } else {
            SecurityCacheKeys::user_session(session_id)
        };
        
        self.cache.delete(&key).await?;
        info!("Invalidated {} session: {}", if is_admin { "admin" } else { "user" }, session_id);
        Ok(())
    }
    
    // Generic session caching for middleware validation
    pub async fn cache_session(
        &self,
        session_key: &str,
        session_data: &CachedSession,
        ttl_seconds: i64,
    ) -> Result<(), CacheError> {
        self.cache.set(session_key, session_data, Some(ttl_seconds)).await?;
        debug!("Cached session: {} for user: {}", session_key, session_data.user_id);
        Ok(())
    }
    
    pub async fn get_session(&self, session_key: &str) -> Result<Option<CachedSession>, CacheError> {
        let session = self.cache.get::<CachedSession>(session_key).await?;
        
        if session.is_some() {
            debug!("Cache hit for session: {}", session_key);
        } else {
            debug!("Cache miss for session: {}", session_key);
        }
        
        Ok(session)
    }

    // Permission caching operations
    pub async fn cache_admin_modules(
        &self,
        user_id: &str,
        modules: &[String],
    ) -> Result<(), CacheError> {
        let key = SecurityCacheKeys::admin_modules(user_id);
        self.cache.set(&key, &modules.to_vec(), Some(self.admin_module_ttl)).await?;
        
        debug!("Cached admin modules for user: {} (modules: {:?})", user_id, modules);
        Ok(())
    }
    
    pub async fn get_admin_modules(&self, user_id: &str) -> Result<Option<Vec<String>>, CacheError> {
        let key = SecurityCacheKeys::admin_modules(user_id);
        let modules = self.cache.get::<Vec<String>>(&key).await?;
        
        if modules.is_some() {
            debug!("Cache hit for admin modules: {}", user_id);
        } else {
            debug!("Cache miss for admin modules: {}", user_id);
        }
        
        Ok(modules)
    }
    
    pub async fn cache_user_permissions(
        &self,
        user_id: &str,
        permissions: &[String],
    ) -> Result<(), CacheError> {
        let key = SecurityCacheKeys::user_permissions(user_id);
        self.cache.set(&key, &permissions.to_vec(), Some(self.permission_ttl)).await?;
        
        debug!("Cached user permissions for user: {} (count: {})", user_id, permissions.len());
        Ok(())
    }
    
    pub async fn get_user_permissions(&self, user_id: &str) -> Result<Option<Vec<String>>, CacheError> {
        let key = SecurityCacheKeys::user_permissions(user_id);
        let permissions = self.cache.get::<Vec<String>>(&key).await?;
        
        if permissions.is_some() {
            debug!("Cache hit for user permissions: {}", user_id);
        } else {
            debug!("Cache miss for user permissions: {}", user_id);
        }
        
        Ok(permissions)
    }
    
    pub async fn cache_package_tier(&self, user_id: &str, tier: &str) -> Result<(), CacheError> {
        let key = SecurityCacheKeys::package_tier(user_id);
        self.cache.set(&key, &tier.to_string(), Some(self.permission_ttl)).await?;
        
        debug!("Cached package tier for user: {} (tier: {})", user_id, tier);
        Ok(())
    }
    
    pub async fn get_package_tier(&self, user_id: &str) -> Result<Option<String>, CacheError> {
        let key = SecurityCacheKeys::package_tier(user_id);
        let tier = self.cache.get::<String>(&key).await?;
        
        if tier.is_some() {
            debug!("Cache hit for package tier: {}", user_id);
        } else {
            debug!("Cache miss for package tier: {}", user_id);
        }
        
        Ok(tier)
    }

    // Security event operations
    pub async fn log_security_event(&self, event: &SecurityEvent) -> Result<(), CacheError> {
        let key = SecurityCacheKeys::security_events();
        let score = event.timestamp.timestamp_millis() as f64;
        
        // Store in sorted set for time-based queries
        let event_json = serde_json::to_string(event)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        
        // Use raw Redis commands for sorted set operations
        // This would require direct Redis access, so for now we'll use regular cache
        self.cache.set(&format!("{}:{}", key, score), &event_json, Some(self.security_event_ttl)).await?;
        
        info!("Logged security event: {} for user: {:?}", event.event_type, event.user_id);
        Ok(())
    }
    
    // Enhanced security event logging for middleware operations
    pub async fn log_middleware_security_event(
        &self,
        event: CreateSecurityEventRequest,
    ) -> Result<(), CacheError> {
        // Convert CreateSecurityEventRequest to internal SecurityEvent format
        let internal_event = SecurityEvent {
            event_type: event.event_type.to_string(),
            timestamp: chrono::Utc::now(),
            user_id: event.user_id,
            session_id: event.session_id,
            path: event.path.unwrap_or_else(|| "unknown".to_string()),
            method: event.method,
            ip: event.ip_address,
            user_agent: event.user_agent.unwrap_or_else(|| "unknown".to_string()),
            severity: event.severity.to_string(),
            details: event.details.unwrap_or(serde_json::Value::Null),
            risk_score: None,
            country_code: event.country_code,
            city: event.city,
            device_fingerprint: event.device_fingerprint,
            correlation_id: event.correlation_id,
        };
        
        self.log_security_event(&internal_event).await
    }
    
    pub async fn get_recent_security_events(
        &self,
        _hours: i64,
    ) -> Result<Vec<SecurityEvent>, CacheError> {
        // This is a simplified implementation
        // In a full Redis implementation, we would use ZRANGEBYSCORE
        // For now, return empty vec and log the request
        warn!("get_recent_security_events not fully implemented - needs direct Redis access");
        Ok(vec![])
    }

    // Brute force protection
    pub async fn increment_brute_force_attempts(
        &self,
        ip: &str,
        window_seconds: i64,
    ) -> Result<i64, CacheError> {
        let key = SecurityCacheKeys::brute_force_attempts(ip);
        let count = self.cache.increment(&key, 1, Some(window_seconds)).await?;
        
        if count == 1 {
            info!("Started tracking brute force attempts for IP: {}", ip);
        } else {
            warn!("Brute force attempt #{} from IP: {}", count, ip);
        }
        
        Ok(count)
    }
    
    pub async fn get_brute_force_attempts(&self, ip: &str) -> Result<i64, CacheError> {
        let key = SecurityCacheKeys::brute_force_attempts(ip);
        match self.cache.get::<i64>(&key).await? {
            Some(count) => Ok(count),
            None => Ok(0),
        }
    }
    
    pub async fn clear_brute_force_attempts(&self, ip: &str) -> Result<(), CacheError> {
        let key = SecurityCacheKeys::brute_force_attempts(ip);
        self.cache.delete(&key).await?;
        info!("Cleared brute force attempts for IP: {}", ip);
        Ok(())
    }

    // Performance metrics
    pub async fn record_performance_metrics(
        &self,
        metrics: &PerformanceMetrics,
    ) -> Result<(), CacheError> {
        let date = metrics.timestamp.format("%Y-%m-%d").to_string();
        let key = SecurityCacheKeys::performance_metrics(&date);
        
        // Store metrics with daily aggregation
        self.cache.set(&format!("{}:{}", key, metrics.timestamp.timestamp()), metrics, Some(86400)).await?;
        
        debug!("Recorded performance metrics: middleware_time={}ms, cache_hit_rate={}%", 
               metrics.middleware_execution_time, metrics.cache_hit_rate * 100.0);
        Ok(())
    }

    // Cache statistics and health
    pub async fn get_cache_health(&self) -> Result<serde_json::Value, CacheError> {
        let stats = self.cache.stats().await?;
        
        let health = serde_json::json!({
            "total_entries": stats.total_entries,
            "active_entries": stats.active_entries,
            "memory_usage_bytes": stats.memory_usage_bytes,
            "hit_rate": stats.hit_rate,
            "cache_type": "security_cache",
            "timestamp": Utc::now()
        });
        
        Ok(health)
    }
    
    pub async fn update_last_activity(
        &self,
        session_id: &str,
        is_admin: bool,
    ) -> Result<(), CacheError> {
        let key = if is_admin {
            SecurityCacheKeys::admin_session(session_id)
        } else {
            SecurityCacheKeys::user_session(session_id)
        };
        
        // Get existing session, update last_activity, and save back
        if is_admin {
            if let Some(mut session) = self.cache.get::<CachedAdminSession>(&key).await? {
                session.last_activity = Utc::now();
                self.cache.set(&key, &session, Some(self.session_validation_ttl)).await?;
            }
        } else {
            if let Some(mut session) = self.cache.get::<CachedUserSession>(&key).await? {
                session.last_activity = Utc::now();
                self.cache.set(&key, &session, Some(self.session_validation_ttl)).await?;
            }
        }
        
        Ok(())
    }
    
    // Real-time threat detection and IP blocking
    pub async fn analyze_ip_reputation(&self, ip: &str) -> Result<IpThreatData, CacheError> {
        let key = SecurityCacheKeys::ip_reputation(ip);
        
        if let Some(cached_data) = self.cache.get::<IpThreatData>(&key).await? {
            debug!("IP reputation cache hit for: {}", ip);
            return Ok(cached_data);
        }
        
        // Create new IP threat analysis (in real implementation, this would query threat intel APIs)
        let threat_data = IpThreatData {
            ip_address: ip.to_string(),
            is_malicious: false, // Would be determined by threat intel lookup
            is_vpn: false,       // Would be determined by VPN detection
            is_proxy: false,     // Would be determined by proxy detection
            reputation_score: 0.5, // Neutral score
            country_code: None,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
            threat_types: vec![],
            blocked: false,
        };
        
        self.cache.set(&key, &threat_data, Some(self.ip_reputation_ttl)).await?;
        debug!("Cached new IP reputation for: {}", ip);
        
        Ok(threat_data)
    }
    
    pub async fn block_suspicious_ip(&self, ip: &str, reason: &str) -> Result<(), CacheError> {
        let blocked_ips_key = SecurityCacheKeys::blocked_ips();
        let ip_key = SecurityCacheKeys::ip_reputation(ip);
        
        // Add to blocked IPs set
        let blocked_ip_data = serde_json::json!({
            "ip": ip,
            "blocked_at": Utc::now(),
            "reason": reason,
            "auto_blocked": true
        });
        
        self.cache.set(&format!("{}:{}", blocked_ips_key, ip), &blocked_ip_data, None).await?;
        
        // Update IP reputation to mark as blocked
        if let Some(mut threat_data) = self.cache.get::<IpThreatData>(&ip_key).await? {
            threat_data.blocked = true;
            threat_data.last_seen = Utc::now();
            self.cache.set(&ip_key, &threat_data, Some(self.ip_reputation_ttl)).await?;
        }
        
        warn!("Blocked suspicious IP: {} - Reason: {}", ip, reason);
        Ok(())
    }
    
    pub async fn is_ip_blocked(&self, ip: &str) -> Result<bool, CacheError> {
        let blocked_ips_key = SecurityCacheKeys::blocked_ips();
        let key = format!("{}:{}", blocked_ips_key, ip);
        
        match self.cache.get::<serde_json::Value>(&key).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
    
    pub async fn unblock_ip(&self, ip: &str) -> Result<(), CacheError> {
        let blocked_ips_key = SecurityCacheKeys::blocked_ips();
        let ip_key = SecurityCacheKeys::ip_reputation(ip);
        let key = format!("{}:{}", blocked_ips_key, ip);
        
        // Remove from blocked IPs
        self.cache.delete(&key).await?;
        
        // Update IP reputation to mark as unblocked
        if let Some(mut threat_data) = self.cache.get::<IpThreatData>(&ip_key).await? {
            threat_data.blocked = false;
            threat_data.last_seen = Utc::now();
            self.cache.set(&ip_key, &threat_data, Some(self.ip_reputation_ttl)).await?;
        }
        
        info!("Unblocked IP: {}", ip);
        Ok(())
    }
    
    // Rate limiting and pattern detection
    pub async fn check_rate_limit(
        &self, 
        ip: &str, 
        endpoint: &str, 
        max_requests: i64
    ) -> Result<bool, CacheError> {
        let key = SecurityCacheKeys::rate_limit_tracker(ip, endpoint);
        let current_count = self.cache.increment(&key, 1, Some(self.rate_limit_window)).await?;
        
        if current_count > max_requests {
            warn!("Rate limit exceeded for IP: {} on endpoint: {} ({} > {})", 
                  ip, endpoint, current_count, max_requests);
            return Ok(false);
        }
        
        debug!("Rate limit check passed for IP: {} on endpoint: {} ({}/{})", 
               ip, endpoint, current_count, max_requests);
        Ok(true)
    }
    
    pub async fn detect_suspicious_patterns(
        &self,
        event: &SecurityEvent
    ) -> Result<Vec<ThreatPattern>, CacheError> {
        let mut patterns = Vec::new();
        
        // Check for brute force patterns
        if matches!(event.event_type.as_str(), "BRUTE_FORCE_ATTEMPT" | "MULTIPLE_FAILED_LOGINS") {
            let failed_attempts = self.get_brute_force_attempts(&event.ip).await?;
            if failed_attempts > 5 {
                patterns.push(ThreatPattern {
                    pattern_id: format!("brute_force_{}", event.ip),
                    pattern_type: "BRUTE_FORCE".to_string(),
                    source_ips: vec![event.ip.clone()],
                    detection_count: failed_attempts,
                    first_detected: event.timestamp,
                    last_detected: event.timestamp,
                    severity: "HIGH".to_string(),
                    auto_blocked: failed_attempts > 10,
                });
            }
        }
        
        // Check for geographic anomalies
        if let Some(country_code) = &event.country_code {
            if let Some(user_id) = &event.user_id {
                let geo_key = SecurityCacheKeys::geographic_anomalies(user_id);
                if let Some(last_country) = self.cache.get::<String>(&geo_key).await? {
                    if last_country != *country_code {
                        patterns.push(ThreatPattern {
                            pattern_id: format!("geo_anomaly_{}_{}", user_id, country_code),
                            pattern_type: "GEOGRAPHIC_ANOMALY".to_string(),
                            source_ips: vec![event.ip.clone()],
                            detection_count: 1,
                            first_detected: event.timestamp,
                            last_detected: event.timestamp,
                            severity: "MEDIUM".to_string(),
                            auto_blocked: false,
                        });
                    }
                }
                
                // Update last known location
                self.cache.set(&geo_key, country_code, Some(86400)).await?;
            }
        }
        
        // Store detected patterns
        for pattern in &patterns {
            let pattern_key = format!("{}:{}", SecurityCacheKeys::suspicious_patterns(), pattern.pattern_id);
            self.cache.set(&pattern_key, pattern, Some(self.threat_pattern_ttl)).await?;
        }
        
        if !patterns.is_empty() {
            warn!("Detected {} suspicious patterns for event: {}", patterns.len(), event.event_type);
        }
        
        Ok(patterns)
    }
    
    pub async fn calculate_risk_score(&self, event: &SecurityEvent) -> Result<f64, CacheError> {
        let mut risk_score = 0.0;
        
        // Base risk based on event type
        risk_score += match event.event_type.as_str() {
            "BRUTE_FORCE_DETECTED" => 8.0,
            "SUSPICIOUS_IP_DETECTED" => 7.0,
            "PRIVILEGE_ESCALATION_ATTEMPT" => 9.0,
            "UNAUTHORIZED_API_ACCESS" => 6.0,
            "RATE_LIMIT_EXCEEDED" => 4.0,
            "MULTIPLE_FAILED_LOGINS" => 5.0,
            "GEOGRAPHIC_ANOMALY_DETECTED" => 3.0,
            _ => 1.0,
        };
        
        // IP reputation factor
        if let Ok(ip_data) = self.analyze_ip_reputation(&event.ip).await {
            if ip_data.is_malicious {
                risk_score += 5.0;
            }
            if ip_data.is_vpn || ip_data.is_proxy {
                risk_score += 2.0;
            }
            risk_score += (1.0 - ip_data.reputation_score) * 3.0;
        }
        
        // Failed attempt history
        let failed_attempts = self.get_brute_force_attempts(&event.ip).await?;
        risk_score += (failed_attempts as f64 / 10.0).min(3.0);
        
        // Normalize to 0-10 scale
        let normalized_score = risk_score.min(10.0).max(0.0);
        
        debug!("Calculated risk score {} for event: {} from IP: {}", 
               normalized_score, event.event_type, event.ip);
        
        Ok(normalized_score)
    }
    
    // Enhanced security analytics
    pub async fn get_threat_summary(&self, hours: i64) -> Result<serde_json::Value, CacheError> {
        let current_time = Utc::now();
        let start_time = current_time - chrono::Duration::hours(hours);
        
        // This is a simplified implementation - in practice would aggregate from stored events
        let summary = serde_json::json!({
            "time_range": {
                "start": start_time,
                "end": current_time,
                "hours": hours
            },
            "threat_summary": {
                "high_risk_events": 0,
                "blocked_ips": 0,
                "suspicious_patterns": 0,
                "geographic_anomalies": 0
            },
            "top_threat_ips": [],
            "security_recommendations": [
                "Monitor failed login attempts closely",
                "Consider implementing additional MFA for high-risk accounts",
                "Review geographic access patterns regularly"
            ]
        });
        
        Ok(summary)
    }
}

// Factory for creating SecurityCache instances
pub struct SecurityCacheFactory;

impl SecurityCacheFactory {
    pub fn create(
        cache: Arc<dyn Cache>,
        security_event_ttl: i64,
        session_validation_ttl: i64,
        permission_ttl: i64,
        admin_module_ttl: i64,
    ) -> SecurityCache {
        SecurityCache::new(
            cache,
            security_event_ttl,
            session_validation_ttl,
            permission_ttl,
            admin_module_ttl,
        )
    }
    
    pub fn create_with_threat_detection(
        cache: Arc<dyn Cache>,
        security_event_ttl: i64,
        session_validation_ttl: i64,
        permission_ttl: i64,
        admin_module_ttl: i64,
        ip_reputation_ttl: i64,
        threat_pattern_ttl: i64,
        rate_limit_window: i64,
    ) -> SecurityCache {
        SecurityCache::with_threat_config(
            cache,
            security_event_ttl,
            session_validation_ttl,
            permission_ttl,
            admin_module_ttl,
            ip_reputation_ttl,
            threat_pattern_ttl,
            rate_limit_window,
        )
    }
    
    pub fn from_config(
        cache: Arc<dyn Cache>,
        config: &crate::config::env::SecurityConfig,
    ) -> SecurityCache {
        SecurityCache::new(
            cache,
            config.security_event_cache_ttl,
            config.session_validation_cache_ttl,
            config.permission_cache_ttl,
            config.admin_module_cache_ttl,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::cache::CacheFactory;

    async fn create_test_security_cache() -> SecurityCache {
        let cache = CacheFactory::with_fallback().await;
        
        SecurityCache::new(
            cache,
            86400, // security_event_ttl
            3600,  // session_validation_ttl
            300,   // permission_ttl
            1800,  // admin_module_ttl
        )
    }
    
    #[tokio::test]
    async fn test_admin_session_caching() {
        let security_cache = create_test_security_cache().await;
        
        let session_data = CachedAdminSession {
            user_id: "test-user-123".to_string(),
            email: "admin@example.com".to_string(),
            admin_modules: vec!["user_operations".to_string(), "analytics_specialist".to_string()],
            effective_permissions: vec!["user:read".to_string(), "user:write".to_string()],
            session_id: "session-123".to_string(),
            is_active: true,
            last_activity: Utc::now(),
            cached_at: Utc::now(),
        };
        
        // Cache the session
        security_cache.cache_admin_session("session-123", &session_data).await.unwrap();
        
        // Retrieve the session
        let cached_session = security_cache.get_admin_session("session-123").await.unwrap();
        assert!(cached_session.is_some());
        
        let cached_session = cached_session.unwrap();
        assert_eq!(cached_session.user_id, "test-user-123");
        assert_eq!(cached_session.admin_modules.len(), 2);
    }
    
    #[tokio::test]
    async fn test_brute_force_tracking() {
        let security_cache = create_test_security_cache().await;
        let ip = "192.168.1.100";
        
        // Increment attempts
        let count1 = security_cache.increment_brute_force_attempts(ip, 300).await.unwrap();
        assert_eq!(count1, 1);
        
        let count2 = security_cache.increment_brute_force_attempts(ip, 300).await.unwrap();
        assert_eq!(count2, 2);
        
        // Check current count
        let current_count = security_cache.get_brute_force_attempts(ip).await.unwrap();
        assert_eq!(current_count, 2);
        
        // Clear attempts
        security_cache.clear_brute_force_attempts(ip).await.unwrap();
        let cleared_count = security_cache.get_brute_force_attempts(ip).await.unwrap();
        assert_eq!(cleared_count, 0);
    }
}