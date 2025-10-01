use async_trait::async_trait;

// Email service removed - Web3-first system uses direct wallet notifications

/// Push Notification Service Port
#[async_trait]
pub trait NotificationServicePort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn send_push_notification(&self, device_token: &str, message: &str) -> Result<(), Self::Error>;
}

/// External API Service Port
#[async_trait]
pub trait ExternalApiServicePort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn fetch_market_data(&self, symbol: &str) -> Result<MarketData, Self::Error>;
}

/// Security Monitoring Service Port
#[async_trait]
pub trait SecurityMonitoringServicePort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn log_security_event(&self, event: SecurityEvent) -> Result<(), Self::Error>;
    async fn check_threat_level(&self, ip: &str) -> Result<ThreatLevel, Self::Error>;
}

/// Admin Client Service Port
#[async_trait]
pub trait AdminClientPort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn get_admin_user(&self, wallet_address: &str) -> Result<Option<AdminUser>, Self::Error>;
    async fn list_admin_users(&self) -> Result<Vec<AdminUser>, Self::Error>;
}

/// Granular Permissions Client Port
#[async_trait]
pub trait GranularPermissionsClientPort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn get_user_permissions(&self, wallet_address: &str) -> Result<Vec<String>, Self::Error>;
    async fn grant_permission(&self, wallet_address: &str, permission: &str) -> Result<(), Self::Error>;
}

// Email service errors and type aliases removed - Web3-first system uses direct wallet notifications

// Placeholder types
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

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

#[derive(Debug, Clone)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
}