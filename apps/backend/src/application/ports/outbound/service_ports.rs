use async_trait::async_trait;

// Email service removed - Web3-first system uses direct wallet notifications

/// Push Notification Service Port
#[async_trait]
pub trait NotificationServicePort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn send_push_notification(&self, device_token: &str, message: &str) -> Result<(), Self::Error>;
}

// External API Service Port - REMOVED
// No implementation found - market data fetched via TradingViewRestClient instead

/// Security Monitoring Service Port
#[async_trait]
pub trait SecurityMonitoringServicePort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn log_security_event(&self, event: SecurityEvent) -> Result<(), Self::Error>;
    async fn check_threat_level(&self, ip: &str) -> Result<ThreatLevel, Self::Error>;
}

// Admin Client Service Port - REMOVED
// Unused placeholder implementation removed - admin functionality handled by Web3PermissionServiceAdapter

// Granular Permissions Client Port - REMOVED
// Unused placeholder implementation removed - permission management handled by WalletUserPermissionRepository

// Email service errors and type aliases removed - Web3-first system uses direct wallet notifications

// MarketData type removed - no longer needed after removing ExternalApiServicePort

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityEvent {
    pub event_type: String,
    pub source_ip: String,
    pub wallet_address: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecuritySummary {
    pub wallet_address: String,
    pub recent_login_attempts: u32,
    pub failed_attempts: u32,
    pub suspicious_activities: u32,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub risk_score: ThreatLevel,
    pub is_locked: bool,
}

// AdminUser type removed - no longer needed after removing AdminClientPort