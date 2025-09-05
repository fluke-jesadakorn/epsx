// Session Collection Error
// Defines errors that can occur when working with session collections

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Errors that can occur during session collection operations
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum SessionCollectionError {
    #[error("Session collection is full (max capacity: {max_capacity})")]
    CollectionFull { max_capacity: usize },
    
    #[error("Session not found in collection: {session_id}")]
    SessionNotFound { session_id: String },
    
    #[error("Duplicate session in collection: {session_id}")]
    DuplicateSession { session_id: String },
    
    #[error("Session collection is empty")]
    EmptyCollection,
    
    #[error("Invalid session state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },
    
    #[error("Session collection operation failed: {operation}")]
    OperationFailed { operation: String },
    
    #[error("Concurrent modification detected during operation: {operation}")]
    ConcurrentModification { operation: String },
    
    #[error("Session collection validation failed: {reason}")]
    ValidationFailed { reason: String },
    
    #[error("Session limit exceeded: {current_count} sessions (max: {max_allowed})")]
    SessionLimitExceeded { current_count: usize, max_allowed: usize },
    
    #[error("Session collection corrupted: {details}")]
    CollectionCorrupted { details: String },
    
    #[error("Permission denied for session operation: {operation} on session {session_id}")]
    PermissionDenied { operation: String, session_id: String },
    
    #[error("Session collection locked by another operation")]
    CollectionLocked,
    
    #[error("Session collection backup/restore failed: {reason}")]
    BackupRestoreFailed { reason: String },
    
    #[error("Invalid session metadata: {field} = {value}")]
    InvalidSessionMetadata { field: String, value: String },
    
    #[error("Session collection consistency check failed: {details}")]
    ConsistencyCheckFailed { details: String },
}

impl SessionCollectionError {
    /// Create a collection full error
    pub fn collection_full(max_capacity: usize) -> Self {
        Self::CollectionFull { max_capacity }
    }
    
    /// Create a session not found error
    pub fn session_not_found(session_id: impl Into<String>) -> Self {
        Self::SessionNotFound { session_id: session_id.into() }
    }
    
    /// Create a duplicate session error
    pub fn duplicate_session(session_id: impl Into<String>) -> Self {
        Self::DuplicateSession { session_id: session_id.into() }
    }
    
    /// Create an invalid state transition error
    pub fn invalid_state_transition(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::InvalidStateTransition { from: from.into(), to: to.into() }
    }
    
    /// Create an operation failed error
    pub fn operation_failed(operation: impl Into<String>) -> Self {
        Self::OperationFailed { operation: operation.into() }
    }
    
    /// Create a concurrent modification error
    pub fn concurrent_modification(operation: impl Into<String>) -> Self {
        Self::ConcurrentModification { operation: operation.into() }
    }
    
    /// Create a validation failed error
    pub fn validation_failed(reason: impl Into<String>) -> Self {
        Self::ValidationFailed { reason: reason.into() }
    }
    
    /// Create a session limit exceeded error
    pub fn session_limit_exceeded(current_count: usize, max_allowed: usize) -> Self {
        Self::SessionLimitExceeded { current_count, max_allowed }
    }
    
    /// Create a collection corrupted error
    pub fn collection_corrupted(details: impl Into<String>) -> Self {
        Self::CollectionCorrupted { details: details.into() }
    }
    
    /// Create a permission denied error
    pub fn permission_denied(operation: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self::PermissionDenied { 
            operation: operation.into(), 
            session_id: session_id.into() 
        }
    }
    
    /// Create a backup/restore failed error
    pub fn backup_restore_failed(reason: impl Into<String>) -> Self {
        Self::BackupRestoreFailed { reason: reason.into() }
    }
    
    /// Create an invalid session metadata error
    pub fn invalid_session_metadata(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::InvalidSessionMetadata { 
            field: field.into(), 
            value: value.into() 
        }
    }
    
