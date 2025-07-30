// Domain error types

use thiserror::Error;
use crate::dom::values::Role;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Cannot upgrade from {current:?} to {target:?}")]
    RoleUpgradeNotAllowed { current: Role, target: Role },
    
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    
    #[error("Assignment not found: {0}")]
    AssignmentNotFound(String),
    
    #[error("API key not found: {0}")]
    ApiKeyNotFound(String),
    
    #[error("Database operation failed: {0}")]
    DatabaseError(String),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}