use std::collections::HashMap;use chrono::{DateTime, Utc};// Enhanced error handling system with context and correlation

use serde::{Serialize, Deserialize};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use uuid::Uuid;

use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
use serde_json::json;

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
    InternalServerError,
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
            ErrorKind::InternalServerError => "Internal Server Error",
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

    // Convenience constructors for backwards compatibility
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AggregateNotFound, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AuthorizationError, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ValidationError, message)
    }

    pub fn internal_server_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InternalServerError, message)
    }

    pub fn external_service_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ExternalServiceError, message)
    }

    pub fn database_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::DatabaseError, message)
    }

    pub fn authentication_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AuthenticationError, message)
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ValidationError, message)
    }

    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ConfigurationError, message)
    }

    pub fn security_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AuthorizationError, message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InternalError, message)
    }

    pub fn network_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NetworkError, message)
    }

    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ExternalServiceError, message)
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
            ErrorKind::InternalServerError => 500,
            ErrorKind::DatabaseError => 500,
            ErrorKind::ExternalServiceError => 502,
            ErrorKind::ServiceUnavailable => 503,
            ErrorKind::TimeoutError => 504,
            _ => 500,
        }
    }
}

/// Convert AppError to HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = StatusCode::from_u16(self.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        let sanitized_error = ErrorSanitizer::sanitize_for_user(&self);
        
        let error_response = json!({
            "error": sanitized_error.kind.to_string(),
            "message": sanitized_error.message,
            "correlation_id": sanitized_error.correlation_id,
            "timestamp": sanitized_error.timestamp
        });
        
        (status_code, Json(error_response)).into_response()
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

/// Enhanced error context builder
pub struct ErrorContextBuilder {
    context: ErrorContext,
}

impl ErrorContextBuilder {
    pub fn new(operation: impl Into<String>, service: impl Into<String>) -> Self {
        Self {
            context: ErrorContext {
                operation: operation.into(),
                service: service.into(),
                ..Default::default()
            }
        }
    }
    
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.context.user_id = Some(user_id.into());
        self
    }
    
    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.context.request_id = Some(request_id.into());
        self
    }
    
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.metadata.insert(key.into(), value.into());
        self
    }
    
    pub fn build(self) -> ErrorContext {
        self.context
    }
}

/// Error aggregation for collecting multiple errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCollection {
    pub errors: Vec<AppError>,
    pub summary: String,
}

impl ErrorCollection {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            summary: String::new(),
        }
    }
    
    pub fn add(&mut self, error: AppError) {
        self.errors.push(error);
        self.update_summary();
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn into_result<T>(self, value: T) -> Result<T, AppError> {
        if self.has_errors() {
            Err(AppError::new(
                ErrorKind::ValidationError,
                format!("Multiple errors occurred: {}", self.summary)
            ).with_context(ErrorContext {
                operation: "error_aggregation".to_string(),
                service: "core".to_string(),
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert("error_count".to_string(), self.errors.len().to_string());
                    metadata
                },
                ..Default::default()
            }))
        } else {
            Ok(value)
        }
    }
    
    fn update_summary(&mut self) {
        self.summary = format!("{} validation errors", self.errors.len());
    }
}

/// Structured error logging
pub struct ErrorLogger;

impl ErrorLogger {
    pub fn log_error(error: &AppError) {
        tracing::error!(
            correlation_id = %error.correlation_id,
            error_kind = %error.kind,
            timestamp = %error.timestamp,
            user_id = ?error.context.user_id,
            request_id = ?error.context.request_id,
            operation = %error.context.operation,
            service = %error.context.service,
            metadata = ?error.context.metadata,
            stack_trace = ?error.stack_trace,
            "Error occurred: {}", error.message
        );
    }
    
    pub fn log_warning(error: &AppError) {
        tracing::warn!(
            correlation_id = %error.correlation_id,
            error_kind = %error.kind,
            user_id = ?error.context.user_id,
            operation = %error.context.operation,
            service = %error.context.service,
            "Warning: {}", error.message
        );
    }
    
