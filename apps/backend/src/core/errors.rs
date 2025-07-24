// Enhanced error handling system with context and correlation

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use uuid::Uuid;

/// Contextual error with correlation tracking
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("{kind}: {message}")]
pub struct AppError {
    /// Error classification
    pub kind: ErrorKind,
    /// Human-readable error message
    pub message: String,
    /// Additional error context
    pub context: ErrorContext,
    /// Correlation ID for distributed tracing
    pub correlation_id: String,
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// Optional stack trace for debugging
    pub stack_trace: Option<String>,
}

/// Error classification for better handling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorKind {
    // Domain errors
    ValidationError,
    BusinessRuleViolation,
    AggregateNotFound,
    ConcurrencyConflict,
    
    // Infrastructure errors
    DatabaseError,
    NetworkError,
    ExternalServiceError,
    ConfigurationError,
    
    // Application errors
    AuthenticationError,
    AuthorizationError,
    RateLimitExceeded,
    QuotaExceeded,
    
    // System errors
    InternalError,
    ServiceUnavailable,
    TimeoutError,
    ResourceExhausted,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_str = match self {
            ErrorKind::ValidationError => "Validation Error",
            ErrorKind::BusinessRuleViolation => "Business Rule Violation",
            ErrorKind::AggregateNotFound => "Aggregate Not Found",
            ErrorKind::ConcurrencyConflict => "Concurrency Conflict",
            ErrorKind::DatabaseError => "Database Error",
            ErrorKind::NetworkError => "Network Error",
            ErrorKind::ExternalServiceError => "External Service Error",
            ErrorKind::ConfigurationError => "Configuration Error",
            ErrorKind::AuthenticationError => "Authentication Error",
            ErrorKind::AuthorizationError => "Authorization Error",
            ErrorKind::RateLimitExceeded => "Rate Limit Exceeded",
            ErrorKind::QuotaExceeded => "Quota Exceeded",
            ErrorKind::InternalError => "Internal Error",
            ErrorKind::ServiceUnavailable => "Service Unavailable",
            ErrorKind::TimeoutError => "Timeout Error",
            ErrorKind::ResourceExhausted => "Resource Exhausted",
        };
        write!(f, "{}", display_str)
    }
}

/// Error context for debugging and correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// User ID if available
    pub user_id: Option<String>,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Operation being performed
    pub operation: String,
    /// Service/module where error occurred
    pub service: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl AppError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: ErrorContext::default(),
            correlation_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            stack_trace: None,
        }
    }
    
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }
    
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = correlation_id;
        self
    }
    
    pub fn with_stack_trace(mut self, stack_trace: String) -> Self {
        self.stack_trace = Some(stack_trace);
        self
    }
    
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self.kind, 
            ErrorKind::NetworkError |
            ErrorKind::ServiceUnavailable |
            ErrorKind::TimeoutError |
            ErrorKind::ResourceExhausted
        )
    }
    
    /// Get HTTP status code for web responses
    pub fn http_status(&self) -> u16 {
        match self.kind {
            ErrorKind::ValidationError => 400,
            ErrorKind::BusinessRuleViolation => 422,
            ErrorKind::AuthenticationError => 401,
            ErrorKind::AuthorizationError => 403,
            ErrorKind::AggregateNotFound => 404,
            ErrorKind::ConcurrencyConflict => 409,
            ErrorKind::RateLimitExceeded => 429,
            ErrorKind::QuotaExceeded => 429,
            ErrorKind::InternalError => 500,
            ErrorKind::DatabaseError => 500,
            ErrorKind::ExternalServiceError => 502,
            ErrorKind::ServiceUnavailable => 503,
            ErrorKind::TimeoutError => 504,
            _ => 500,
        }
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            user_id: None,
            request_id: None,
            operation: "unknown".to_string(),
            service: "epsx".to_string(),
            metadata: HashMap::new(),
        }
    }
}

/// Error recovery strategy trait
#[async_trait::async_trait]
pub trait ErrorRecovery<T>: Send + Sync {
    async fn recover(&self, error: &AppError) -> Result<Option<T>, AppError>;
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

/// Conversion from standard errors
impl From<Box<dyn std::error::Error>> for AppError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AppError::new(
            ErrorKind::InternalError,
            format!("Internal error: {}", err)
        )
    }
}

/// Domain-specific error macros for convenience
#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        AppError::new(ErrorKind::ValidationError, $msg)
    };
    ($msg:expr, $($arg:tt)*) => {
        AppError::new(ErrorKind::ValidationError, format!($msg, $($arg)*))
    };
}

#[macro_export]
macro_rules! business_error {
    ($msg:expr) => {
        AppError::new(ErrorKind::BusinessRuleViolation, $msg)
    };
    ($msg:expr, $($arg:tt)*) => {
        AppError::new(ErrorKind::BusinessRuleViolation, format!($msg, $($arg)*))
    };
}

#[macro_export]
macro_rules! not_found_error {
    ($entity:expr, $id:expr) => {
        AppError::new(ErrorKind::AggregateNotFound, format!("{} with id {} not found", $entity, $id))
    };
}