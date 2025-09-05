use std::fmt;
use crate::domain::shared_kernel::DomainError;

/// Application layer error types
/// These wrap domain errors and add application-specific concerns
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    
    #[error("Validation failed: {field} - {message}")]
    Validation { field: String, message: String },
    
    #[error("Authorization failed: {action} not allowed for user")]
    Authorization { action: String },
    
    #[error("Resource not found: {resource_type} with id {id}")]
    NotFound { resource_type: String, id: String },
    
    #[error("Conflict: {resource} already exists")]
    Conflict { resource: String },
    
    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },
    
    #[error("Infrastructure error: {message}")]
    Infrastructure { message: String },
    
    #[error("Concurrency error: {message}")]
    Concurrency { message: String },
    
    #[error("Business rule violation: {rule}")]
    BusinessRule { rule: String },
}

impl ApplicationError {
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
    
    pub fn authorization(action: impl Into<String>) -> Self {
        Self::Authorization {
            action: action.into(),
        }
    }
    
    pub fn not_found(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }
    
    pub fn conflict(resource: impl Into<String>) -> Self {
        Self::Conflict {
            resource: resource.into(),
        }
    }
    
    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }
    
    pub fn infrastructure(message: impl Into<String>) -> Self {
        Self::Infrastructure {
            message: message.into(),
        }
    }
    
    pub fn concurrency(message: impl Into<String>) -> Self {
        Self::Concurrency {
            message: message.into(),
        }
    }
    
    pub fn business_rule(rule: impl Into<String>) -> Self {
        Self::BusinessRule {
            rule: rule.into(),
        }
    }
}

/// Result type for application operations
pub type ApplicationResult<T> = Result<T, ApplicationError>;