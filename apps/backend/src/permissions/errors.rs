// Error types for the unified permission system

use thiserror::Error;
use crate::dom::values::UserId;

/// Main permission system error type
#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("Permission validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),
    
    #[error("Policy evaluation error: {0}")]
    PolicyError(#[from] PolicyError),
    
    #[error("Cache operation failed: {0}")]
    CacheError(#[from] CacheError),
    
    #[error("Audit logging failed: {0}")]
    AuditError(#[from] AuditError),
    
    #[error("Database operation failed: {0}")]
    DatabaseError(String),
    
    #[error("Permission denied: {reason}")]
    PermissionDenied { reason: String },
    
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: UserId },
    
    #[error("Resource not found: {resource}")]
    ResourceNotFound { resource: String },
    
    #[error("Invalid permission format: {permission}")]
    InvalidPermissionFormat { permission: String },
    
    #[error("Permission expired")]
    PermissionExpired,
    
    #[error("Insufficient package tier")]
    InsufficientTier,
    
    #[error("Module disabled: {module}")]
    ModuleDisabled { module: String },
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Security threat detected: {threat}")]
    SecurityThreat { threat: String },
    
    #[error("System in maintenance mode")]
    MaintenanceMode,
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Permission validation specific errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Empty permission string")]
    EmptyPermission,
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid scope: {scope}")]
    InvalidScope { scope: String },
    
    #[error("Invalid level: {level}")]
    InvalidLevel { level: String },
    
    #[error("Condition evaluation failed: {condition}")]
    ConditionFailed { condition: String },
    
    #[error("Context validation failed: {context}")]
    ContextValidationFailed { context: String },
    
    #[error("Permission chain validation failed")]
    ChainValidationFailed,
    
    #[error("Circular dependency detected")]
    CircularDependency,
    
    #[error("Maximum recursion depth exceeded")]
    MaxRecursionDepth,
    
    #[error("Invalid time range")]
    InvalidTimeRange,
    
    #[error("Conflicting permissions")]
    ConflictingPermissions,
}

/// Policy evaluation errors
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Policy not found: {policy_id}")]
    PolicyNotFound { policy_id: String },
    
    #[error("Invalid policy document: {reason}")]
    InvalidDocument { reason: String },
    
    #[error("Policy compilation failed: {reason}")]
    CompilationFailed { reason: String },
    
    #[error("Policy execution failed: {reason}")]
    ExecutionFailed { reason: String },
    
    #[error("Effect evaluation failed: {effect}")]
    EffectEvaluationFailed { effect: String },
    
    #[error("Action matching failed: {action}")]
    ActionMatchingFailed { action: String },
    
    #[error("Resource matching failed: {resource}")]
    ResourceMatchingFailed { resource: String },
    
    #[error("Condition evaluation failed: {condition}")]
    ConditionEvaluationFailed { condition: String },
    
    #[error("Principal resolution failed: {principal}")]
    PrincipalResolutionFailed { principal: String },
    
    #[error("Policy conflict detected between policies: {policies:?}")]
    PolicyConflict { policies: Vec<String> },
    
    #[error("Policy version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
    
    #[error("Policy syntax error at line {line}: {message}")]
    SyntaxError { line: usize, message: String },
}

/// Cache operation errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache connection failed: {reason}")]
    ConnectionFailed { reason: String },
    
    #[error("Cache operation timeout")]
    OperationTimeout,
    
    #[error("Cache key not found: {key}")]
    KeyNotFound { key: String },
    
    #[error("Cache serialization failed: {reason}")]
    SerializationFailed { reason: String },
    
    #[error("Cache deserialization failed: {reason}")]
    DeserializationFailed { reason: String },
    
    #[error("Cache capacity exceeded")]
    CapacityExceeded,
    
    #[error("Cache invalidation failed: {reason}")]
    InvalidationFailed { reason: String },
    
    #[error("Cache eviction failed: {reason}")]
    EvictionFailed { reason: String },
    
    #[error("Cache configuration error: {reason}")]
    ConfigurationError { reason: String },
    
    #[error("Cache backend error: {reason}")]
    BackendError { reason: String },
    
    #[error("Cache lock contention")]
    LockContention,
    
    #[error("Cache memory pressure")]
    MemoryPressure,
}

/// Audit logging errors
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Audit log write failed: {reason}")]
    WriteFailed { reason: String },
    
    #[error("Audit log read failed: {reason}")]
    ReadFailed { reason: String },
    
    #[error("Audit log format invalid: {reason}")]
    InvalidFormat { reason: String },
    
    #[error("Audit log storage full")]
    StorageFull,
    
    #[error("Audit log retention policy violation: {reason}")]
    RetentionViolation { reason: String },
    
    #[error("Audit log encryption failed: {reason}")]
    EncryptionFailed { reason: String },
    
    #[error("Audit log decryption failed: {reason}")]
    DecryptionFailed { reason: String },
    
    #[error("Audit log integrity check failed")]
    IntegrityCheckFailed,
    
    #[error("Audit log export failed: {reason}")]
    ExportFailed { reason: String },
    
    #[error("Audit log import failed: {reason}")]
    ImportFailed { reason: String },
    
    #[error("Audit log archival failed: {reason}")]
    ArchivalFailed { reason: String },
    
    #[error("Audit log compliance violation: {reason}")]
    ComplianceViolation { reason: String },
}

/// Result type aliases for convenience
pub type PermissionResult<T> = Result<T, PermissionError>;
pub type ValidationResult<T> = Result<T, ValidationError>;
pub type PolicyResult<T> = Result<T, PolicyError>;
pub type CacheResult<T> = Result<T, CacheError>;
pub type AuditResult<T> = Result<T, AuditError>;

