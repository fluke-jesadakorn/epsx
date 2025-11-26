use std::collections::HashMap;
use chrono::{DateTime, Utc};
// Enhanced unified error handling system with context and correlation

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
    pub context: Box<ErrorContext>,
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
    pub wallet_address: Option<String>,
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
            context: Box::new(ErrorContext::default()),
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
        Self::new(ErrorKind::AuthenticationError, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
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

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ConcurrencyConflict, message)
    }

    pub fn entity_not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::AggregateNotFound, message)
    }

    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::BusinessRuleViolation, message)
    }

    pub fn blockchain_rpc_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::ExternalServiceError, message)
    }
    
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Box::new(context);
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
    
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.context.service = component.into();
        self
    }
    
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.context.operation = operation.into();
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

// Unified error conversions - standardize all domain errors to AppError
impl From<crate::application::shared::error::ApplicationError> for AppError {
    fn from(err: crate::application::shared::error::ApplicationError) -> Self {
        use crate::application::shared::error::ApplicationError;
        match err {
            ApplicationError::Validation { field, message } => 
                AppError::validation_error(format!("Validation failed for {}: {}", field, message)),
            ApplicationError::Authorization { action } => 
                AppError::unauthorized(format!("Authorization failed: {} not allowed", action)),
            ApplicationError::Domain(domain_err) => 
                AppError::business_rule_violation(domain_err.to_string()),
            ApplicationError::NotFound { resource_type, id } => 
                AppError::not_found(format!("{} with id {} not found", resource_type, id)),
            ApplicationError::Infrastructure { message } => 
                AppError::external_service_error(message),
            ApplicationError::Conflict { resource } => 
                AppError::conflict(format!("Conflict: {} already exists", resource)),
            ApplicationError::ExternalService { service, message } => 
                AppError::external_service_error(format!("{}: {}", service, message)),
            ApplicationError::Concurrency { message } => 
                AppError::conflict(format!("Concurrency error: {}", message)),
            ApplicationError::BusinessRule { rule } => 
                AppError::business_rule_violation(format!("Business rule violation: {}", rule)),
            ApplicationError::NotImplemented { feature } => 
                AppError::internal_error(format!("Feature not implemented: {}", feature)),
            ApplicationError::BusinessLogic { message } => 
                AppError::business_rule_violation(format!("Business logic error: {}", message)),
            ApplicationError::Security { message } => 
                AppError::unauthorized(format!("Security error: {}", message)),
            ApplicationError::WalletAddress(wallet_err) =>
                AppError::validation_error(format!("Invalid wallet address: {}", wallet_err)),
            ApplicationError::ValueObject(value_err) =>
                AppError::validation_error(format!("Value object error: {}", value_err)),
        }
    }
}

impl From<crate::domain::shared_kernel::value_object::ValueObjectError> for AppError {
    fn from(err: crate::domain::shared_kernel::value_object::ValueObjectError) -> Self {
        use crate::domain::shared_kernel::value_object::ValueObjectError;
        match err {
            ValueObjectError::InvalidFormat(msg) => 
                AppError::validation_error(format!("Invalid format: {}", msg)),
            ValueObjectError::OutOfRange(msg) => 
                AppError::validation_error(format!("Value out of range: {}", msg)),
            ValueObjectError::Required(field) => 
                AppError::validation_error(format!("Required field missing: {}", field)),
            ValueObjectError::ValidationFailed(msg) => 
                AppError::validation_error(format!("Validation failed: {}", msg)),
        }
    }
}

// Diesel error conversions for core::errors::AppError
impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => AppError::not_found("Record not found"),
            diesel::result::Error::DatabaseError(_, info) => AppError::database_error(info.message().to_string()),
            _ => AppError::database_error(format!("Database error: {}", err)),
        }
    }
}

// Deadpool error conversion for connection pool errors
// Using the re-exported types from diesel_async
impl From<diesel_async::pooled_connection::PoolError> for AppError {
    fn from(err: diesel_async::pooled_connection::PoolError) -> Self {
        AppError::database_error(format!("Connection pool error: {}", err))
    }
}