    pub fn log_recovery_attempt(error: &AppError, recovery_strategy: &str) {
        tracing::info!(
            correlation_id = %error.correlation_id,
            error_kind = %error.kind,
            recovery_strategy = recovery_strategy,
            "Attempting error recovery for: {}", error.message
        );
    }
}

/// Error sanitization for removing sensitive information
pub struct ErrorSanitizer;

impl ErrorSanitizer {
    pub fn sanitize_for_user(error: &AppError) -> AppError {
        let sanitized_message = Self::sanitize_message(&error.message);
        let mut sanitized_error = error.clone();
        sanitized_error.message = sanitized_message;
        sanitized_error.stack_trace = None; // Never expose stack traces to users
        sanitized_error.context.metadata = HashMap::new(); // Remove internal metadata
        sanitized_error
    }
    
    fn sanitize_message(message: &str) -> String {
        let sensitive_patterns = [
            r"password\s*[:=]\s*\S+",
            r"token\s*[:=]\s*\S+",
            r"key\s*[:=]\s*\S+",
            r"secret\s*[:=]\s*\S+",
            r"connection\s+string",
            r"database\s+url",
            r"\b\d{3,4}-\d{2}-\d{4}\b", // SSN pattern
            r"\b\d{4}[\s-]\d{4}[\s-]\d{4}[\s-]\d{4}\b", // Credit card pattern
        ];
        
        let mut sanitized = message.to_string();
        for pattern in &sensitive_patterns {
            let regex = regex::Regex::new(pattern).unwrap();
            sanitized = regex.replace_all(&sanitized, "[REDACTED]").to_string();
        }
        sanitized
    }
}

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

#[macro_export]
macro_rules! database_error {
    ($operation:expr, $details:expr) => {
        AppError::new(
            ErrorKind::DatabaseError, 
            format!("Database operation '{}' failed: {}", $operation, $details)
        ).with_context(
            ErrorContextBuilder::new($operation, "database")
                .metadata("error_type", "database_operation_failed")
                .build()
        )
    };
}

#[macro_export]
macro_rules! external_service_error {
    ($service:expr, $operation:expr, $details:expr) => {
        AppError::new(
            ErrorKind::ExternalServiceError,
            format!("External service '{}' operation '{}' failed: {}", $service, $operation, $details)
        ).with_context(
            ErrorContextBuilder::new($operation, $service)
                .metadata("service_name", $service)
                .metadata("error_type", "external_service_failure")
                .build()
        )
    };
}

#[macro_export]
macro_rules! auth_error {
    ($msg:expr) => {
        AppError::new(ErrorKind::AuthenticationError, $msg)
    };
    ($msg:expr, $($arg:tt)*) => {
        AppError::new(ErrorKind::AuthenticationError, format!($msg, $($arg)*))
    };
}

#[macro_export]
macro_rules! authz_error {
    ($msg:expr) => {
        AppError::new(ErrorKind::AuthorizationError, $msg)
    };
    ($msg:expr, $($arg:tt)*) => {
        AppError::new(ErrorKind::AuthorizationError, format!($msg, $($arg)*))
    };
}

/// Enhanced error context creation macro
#[macro_export]
macro_rules! error_context {
    ($operation:expr, $service:expr) => {
        ErrorContextBuilder::new($operation, $service).build()
    };
    ($operation:expr, $service:expr, user_id = $user_id:expr) => {
        ErrorContextBuilder::new($operation, $service)
            .user_id($user_id)
            .build()
    };
    ($operation:expr, $service:expr, request_id = $request_id:expr) => {
        ErrorContextBuilder::new($operation, $service)
            .request_id($request_id)
            .build()  
    };
    ($operation:expr, $service:expr, user_id = $user_id:expr, request_id = $request_id:expr) => {
        ErrorContextBuilder::new($operation, $service)
            .user_id($user_id)
            .request_id($request_id)
            .build()
    };
}