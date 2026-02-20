// Real-time Threat Detection and Security Monitoring
// Detects anomalies, token tampering attempts, and security violations

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use tracing::{warn, error, info};
use std::sync::Arc;
use crate::infrastructure::cache::Cache;

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityEvent {
    InvalidJwtAttempt,
    PermissionIntegrityViolation,
    DeviceBindingViolation,
    SuspiciousRefreshPattern,
    UnauthorizedPermissionAccess,
    MultipleFailedAttempts,
    TokenReplayAttack,
    AnomalousPermissionRequest,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub wallet_address: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub request_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecurityMetrics {
    failed_attempts: u32,
    last_failure: DateTime<Utc>,
    threat_level: u8,
    blocked_until: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub enum SecurityError {
    MonitoringFailed(String),
    BlockedUser(String),
    RateLimited(String),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SecurityError::MonitoringFailed(msg) => write!(f, "Security monitoring failed: {}", msg),
            SecurityError::BlockedUser(msg) => write!(f, "User blocked: {}", msg),
            SecurityError::RateLimited(msg) => write!(f, "Rate limited: {}", msg),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Real-time threat detection and security monitoring service
pub struct ThreatDetectionService {
    cache: Option<Arc<dyn Cache>>,
}

impl Default for ThreatDetectionService {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ThreatDetectionService {
    pub fn new(cache: Option<Arc<dyn Cache>>) -> Self {
        Self { cache }
    }
    
    // Key generators for Redis cache
    fn metrics_key(wallet_address: &str) -> String {
        format!("threat:metrics:{}", wallet_address)
    }

    fn block_key(wallet_address: &str) -> String {
        format!("threat:blocked:{}", wallet_address)
    }

    /// Set the cache for global instance (called during app startup)
    pub fn set_cache(&mut self, cache: Arc<dyn Cache>) {
        self.cache = Some(cache);
    }

    /// Analyze security event and determine threat level
    pub async fn analyze_security_event(
        &self,
        event: SecurityEvent,
        context: SecurityContext,
    ) -> Result<ThreatLevel, SecurityError> {
        // Check if user is currently blocked
        self.check_user_blocked(&context.wallet_address)?;
        
        // Log security event
        self.log_security_event(&event, &context).await?;
        
        // Analyze threat level
        let threat_level = self.calculate_threat_level(&event, &context).await?;
        
        // Take action based on threat level
        self.handle_threat_response(&threat_level, &context).await?;
        
        Ok(threat_level)
    }
    
    /// Check if user is currently blocked
    pub fn check_user_blocked(&self, wallet_address: &str) -> Result<(), SecurityError> {
        if let Some(cache) = &self.cache {
            let key = Self::block_key(wallet_address);
            if let Some(blocked_until_str) = cache.get(&key) {
                if let Ok(blocked_until) = serde_json::from_str::<DateTime<Utc>>(&blocked_until_str) {
                    if Utc::now() < blocked_until {
                        return Err(SecurityError::BlockedUser(format!(
                            "User {} blocked until {}", wallet_address, blocked_until
                        )));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Log security event for audit trail
    async fn log_security_event(
        &self,
        event: &SecurityEvent,
        context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        // Load existing metrics
        let mut user_metrics = SecurityMetrics {
            failed_attempts: 0,
            last_failure: Utc::now(),
            threat_level: 1,
            blocked_until: None,
        };

        let metrics_key = Self::metrics_key(&context.wallet_address);

        if let Some(cache) = &self.cache {
            if let Some(metrics_str) = cache.get(&metrics_key) {
                if let Ok(existing_metrics) = serde_json::from_str::<SecurityMetrics>(&metrics_str) {
                    user_metrics = existing_metrics;
                }
            }
        }
        
        match event {
            SecurityEvent::InvalidJwtAttempt |
            SecurityEvent::PermissionIntegrityViolation |
            SecurityEvent::DeviceBindingViolation => {
                user_metrics.failed_attempts += 1;
                user_metrics.last_failure = Utc::now();
            },
            _ => {}
        }
        
        // Save updated metrics (expire after 24 hours of inactivity)
        if let Some(cache) = &self.cache {
            if let Ok(serialized) = serde_json::to_string(&user_metrics) {
                cache.set(&metrics_key, serialized, Some(86400)); // 24 hours
            }
        }
        
        // Log to structured logging
        match event {
            SecurityEvent::InvalidJwtAttempt => {
                warn!(
                    wallet_address = %context.wallet_address,
                    ip = ?context.ip_address,
                    "Invalid JWT token attempt detected"
                );
            },
            SecurityEvent::PermissionIntegrityViolation => {
                error!(
                    wallet_address = %context.wallet_address,
                    ip = ?context.ip_address,
                    "Permission integrity violation - possible token tampering"
                );
            },
            SecurityEvent::DeviceBindingViolation => {
                warn!(
                    wallet_address = %context.wallet_address,
                    ip = ?context.ip_address,
                    device_fingerprint = ?context.device_fingerprint,
                    "Device binding violation - token used from different device"
                );
            },
            SecurityEvent::UnauthorizedPermissionAccess => {
                warn!(
                    wallet_address = %context.wallet_address,
                    path = ?context.request_path,
                    "Unauthorized permission access attempt"
                );
            },
            _ => {
                info!(
                    event = ?event,
                    wallet_address = %context.wallet_address,
                    "Security event recorded"
                );
            }
        }
        
        Ok(())
    }
    
    /// Calculate threat level based on event and context
    async fn calculate_threat_level(
        &self,
        event: &SecurityEvent,
        context: &SecurityContext,
    ) -> Result<ThreatLevel, SecurityError> {
        let metrics_key = Self::metrics_key(&context.wallet_address);
        let mut user_metrics = None;

        if let Some(cache) = &self.cache {
            if let Some(metrics_str) = cache.get(&metrics_key) {
                user_metrics = serde_json::from_str::<SecurityMetrics>(&metrics_str).ok();
            }
        }
        
        let base_threat_level = match event {
            SecurityEvent::InvalidJwtAttempt => ThreatLevel::Low,
            SecurityEvent::DeviceBindingViolation => ThreatLevel::Medium,
            SecurityEvent::PermissionIntegrityViolation => ThreatLevel::Critical,
            SecurityEvent::TokenReplayAttack => ThreatLevel::High,
            SecurityEvent::UnauthorizedPermissionAccess => ThreatLevel::Medium,
            SecurityEvent::SuspiciousRefreshPattern => ThreatLevel::Medium,
            SecurityEvent::MultipleFailedAttempts => ThreatLevel::High,
            SecurityEvent::AnomalousPermissionRequest => ThreatLevel::Low,
        };
        
        // Escalate based on user history
        let escalated_threat = if let Some(metrics) = user_metrics {
            match metrics.failed_attempts {
                0..=2 => base_threat_level,
                3..=5 => std::cmp::max(base_threat_level, ThreatLevel::Medium),
                6..=10 => std::cmp::max(base_threat_level, ThreatLevel::High),
                _ => ThreatLevel::Critical,
            }
        } else {
            base_threat_level
        };
        
        Ok(escalated_threat)
    }
    
    /// Handle threat response based on threat level
    async fn handle_threat_response(
        &self,
        threat_level: &ThreatLevel,
        context: &SecurityContext,
    ) -> Result<(), SecurityError> {
        match threat_level {
            ThreatLevel::Low => {
                // Just log, no action needed
                info!(
                    wallet_address = %context.wallet_address,
                    "Low threat level - monitoring"
                );
            },
            ThreatLevel::Medium => {
                warn!(
                    wallet_address = %context.wallet_address,
                    "Medium threat level - increased monitoring"
                );
            },
            ThreatLevel::High => {
                error!(
                    wallet_address = %context.wallet_address,
                    "High threat level - temporary restrictions applied"
                );
                self.apply_temporary_restrictions(&context.wallet_address, Duration::minutes(15)).await?;
            },
            ThreatLevel::Critical => {
                error!(
                    wallet_address = %context.wallet_address,
                    "Critical threat level - user blocked"
                );
                self.block_user(&context.wallet_address, Duration::hours(1)).await?;
                // In production: notify security team, trigger incident response
            },
        }
        
        Ok(())
    }
    
    /// Apply temporary restrictions to user
    async fn apply_temporary_restrictions(
        &self,
        wallet_address: &str,
        duration: Duration,
    ) -> Result<(), SecurityError> {
        let blocked_until = Utc::now() + duration;
        let block_key = Self::block_key(wallet_address);

        if let Some(cache) = &self.cache {
            if let Ok(serialized) = serde_json::to_string(&blocked_until) {
                // Set TTL so Redis automatically clears the block
                cache.set(&block_key, serialized, Some(duration.num_seconds() as u64));
            }
        }
        
        warn!(
            wallet_address = %wallet_address,
            blocked_until = %blocked_until,
            "User temporarily restricted due to security concerns"
        );
        
        Ok(())
    }
    
    /// Block user for security violation
    async fn block_user(&self, wallet_address: &str, duration: Duration) -> Result<(), SecurityError> {
        let blocked_until = Utc::now() + duration;
        let block_key = Self::block_key(wallet_address);

        if let Some(cache) = &self.cache {
            if let Ok(serialized) = serde_json::to_string(&blocked_until) {
                cache.set(&block_key, serialized, Some(duration.num_seconds() as u64));
            }
        }
        
        error!(
            wallet_address = %wallet_address,
            blocked_until = %blocked_until,
            "User blocked due to critical security violation"
        );
        
        Ok(())
    }
    
    /// Get security summary for user
    pub fn get_security_summary(&self, wallet_address: &str) -> Option<SecuritySummary> {
        let block_key = Self::block_key(wallet_address);
        let metrics_key = Self::metrics_key(wallet_address);

        if let Some(cache) = &self.cache {
            let user_metrics = cache.get(&metrics_key)
                .and_then(|data| serde_json::from_str::<SecurityMetrics>(&data).ok());
            
            let blocked_until = cache.get(&block_key)
                .and_then(|data| serde_json::from_str::<DateTime<Utc>>(&data).ok());

            if user_metrics.is_none() && blocked_until.is_none() {
                return None;
            }

            let metrics = user_metrics.unwrap_or_else(|| SecurityMetrics {
                failed_attempts: 0,
                last_failure: Utc::now(),
                threat_level: 1,
                blocked_until: None,
            });

            return Some(SecuritySummary {
                wallet_address: wallet_address.to_string(),
                failed_attempts: metrics.failed_attempts,
                last_failure: metrics.last_failure,
                threat_level: match metrics.threat_level {
                    1 => ThreatLevel::Low,
                    2 => ThreatLevel::Medium,
                    3 => ThreatLevel::High,
                    4 => ThreatLevel::Critical,
                    _ => ThreatLevel::Low,
                },
                blocked_until,
                is_blocked: blocked_until.is_some_and(|until| Utc::now() < until),
            });
        }
        
        None
    }
    
    /// Reset security metrics for user (admin function)
    pub fn reset_user_security(&self, wallet_address: &str) {
        if let Some(cache) = &self.cache {
            cache.delete(&Self::metrics_key(wallet_address));
            cache.delete(&Self::block_key(wallet_address));
        }
        
        info!(
            wallet_address = %wallet_address,
            "Security metrics reset for user"
        );
    }
    
    /// Clean up expired blocks and old metrics (now handled automatically by Redis TTL)
    pub async fn cleanup_expired_data(&self) {
        info!("Cleanup now handled by Redis TTL expiration");
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SecuritySummary {
    pub wallet_address: String,
    pub failed_attempts: u32,
    pub last_failure: DateTime<Utc>,
    pub threat_level: ThreatLevel,
    pub blocked_until: Option<DateTime<Utc>>,
    pub is_blocked: bool,
}

// Global threat detection service logic
use std::sync::RwLock;
static THREAT_DETECTION: RwLock<Option<Arc<ThreatDetectionService>>> = RwLock::new(None);

/// Get global threat detection service instance
pub fn get_threat_detection_service() -> Arc<ThreatDetectionService> {
    let instance = THREAT_DETECTION.read().unwrap();
    if instance.is_none() {
        drop(instance);
        let mut write_instance = THREAT_DETECTION.write().unwrap();
        if write_instance.is_none() {
            *write_instance = Some(Arc::new(ThreatDetectionService::new(None)));
        }
        return write_instance.as_ref().unwrap().clone();
    }
    instance.as_ref().unwrap().clone()
}

pub fn initialize_global_threat_detection(cache: Arc<dyn Cache>) {
    let mut write_instance = THREAT_DETECTION.write().unwrap();
    *write_instance = Some(Arc::new(ThreatDetectionService::new(Some(cache))));
    info!("Initialized global threat detection service with cache");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::cache::memory_cache::MemoryCache;
    
    #[tokio::test]
    async fn test_threat_escalation() {
        let cache = Arc::new(MemoryCache::new());
        let service = ThreatDetectionService::new(Some(cache));
        let context = SecurityContext {
            wallet_address: "test_user".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: None,
            device_fingerprint: None,
            timestamp: Utc::now(),
            request_path: None,
        };
        
        // First attempt should be low threat
        let threat1 = service.analyze_security_event(
            SecurityEvent::InvalidJwtAttempt,
            context.clone(),
        ).await.unwrap();
        assert_eq!(threat1, ThreatLevel::Low);
        
        // Multiple attempts should escalate
        for _ in 0..4 {
            let _ = service.analyze_security_event(
                SecurityEvent::InvalidJwtAttempt,
                context.clone(),
            ).await;
        }
        
        // Should be higher threat level now
        let final_threat = service.analyze_security_event(
            SecurityEvent::InvalidJwtAttempt,
            context.clone(),
        ).await.unwrap();
        assert!(final_threat >= ThreatLevel::Medium);
    }
    
    #[tokio::test]
    async fn test_user_blocking() {
        let cache = Arc::new(MemoryCache::new());
        let service = ThreatDetectionService::new(Some(cache));
        let context = SecurityContext {
            wallet_address: "blocked_user".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: None,
            device_fingerprint: None,
            timestamp: Utc::now(),
            request_path: None,
        };
        
        // Trigger critical event
        service.analyze_security_event(
            SecurityEvent::PermissionIntegrityViolation,
            context.clone(),
        ).await.unwrap();
        
        // User should be blocked now
        let result = service.check_user_blocked("blocked_user");
        assert!(result.is_err());
    }
}