// Additional domain-specific error conversions
// Note: WalletAddressError -> AppError conversion is implemented in wallet_address.rs to avoid conflicts

// Note: EPSError -> AppError conversion already exists in web::analytics::eps::errors
// Removed duplicate implementation to avoid conflicts

// Generic string error conversion (for cases where we're still using String errors)
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::internal_error(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::internal_error(err.to_string())
    }
}

// Standard library error conversions
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::external_service_error(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::validation_error(format!("JSON parsing error: {}", err))
    }
}

impl AppError {
    pub fn business_rule_violation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::BusinessRuleViolation, message)
    }
    
    /// Create AppError with full context
    pub fn with_full_context(
        kind: ErrorKind,
        message: impl Into<String>,
        wallet_address: Option<String>,
        request_id: Option<String>,
        operation: impl Into<String>,
        service: impl Into<String>,
    ) -> Self {
        let context = ErrorContext {
            wallet_address,
            request_id,
            operation: operation.into(),
            service: service.into(),
            metadata: std::collections::HashMap::new(),
        };
        
        Self::new(kind, message).with_context(context)
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            wallet_address: None,
            request_id: None,
            operation: "unknown".to_string(),
            service: "epsx".to_string(),
            metadata: HashMap::new(),
        }
    }
}

// ============================================================================
// UNIFIED RESULT TYPES - Standardize all Result types to use AppError
// ============================================================================

/// Standard result type for all operations that can fail
/// This replaces inconsistent Result<T, various_error_types> across the codebase
pub type AppResult<T> = std::result::Result<T, AppError>;

/// Result type for async operations
pub type AsyncResult<T> = std::result::Result<T, AppError>;

/// Result type for web/API handlers - all handlers should return this
pub type ApiResult<T> = std::result::Result<T, AppError>;

/// Result type for application layer operations
pub type ApplicationResult<T> = std::result::Result<T, AppError>;

/// Result type for infrastructure operations  
pub type InfrastructureResult<T> = std::result::Result<T, AppError>;

/// Convenience type alias for operations that don't return data
pub type EmptyResult = AppResult<()>;

/// Error recovery strategy trait
#[async_trait::async_trait]
pub trait ErrorRecovery<T>: Send + Sync {
    async fn recover(&self, error: &AppError) -> AppResult<Option<T>>;
}

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
    
    pub fn user_id(mut self, wallet_address: impl Into<String>) -> Self {
        self.context.wallet_address = Some(wallet_address.into());
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

impl Default for ErrorCollection {
    fn default() -> Self {
        Self::new()
    }
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
            wallet_address = ?error.context.wallet_address,
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
            wallet_address = ?error.context.wallet_address,
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
            match regex::Regex::new(pattern) {
                Ok(regex) => {
                    sanitized = regex.replace_all(&sanitized, "[REDACTED]").to_string();
                }
                Err(e) => {
                    // This should never happen with hardcoded patterns, but log if it does
                    tracing::warn!("Failed to compile sanitization regex {}: {}", pattern, e);
                }
            }
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
    ($operation:expr, $service:expr, user_id = $wallet_address:expr) => {
        ErrorContextBuilder::new($operation, $service)
            .wallet_address($user_id)
            .build()
    };
    ($operation:expr, $service:expr, request_id = $request_id:expr) => {
        ErrorContextBuilder::new($operation, $service)
            .request_id($request_id)
            .build()  
    };
    ($operation:expr, $service:expr, user_id = $wallet_address:expr, request_id = $request_id:expr) => {
        ErrorContextBuilder::new($operation, $service)
            .wallet_address($user_id)
            .request_id($request_id)
            .build()
    };
}

// ============================================================================
// WEB3 TYPES - Wallet validation and permission types for Web3-first architecture
// ============================================================================

/// Wallet validation types for Web3 authentication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WalletValidationType {
    AddressFormat,
    SignatureVerification, 
    NonceValidation,
    ChainMismatch,
    InsufficientBalance,
    ContractInteraction,
}

/// Web3 permission types for blockchain-based access control
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Web3PermissionType {
    Manual,
    NftGated,
    TokenGated,
    DaoGovernance,
    CrossChain,
}