    /// Create a consistency check failed error
    pub fn consistency_check_failed(details: impl Into<String>) -> Self {
        Self::ConsistencyCheckFailed { details: details.into() }
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::CollectionFull { .. } => false,
            Self::SessionNotFound { .. } => false,
            Self::DuplicateSession { .. } => true,
            Self::EmptyCollection => false,
            Self::InvalidStateTransition { .. } => false,
            Self::OperationFailed { .. } => true,
            Self::ConcurrentModification { .. } => true,
            Self::ValidationFailed { .. } => false,
            Self::SessionLimitExceeded { .. } => false,
            Self::CollectionCorrupted { .. } => false,
            Self::PermissionDenied { .. } => false,
            Self::CollectionLocked => true,
            Self::BackupRestoreFailed { .. } => true,
            Self::InvalidSessionMetadata { .. } => false,
            Self::ConsistencyCheckFailed { .. } => false,
        }
    }
    
    /// Check if this error requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        match self {
            Self::CollectionCorrupted { .. } => true,
            Self::ConsistencyCheckFailed { .. } => true,
            Self::SessionLimitExceeded { .. } => true,
            _ => false,
        }
    }
    
    /// Get error category for logging/monitoring
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::CollectionFull { .. } => ErrorCategory::Capacity,
            Self::SessionNotFound { .. } => ErrorCategory::NotFound,
            Self::DuplicateSession { .. } => ErrorCategory::Conflict,
            Self::EmptyCollection => ErrorCategory::State,
            Self::InvalidStateTransition { .. } => ErrorCategory::State,
            Self::OperationFailed { .. } => ErrorCategory::Operation,
            Self::ConcurrentModification { .. } => ErrorCategory::Concurrency,
            Self::ValidationFailed { .. } => ErrorCategory::Validation,
            Self::SessionLimitExceeded { .. } => ErrorCategory::Capacity,
            Self::CollectionCorrupted { .. } => ErrorCategory::Corruption,
            Self::PermissionDenied { .. } => ErrorCategory::Security,
            Self::CollectionLocked => ErrorCategory::Concurrency,
            Self::BackupRestoreFailed { .. } => ErrorCategory::Operation,
            Self::InvalidSessionMetadata { .. } => ErrorCategory::Validation,
            Self::ConsistencyCheckFailed { .. } => ErrorCategory::Corruption,
        }
    }
}

/// Categories of session collection errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    Capacity,
    NotFound,
    Conflict,
    State,
    Operation,
    Concurrency,
    Validation,
    Corruption,
    Security,
}

impl ErrorCategory {
    /// Get error priority level
    pub fn priority_level(&self) -> u8 {
        match self {
            ErrorCategory::Corruption => 5,  // Highest priority
            ErrorCategory::Security => 4,
            ErrorCategory::Capacity => 3,
            ErrorCategory::State => 2,
            ErrorCategory::Validation => 2,
            ErrorCategory::Concurrency => 1,
            ErrorCategory::Operation => 1,
            ErrorCategory::NotFound => 1,
            ErrorCategory::Conflict => 1,
        }
    }
}

/// Session collection error with additional context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualSessionCollectionError {
    pub error: SessionCollectionError,
    pub context: ErrorContext,
    pub occurred_at: DateTime<Utc>,
    pub error_id: String,
}

/// Additional context for session collection errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub collection_size: Option<usize>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ContextualSessionCollectionError {
    /// Create a new contextual error
    pub fn new(error: SessionCollectionError, context: ErrorContext) -> Self {
        Self {
            error,
            context,
            occurred_at: Utc::now(),
            error_id: uuid::Uuid::new_v4().to_string(),
        }
    }
    
    /// Create error context builder
    pub fn with_context(error: SessionCollectionError) -> ErrorContextBuilder {
        ErrorContextBuilder::new(error)
    }
}

/// Builder for error context
pub struct ErrorContextBuilder {
    error: SessionCollectionError,
    context: ErrorContext,
}

impl ErrorContextBuilder {
    fn new(error: SessionCollectionError) -> Self {
        Self {
            error,
            context: ErrorContext {
                operation: String::new(),
                user_id: None,
                session_id: None,
                collection_size: None,
                additional_info: std::collections::HashMap::new(),
            },
        }
    }
    
    pub fn operation(mut self, operation: impl Into<String>) -> Self {
        self.context.operation = operation.into();
        self
    }
    
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.context.user_id = Some(user_id.into());
        self
    }
    
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.context.session_id = Some(session_id.into());
        self
    }
    
    pub fn collection_size(mut self, size: usize) -> Self {
        self.context.collection_size = Some(size);
        self
    }
    
    pub fn additional_info(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.additional_info.insert(key.into(), value.into());
        self
    }
    
    pub fn build(self) -> ContextualSessionCollectionError {
        ContextualSessionCollectionError::new(self.error, self.context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_collection_error_creation() {
        let error = SessionCollectionError::collection_full(100);
        assert!(!error.is_recoverable());
        assert_eq!(error.category(), ErrorCategory::Capacity);
    }
    
    #[test]
    fn test_contextual_error() {
        let error = SessionCollectionError::session_not_found("session123");
        let contextual = ContextualSessionCollectionError::with_context(error)
            .operation("get_session")
            .user_id("user456")
            .session_id("session123")
            .collection_size(5)
            .build();
        
        assert!(contextual.error_id.len() > 0);
        assert_eq!(contextual.context.operation, "get_session");
        assert_eq!(contextual.context.user_id, Some("user456".to_string()));
    }
    
    #[test]
    fn test_error_priority_levels() {
        assert_eq!(ErrorCategory::Corruption.priority_level(), 5);
        assert_eq!(ErrorCategory::Security.priority_level(), 4);
        assert_eq!(ErrorCategory::NotFound.priority_level(), 1);
    }
}