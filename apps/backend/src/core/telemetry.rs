// Comprehensive telemetry system for logging, metrics, and tracing
use chrono::{DateTime, Utc};
use uuid::Uuid;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;


/// Structured logging context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    /// User ID if available
    pub user_id: Option<String>,
    /// Request ID for correlation
    pub request_id: String,
    /// Operation being performed
    pub operation: String,
    /// Service/module name
    pub service: String,
    /// Operation duration if completed
    pub duration: Option<Duration>,
    /// Error information if applicable
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LogContext {
    pub fn new(operation: impl Into<String>, service: impl Into<String>) -> Self {
        Self {
            user_id: None,
            request_id: Uuid::new_v4().to_string(),
            operation: operation.into(),
            service: service.into(),
            duration: None,
            error: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = request_id;
        self
    }
    
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
    
    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Performance statistics
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceStats {
    pub operation: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub error_rate: f64,
    pub throughput_per_second: f64,
    pub last_updated: DateTime<Utc>,
}

/// Health check result
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub response_time_ms: u64,
    pub last_check: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Overall system health
#[derive(Debug, Clone, Serialize)]
pub struct OverallHealth {
    pub status: HealthStatus,
    pub components: Vec<HealthCheckResult>,
    pub healthy_count: usize,
    pub warning_count: usize,
    pub critical_count: usize,
    pub last_check: DateTime<Utc>,
}

/// Alert definition
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub alert_type: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub context: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
    pub metrics_endpoint: Option<String>,
    pub tracing_endpoint: Option<String>,
    pub log_level: String,
    pub sample_rate: f64,
    pub enable_profiling: bool,
}