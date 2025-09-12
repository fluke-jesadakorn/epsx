use std::fmt;

/// Base domain error type
/// All domain errors should implement this trait
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Entity not found: {entity_type} with id {id}")]
    EntityNotFound {
        entity_type: &'static str,
        id: String,
    },
    
    #[error("Business rule violation: {rule}")]
    BusinessRuleViolation { rule: String },
    
    #[error("Concurrency conflict: version {expected} expected, got {actual}")]
    ConcurrencyConflict { expected: u64, actual: u64 },
    
    #[error("Invalid operation: {operation} on {entity_type}")]
    InvalidOperation {
        operation: String,
        entity_type: &'static str,
    },
    
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },
    
    #[error("Permission denied: {action} on {resource}")]
    PermissionDenied { action: String, resource: String },
    
    #[error("Resource conflict: {resource} - {reason}")]
    ResourceConflict { resource: String, reason: String },
    
    #[error("Invariant violation: {invariant}")]
    InvariantViolation { invariant: String },
    
    #[error("Infrastructure error: {message}")]
    Infrastructure { message: String },
}

impl DomainError {
    pub fn entity_not_found(entity_type: &'static str, id: impl fmt::Display) -> Self {
        Self::EntityNotFound {
            entity_type,
            id: id.to_string(),
        }
    }
    
    pub fn business_rule_violation(rule: impl Into<String>) -> Self {
        Self::BusinessRuleViolation { rule: rule.into() }
    }
    
    pub fn concurrency_conflict(expected: u64, actual: u64) -> Self {
        Self::ConcurrencyConflict { expected, actual }
    }
    
    pub fn invalid_operation(operation: impl Into<String>, entity_type: &'static str) -> Self {
        Self::InvalidOperation {
            operation: operation.into(),
            entity_type,
        }
    }
    
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
    
    pub fn permission_denied(action: impl Into<String>, resource: impl Into<String>) -> Self {
        Self::PermissionDenied {
            action: action.into(),
            resource: resource.into(),
        }
    }
    
    pub fn resource_conflict(resource: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ResourceConflict {
            resource: resource.into(),
            reason: reason.into(),
        }
    }
    
    pub fn invariant_violation(invariant: impl Into<String>) -> Self {
        Self::InvariantViolation {
            invariant: invariant.into(),
        }
    }
    
    pub fn infrastructure(message: impl Into<String>) -> Self {
        Self::Infrastructure {
            message: message.into(),
        }
    }
}

/// Result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

/// Convert value object errors to domain errors
impl From<super::value_object::ValueObjectError> for DomainError {
    fn from(error: super::value_object::ValueObjectError) -> Self {
        match error {
            super::value_object::ValueObjectError::InvalidFormat(msg) => {
                Self::validation_error("value_object", msg)
            }
            super::value_object::ValueObjectError::OutOfRange(msg) => {
                Self::validation_error("value_object", msg)
            }
            super::value_object::ValueObjectError::Required(msg) => {
                Self::validation_error("value_object", msg)
            }
            super::value_object::ValueObjectError::ValidationFailed(msg) => {
                Self::validation_error("value_object", msg)
            }
        }
    }
}