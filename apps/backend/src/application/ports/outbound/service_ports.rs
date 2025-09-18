use async_trait::async_trait;

#[async_trait]
pub trait EmailServicePort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), Self::Error>;
    async fn send_template_email(&self, to: &str, template: &str, data: &serde_json::Value) -> Result<(), Self::Error>;
}

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
    
    async fn get_admin_user(&self, user_id: &str) -> Result<Option<AdminUser>, Self::Error>;
    async fn list_admin_users(&self) -> Result<Vec<AdminUser>, Self::Error>;
}

/// Granular Permissions Client Port
#[async_trait]
pub trait GranularPermissionsClientPort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn get_user_permissions(&self, user_id: &str) -> Result<Vec<String>, Self::Error>;
    async fn grant_permission(&self, user_id: &str, permission: &str) -> Result<(), Self::Error>;
}

// Convenience re-exports for errors
#[derive(Debug, thiserror::Error)]
pub enum EmailServiceError {
    #[error("Failed to send email: {0}")]
    SendFailed(String),
    #[error("Invalid email address: {0}")]
    InvalidAddress(String),
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
}

// Type alias for common service
pub type EmailSvc = Box<dyn EmailServicePort<Error = EmailServiceError>>;

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
    pub user_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
}