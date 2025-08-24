// Webhook Models and DTOs
// Data structures for webhook configuration and delivery tracking

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Webhook authentication types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebhookAuthType {
    None,
    Basic,
    Bearer,
    Hmac,
    Custom,
}

impl ToString for WebhookAuthType {
    fn to_string(&self) -> String {
        match self {
            WebhookAuthType::None => "NONE".to_string(),
            WebhookAuthType::Basic => "BASIC".to_string(),
            WebhookAuthType::Bearer => "BEARER".to_string(),
            WebhookAuthType::Hmac => "HMAC".to_string(),
            WebhookAuthType::Custom => "CUSTOM".to_string(),
        }
    }
}

impl From<String> for WebhookAuthType {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "NONE" => WebhookAuthType::None,
            "BASIC" => WebhookAuthType::Basic,
            "BEARER" => WebhookAuthType::Bearer,
            "HMAC" => WebhookAuthType::Hmac,
            "CUSTOM" => WebhookAuthType::Custom,
            _ => WebhookAuthType::None,
        }
    }
}

/// Webhook health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebhookHealthStatus {
    Healthy,
    Degraded,
    Failed,
}

impl ToString for WebhookHealthStatus {
    fn to_string(&self) -> String {
        match self {
            WebhookHealthStatus::Healthy => "HEALTHY".to_string(),
            WebhookHealthStatus::Degraded => "DEGRADED".to_string(),
            WebhookHealthStatus::Failed => "FAILED".to_string(),
        }
    }
}

impl From<String> for WebhookHealthStatus {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "HEALTHY" => WebhookHealthStatus::Healthy,
            "DEGRADED" => WebhookHealthStatus::Degraded,
            "FAILED" => WebhookHealthStatus::Failed,
            _ => WebhookHealthStatus::Healthy,
        }
    }
}

/// Delivery status for webhook notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Retrying,
}

impl ToString for DeliveryStatus {
    fn to_string(&self) -> String {
        match self {
            DeliveryStatus::Pending => "PENDING".to_string(),
            DeliveryStatus::Sent => "SENT".to_string(),
            DeliveryStatus::Delivered => "DELIVERED".to_string(),
            DeliveryStatus::Failed => "FAILED".to_string(),
            DeliveryStatus::Retrying => "RETRYING".to_string(),
        }
    }
}

impl From<String> for DeliveryStatus {
    fn from(s: String) -> Self {
        match s.to_uppercase().as_str() {
            "PENDING" => DeliveryStatus::Pending,
            "SENT" => DeliveryStatus::Sent,
            "DELIVERED" => DeliveryStatus::Delivered,
            "FAILED" => DeliveryStatus::Failed,
            "RETRYING" => DeliveryStatus::Retrying,
            _ => DeliveryStatus::Pending,
        }
    }
}

/// Webhook endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpointConfig {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub http_method: String,
    pub content_type: String,
    pub auth_type: String,
    pub auth_config: serde_json::Value,
    pub custom_headers: serde_json::Value,
    pub payload_template: Option<serde_json::Value>,
    pub retry_attempts: Option<i32>,
    pub retry_backoff_seconds: Option<i32>,
    pub timeout_seconds: Option<i32>,
    pub alert_severity_filter: Option<Vec<String>>,
    pub alert_category_filter: Option<Vec<String>>,
    pub is_active: bool,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub failure_count: Option<i32>,
    pub health_status: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

/// Create webhook endpoint request
#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub url: String,
    pub http_method: Option<String>,
    pub content_type: Option<String>,
    pub auth_type: WebhookAuthType,
    pub auth_config: Option<serde_json::Value>,
    pub custom_headers: Option<HashMap<String, String>>,
    pub payload_template: Option<serde_json::Value>,
    pub retry_attempts: Option<i32>,
    pub retry_backoff_seconds: Option<i32>,
    pub timeout_seconds: Option<i32>,
    pub alert_severity_filter: Option<Vec<String>>,
    pub alert_category_filter: Option<Vec<String>>,
    pub description: Option<String>,
}

/// Update webhook endpoint request
#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub http_method: Option<String>,
    pub content_type: Option<String>,
    pub auth_type: Option<WebhookAuthType>,
    pub auth_config: Option<serde_json::Value>,
    pub custom_headers: Option<HashMap<String, String>>,
    pub payload_template: Option<serde_json::Value>,
    pub retry_attempts: Option<i32>,
    pub retry_backoff_seconds: Option<i32>,
    pub timeout_seconds: Option<i32>,
    pub alert_severity_filter: Option<Vec<String>>,
    pub alert_category_filter: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub description: Option<String>,
}

/// Webhook delivery record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub alert_id: Uuid,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub scheduled_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Webhook delivery statistics
#[derive(Debug, Serialize)]
pub struct WebhookStats {
    pub webhook_id: Uuid,
    pub webhook_name: String,
    pub total_deliveries: i64,
    pub successful_deliveries: i64,
    pub failed_deliveries: i64,
    pub pending_deliveries: i64,
    pub average_response_time: Option<f64>,
    pub success_rate: f64,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub health_status: String,
    pub current_failure_count: i32,
}