// Conversion implementations for common error types
impl From<serde_json::Error> for PermissionError {
    fn from(err: serde_json::Error) -> Self {
        PermissionError::SerializationError(err.to_string())
    }
}

impl From<diesel::result::Error> for PermissionError {
    fn from(err: diesel::result::Error) -> Self {
        PermissionError::DatabaseError(err.to_string())
    }
}

impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        CacheError::BackendError { reason: err.to_string() }
    }
}

impl From<tokio::time::error::Elapsed> for PermissionError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        PermissionError::TimeoutError(err.to_string())
    }
}

impl From<std::io::Error> for AuditError {
    fn from(err: std::io::Error) -> Self {
        AuditError::WriteFailed {
            reason: err.to_string(),
        }
    }
}

// Error helper functions
impl PermissionError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            PermissionError::NetworkError(_)
                | PermissionError::TimeoutError(_)
                | PermissionError::CacheError(CacheError::ConnectionFailed { .. })
                | PermissionError::CacheError(CacheError::OperationTimeout)
                | PermissionError::DatabaseError(_)
        )
    }
    
    /// Check if error is security-related
    pub fn is_security_error(&self) -> bool {
        matches!(
            self,
            PermissionError::PermissionDenied { .. }
                | PermissionError::SecurityThreat { .. }
                | PermissionError::InsufficientTier
                | PermissionError::PermissionExpired
                | PermissionError::RateLimitExceeded
        )
    }
    
    /// Check if error should trigger alert
    pub fn should_alert(&self) -> bool {
        matches!(
            self,
            PermissionError::SecurityThreat { .. }
                | PermissionError::AuditError(AuditError::IntegrityCheckFailed)
                | PermissionError::AuditError(AuditError::ComplianceViolation { .. })
        )
    }
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            PermissionError::SecurityThreat { .. } => ErrorSeverity::Critical,
            PermissionError::AuditError(AuditError::IntegrityCheckFailed) => ErrorSeverity::Critical,
            PermissionError::AuditError(AuditError::ComplianceViolation { .. }) => ErrorSeverity::Critical,
            PermissionError::PermissionDenied { .. } => ErrorSeverity::High,
            PermissionError::InsufficientTier => ErrorSeverity::Medium,
            PermissionError::RateLimitExceeded => ErrorSeverity::Medium,
            PermissionError::MaintenanceMode => ErrorSeverity::Low,
            PermissionError::NetworkError(_) => ErrorSeverity::Medium,
            PermissionError::TimeoutError(_) => ErrorSeverity::Medium,
            _ => ErrorSeverity::Low,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

// Error context helpers
impl ValidationError {
    pub fn with_context(self, context: &str) -> Self {
        match self {
            ValidationError::InvalidFormat(msg) => {
                ValidationError::InvalidFormat(format!("{} (context: {})", msg, context))
            }
            ValidationError::ConditionFailed { condition } => {
                ValidationError::ConditionFailed {
                    condition: format!("{} (context: {})", condition, context),
                }
            }
            other => other,
        }
    }
}

impl PolicyError {
    pub fn with_policy_id(self, policy_id: &str) -> Self {
        match self {
            PolicyError::InvalidDocument { reason } => PolicyError::InvalidDocument {
                reason: format!("{} (policy: {})", reason, policy_id),
            },
            PolicyError::CompilationFailed { reason } => PolicyError::CompilationFailed {
                reason: format!("{} (policy: {})", reason, policy_id),
            },
            PolicyError::ExecutionFailed { reason } => PolicyError::ExecutionFailed {
                reason: format!("{} (policy: {})", reason, policy_id),
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Low < ErrorSeverity::Medium);
        assert!(ErrorSeverity::Medium < ErrorSeverity::High);
        assert!(ErrorSeverity::High < ErrorSeverity::Critical);
    }
    
    #[test]
    fn test_permission_error_retryable() {
        let network_error = PermissionError::NetworkError("connection failed".to_string());
        assert!(network_error.is_retryable());
        
        let permission_denied = PermissionError::PermissionDenied {
            reason: "insufficient privileges".to_string(),
        };
        assert!(!permission_denied.is_retryable());
    }
    
    #[test]
    fn test_permission_error_security() {
        let security_threat = PermissionError::SecurityThreat {
            threat: "malicious request".to_string(),
        };
        assert!(security_threat.is_security_error());
        
        let config_error = PermissionError::ConfigurationError("invalid config".to_string());
        assert!(!config_error.is_security_error());
    }
    
    #[test]
    fn test_permission_error_severity() {
        let security_threat = PermissionError::SecurityThreat {
            threat: "malicious request".to_string(),
        };
        assert_eq!(security_threat.severity(), ErrorSeverity::Critical);
        
        let network_error = PermissionError::NetworkError("connection failed".to_string());
        assert_eq!(network_error.severity(), ErrorSeverity::Medium);
    }
    
    #[test]
    fn test_validation_error_context() {
        let error = ValidationError::InvalidFormat("missing colon".to_string());
        let with_context = error.with_context("user:read validation");
        
        match with_context {
            ValidationError::InvalidFormat(msg) => {
                assert!(msg.contains("context: user:read validation"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }
    
    #[test]
    fn test_policy_error_context() {
        let error = PolicyError::CompilationFailed {
            reason: "syntax error".to_string(),
        };
        let with_context = error.with_policy_id("test-policy-123");
        
        match with_context {
            PolicyError::CompilationFailed { reason } => {
                assert!(reason.contains("policy: test-policy-123"));
            }
            _ => panic!("Expected CompilationFailed error"),
        }
    }
    
    #[test]
    fn test_error_conversions() {
        let json_error = serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "test error",
        ));
        let perm_error: PermissionError = json_error.into();
        
        match perm_error {
            PermissionError::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }
    }
}