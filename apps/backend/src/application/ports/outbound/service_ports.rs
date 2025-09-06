use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::application::shared::{ApplicationError, ApplicationResult};
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