/// Webhook test request
#[derive(Debug, Deserialize)]
pub struct TestWebhookRequest {
    pub test_payload: Option<serde_json::Value>,
}

/// Webhook test response
#[derive(Debug, Serialize)]
pub struct TestWebhookResponse {
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub response_time_ms: u128,
    pub error_message: Option<String>,
}

/// Webhook authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookAuthConfig {
    pub auth_type: WebhookAuthType,
    pub credentials: AuthCredentials,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AuthCredentials {
    Basic { username: String, password: String },
    Bearer { token: String },
    Hmac { secret: String, algorithm: String },
    Custom { headers: HashMap<String, String> },
}

/// Webhook payload templates for different alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadTemplate {
    pub name: String,
    pub description: String,
    pub template: serde_json::Value,
    pub variables: Vec<PayloadVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadVariable {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

/// Webhook retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: i32,
    pub initial_delay_seconds: i32,
    pub max_delay_seconds: i32,
    pub backoff_multiplier: f64,
    pub retry_http_codes: Vec<u16>,
    pub stop_on_codes: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_seconds: 60,
            max_delay_seconds: 3600,
            backoff_multiplier: 2.0,
            retry_http_codes: vec![500, 502, 503, 504, 429],
            stop_on_codes: vec![400, 401, 403, 404],
        }
    }
}

/// Webhook health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_minutes: i32,
    pub timeout_seconds: i32,
    pub failure_threshold: i32,
    pub success_threshold: i32,
    pub check_payload: serde_json::Value,
    pub expected_status_codes: Vec<u16>,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 15,
            timeout_seconds: 10,
            failure_threshold: 3,
            success_threshold: 2,
            check_payload: serde_json::json!({
                "type": "health_check",
                "timestamp": "{{timestamp}}"
            }),
            expected_status_codes: vec![200, 201, 202, 204],
        }
    }
}

/// Webhook rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: i32,
    pub requests_per_hour: i32,
    pub burst_limit: i32,
    pub cooldown_seconds: i32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_limit: 10,
            cooldown_seconds: 60,
        }
    }
}

/// Webhook delivery result
#[derive(Debug)]
pub struct DeliveryResult {
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub response_time_ms: u128,
    pub error_message: Option<String>,
    pub should_retry: bool,
    pub retry_after_seconds: Option<u64>,
}

/// Webhook queue item for background processing
#[derive(Debug, Clone)]
pub struct WebhookQueueItem {
    pub delivery_id: Uuid,
    pub webhook_id: Uuid,
    pub alert_id: Uuid,
    pub payload: serde_json::Value,
    pub attempt_count: i32,
    pub scheduled_at: DateTime<Utc>,
    pub priority: WebhookPriority,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WebhookPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl From<String> for WebhookPriority {
    fn from(severity: String) -> Self {
        match severity.to_uppercase().as_str() {
            "CRITICAL" => WebhookPriority::Critical,
            "HIGH" => WebhookPriority::High,
            "MEDIUM" => WebhookPriority::Normal,
            "LOW" => WebhookPriority::Low,
            _ => WebhookPriority::Normal,
        }
    }
}

/// Webhook processing metrics
#[derive(Debug, Clone, Serialize)]
pub struct WebhookMetrics {
    pub total_webhooks: i64,
    pub active_webhooks: i64,
    pub healthy_webhooks: i64,
    pub degraded_webhooks: i64,
    pub failed_webhooks: i64,
    pub total_deliveries_today: i64,
    pub successful_deliveries_today: i64,
    pub failed_deliveries_today: i64,
    pub average_response_time_ms: f64,
    pub queue_size: i64,
    pub retry_queue_size: i64,
    pub processing_rate_per_minute: f64,
    pub error_rate_percentage: f64,
    pub generated_at: DateTime<Utc>,
}

/// Error types for webhook system
#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
    
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Webhook not found: {webhook_id}")]
    WebhookNotFound { webhook_id: Uuid },
    
    #[error("Webhook delivery failed: {webhook_id}, attempt {attempt}")]
    DeliveryFailed { webhook_id: Uuid, attempt: i32 },
    
    #[error("Webhook authentication failed: {webhook_id}")]
    AuthenticationFailed { webhook_id: Uuid },
    
    #[error("Webhook rate limited: {webhook_id}")]
    RateLimited { webhook_id: Uuid },
    
    #[error("Webhook health check failed: {webhook_id}")]
    HealthCheckFailed { webhook_id: Uuid },
    
    #[error("Invalid webhook configuration: {message}")]
    InvalidConfiguration { message: String },
    
    #[error("Webhook timeout: {webhook_id}")]
    Timeout { webhook_id: Uuid },
    
    #[error("Queue full: cannot schedule more webhooks")]
    QueueFull,
    
    #[error("Template rendering error: {message}")]
    TemplateError { message: String },
}

pub type WebhookResult<T> = Result<T, WebhookError>;

/// Webhook event types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebhookEventType {
    Created,
    Updated,
    Deleted,
    Activated,
    Deactivated,
    DeliverySucceeded,
    DeliveryFailed,
    HealthCheckPassed,
    HealthCheckFailed,
    RateLimited,
    AuthenticationFailed,
}

/// Webhook audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookAuditLog {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: WebhookEventType,
    pub details: serde_json::Value,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}