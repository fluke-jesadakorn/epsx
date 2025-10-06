// Security Monitoring Service Adapter
use crate::prelude::*;

use chrono::Duration;
use tracing::{debug, info, warn};

// use crate::domain::authentication::repositories::SecurityMonitoringServicePort; // REMOVED: authentication domain deleted
use crate::application::ports::outbound::service_ports::{SecurityMonitoringServicePort, SecurityEvent, ThreatLevel};
use crate::infrastructure::cache::Cache;

/// Security monitoring service adapter
pub struct SecurityMonitoringServiceAdapter {
    /// Cache for storing security metrics and rate limiting
    cache: Arc<dyn Cache>,
}

impl SecurityMonitoringServiceAdapter {
    /// Create new security monitoring service
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self {
            cache,
        }
    }
    
    /// Get security metrics key for a user
    fn get_security_metrics_key(&self, wallet_address: &str) -> String {
        format!("security_metrics:{}", wallet_address)
    }
    
    /// Get IP reputation key
    fn get_ip_reputation_key(&self, ip: &str) -> String {
        format!("ip_reputation:{}", ip)
    }
    
    
    /// Update user security metrics
    async fn update_user_security_metrics(&self, wallet_address: &str, metric_type: &str, value: u64) -> Result<(), String> {
        let metrics_key = self.get_security_metrics_key(wallet_address);
        
        // Get existing metrics
        let mut metrics: SecurityMetrics = match self.cache.get(&metrics_key) {
            Some(data) => serde_json::from_str(&data).unwrap_or_default(),
            None => SecurityMetrics::default(),
        };
        
        // Update metrics based on type
        match metric_type {
            "failed_auth" => metrics.failed_auth_attempts += value,
            "suspicious_activity" => metrics.suspicious_activities += value,
            "session_creation" => metrics.sessions_created += value,
            "security_event" => metrics.suspicious_activities += value,
            _ => {
                warn!("Unknown metric type: {}", metric_type);
            }
        }
        
        metrics.last_updated = Utc::now();
        
        // Store updated metrics
        let serialized = serde_json::to_string(&metrics)
            .map_err(|e| format!("Failed to serialize security metrics: {}", e))?;
        
        self.cache.set(&metrics_key, serialized, Some(Duration::days(30).num_seconds() as u64));
        
        Ok(())
    }
}

#[async_trait]
impl SecurityMonitoringServicePort for SecurityMonitoringServiceAdapter {
    type Error = SecurityMonitoringError;
    
    async fn log_security_event(&self, event: SecurityEvent) -> Result<(), Self::Error> {
        info!(
            event_type = event.event_type,
            source_ip = event.source_ip,
            wallet_address = event.wallet_address.as_deref().unwrap_or("unknown"),
            "Logging security event"
        );
        
        // Store event in cache with timestamp key
        let event_key = format!(
            "security_events:{}:{}",
            event.wallet_address.as_deref().unwrap_or("system"),
            Utc::now().timestamp()
        );
        
        let event_data = serde_json::to_string(&event)
            .map_err(|e| SecurityMonitoringError::Serialization(e.to_string()))?;
        self.cache.set(&event_key, event_data, Some(604800)); // 7 days TTL
        
        // Update user metrics if wallet_address is provided
        if let Some(wallet_address) = &event.wallet_address {
            self.update_user_security_metrics(wallet_address, "security_event", 1).await
                .map_err(SecurityMonitoringError::Cache)?;
        }
        
        Ok(())
    }
    
    async fn check_threat_level(&self, ip: &str) -> Result<ThreatLevel, Self::Error> {
        debug!(ip = ip, "Checking threat level for IP");
        
        let ip_rep_key = self.get_ip_reputation_key(ip);
        
        match self.cache.get(&ip_rep_key) {
            Some(data) => {
                let reputation: IpReputation = serde_json::from_str(&data)
                    .map_err(|e| SecurityMonitoringError::Serialization(e.to_string()))?;
                
                let threat_level = if reputation.reputation_score > 80.0 {
                    ThreatLevel::Low
                } else if reputation.reputation_score > 60.0 {
                    ThreatLevel::Medium
                } else if reputation.reputation_score > 30.0 {
                    ThreatLevel::High
                } else {
                    ThreatLevel::Critical
                };
                
                if matches!(threat_level, ThreatLevel::High | ThreatLevel::Critical) {
                    warn!(
                        ip = ip,
                        reputation_score = reputation.reputation_score,
                        threat_level = ?threat_level,
                        "High threat level detected for IP"
                    );
                }
                
                Ok(threat_level)
            },
            None => {
                debug!(ip = ip, "No reputation data found for IP, assuming low threat");
                Ok(ThreatLevel::Low)
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SecurityMonitoringError {
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
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
    pub reputation_score: f64,
    pub failed_attempts: u32,
    pub successful_attempts: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl IpReputation {
    pub fn new(ip_address: String) -> Self {
        Self {
            ip_address,
            reputation_score: 100.0, // Start with good reputation
            failed_attempts: 0,
            successful_attempts: 0,
            first_seen: Utc::now(),
            last_seen: Utc::now(),
        }
    }
    
    pub fn record_failure(&mut self, _reason: &str) {
        self.failed_attempts += 1;
        self.reputation_score = (self.reputation_score - 10.0).max(0.0);
        self.last_seen = Utc::now();
    }
    
    pub fn is_suspicious(&self) -> bool {
        self.reputation_score < 50.0 || self.failed_attempts > 10
    }
}