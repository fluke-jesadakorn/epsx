//! Unified application error system with context, correlation, and severity.
//!
//! Source: apps/backend/src/core/errors.rs + apps/backend/src/domain/shared_kernel/app_error.rs
//! Migrated to a single, consolidated error type for the kernel crate.

use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::value_object::ValueObjectError;

/// Error classification for handling and HTTP response mapping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorKind {
    ValidationError,
    BusinessRuleViolation,
    AggregateNotFound,
    ConcurrencyConflict,
    DatabaseError,
    NetworkError,
    ExternalServiceError,
    ConfigurationError,
    AuthenticationError,
    AuthorizationError,
    RateLimitExceeded,
    QuotaExceeded,
    InternalError,
    InternalServerError,
    ServiceUnavailable,
    TimeoutError,
    ResourceExhausted,
    EntityNotFound,
    PermissionDenied,
    ResourceConflict,
    WalletValidationError,
    Web3PermissionValidationError,
    SmartContractError,
    CrossChainError,
    CacheError,
    AuthenticationTypeError,
    BlockchainRpcError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::fmt::Result {
        let s = match self {
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
            ErrorKind::EntityNotFound => "Entity Not Found",
            ErrorKind::PermissionDenied => "Permission Denied",
            ErrorKind::ResourceConflict => "Resource Conflict",
            ErrorKind::WalletValidationError => "Wallet Validation Error",
            ErrorKind::Web3PermissionValidationError => "Web3 Permission Validation Error",
            ErrorKind::SmartContractError => "Smart Contract Error",
            ErrorKind::CrossChainError => "Cross Chain Error",
            ErrorKind::CacheError => "Cache Error",
            ErrorKind::AuthenticationTypeError => "Authentication Error",
            ErrorKind::BlockchainRpcError => "Blockchain RPC Error",
        };
        write!(f, "{}", s)
    }
}

/// Error context with correlation tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub wallet_address: Option<String>,
    pub request_id: Option<String>,
    pub operation: String,
    pub service: String,
    pub metadata: HashMap<String, String>,
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

/// Error severity for monitoring and alerting.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::fmt::Result {
        let s = match self {
            ErrorSeverity::Low => "LOW",
            ErrorSeverity::Medium => "MEDIUM",
            ErrorSeverity::High => "HIGH",
            ErrorSeverity::Critical => "CRITICAL",
        };
        write!(f, "{}", s)
    }
}

/// Unified application error type for the entire EPSX platform.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("{kind}: {message}")]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: Box<ErrorContext>,
    pub correlation_id: String,
    pub timestamp: DateTime<Utc>,
    pub stack_trace: Option<String>,
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
    pub fn business_rule_violation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::BusinessRuleViolation, message)
    }
    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TimeoutError, message)
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

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::NetworkError
                | ErrorKind::ServiceUnavailable
                | ErrorKind::TimeoutError
                | ErrorKind::ResourceExhausted
                | ErrorKind::CacheError
        )
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self.kind {
            ErrorKind::InternalServerError | ErrorKind::InternalError => ErrorSeverity::Critical,
            ErrorKind::DatabaseError | ErrorKind::ServiceUnavailable => ErrorSeverity::High,
            ErrorKind::BlockchainRpcError
            | ErrorKind::Web3PermissionValidationError
            | ErrorKind::NetworkError
            | ErrorKind::TimeoutError
            | ErrorKind::AuthenticationError
            | ErrorKind::AuthenticationTypeError => ErrorSeverity::Medium,
            ErrorKind::ValidationError
            | ErrorKind::BusinessRuleViolation
            | ErrorKind::EntityNotFound
            | ErrorKind::PermissionDenied => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }

    pub fn http_status(&self) -> u16 {
        match self.kind {
            ErrorKind::ValidationError => 400,
            ErrorKind::BusinessRuleViolation => 422,
            ErrorKind::AuthenticationError | ErrorKind::AuthenticationTypeError => 401,
            ErrorKind::AuthorizationError | ErrorKind::PermissionDenied => 403,
            ErrorKind::AggregateNotFound | ErrorKind::EntityNotFound => 404,
            ErrorKind::ConcurrencyConflict | ErrorKind::ResourceConflict => 409,
            ErrorKind::RateLimitExceeded | ErrorKind::QuotaExceeded => 429,
            ErrorKind::InternalError | ErrorKind::InternalServerError | ErrorKind::DatabaseError => 500,
            ErrorKind::ExternalServiceError | ErrorKind::CacheError | ErrorKind::BlockchainRpcError => 502,
            ErrorKind::ServiceUnavailable => 503,
            ErrorKind::TimeoutError => 504,
            _ => 500,
        }
    }
}

impl From<ValueObjectError> for AppError {
    fn from(err: ValueObjectError) -> Self {
        let kind = match &err {
            ValueObjectError::InvalidFormat(_) => ErrorKind::ValidationError,
            ValueObjectError::OutOfRange(_) => ErrorKind::ValidationError,
            ValueObjectError::Required(_) => ErrorKind::ValidationError,
            ValueObjectError::ValidationFailed(_) => ErrorKind::ValidationError,
        };
        AppError::new(kind, err.to_string())
    }
}

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

pub type AppResult<T> = std::result::Result<T, AppError>;
pub type AsyncResult<T> = std::result::Result<T, AppError>;
pub type ApiResult<T> = std::result::Result<T, AppError>;
pub type ApplicationResult<T> = std::result::Result<T, AppError>;
pub type InfrastructureResult<T> = std::result::Result<T, AppError>;
pub type EmptyResult = AppResult<()>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_set_kind() {
        assert_eq!(AppError::not_found("x").kind, ErrorKind::AggregateNotFound);
        assert_eq!(AppError::unauthorized("x").kind, ErrorKind::AuthenticationError);
        assert_eq!(AppError::forbidden("x").kind, ErrorKind::AuthorizationError);
        assert_eq!(AppError::bad_request("x").kind, ErrorKind::ValidationError);
    }

    #[test]
    fn severity_levels() {
        assert_eq!(AppError::internal_error("x").severity(), ErrorSeverity::Critical);
        assert_eq!(AppError::validation_error("x").severity(), ErrorSeverity::Low);
    }

    #[test]
    fn http_status_mapping() {
        assert_eq!(AppError::bad_request("x").http_status(), 400);
        assert_eq!(AppError::unauthorized("x").http_status(), 401);
        assert_eq!(AppError::forbidden("x").http_status(), 403);
        assert_eq!(AppError::not_found("x").http_status(), 404);
    }

    #[test]
    fn is_retryable_matches_known_kinds() {
        assert!(AppError::network_error("x").is_retryable());
        assert!(!AppError::validation_error("x").is_retryable());
    }
}
