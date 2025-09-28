use crate::core::errors::AppError;
use crate::domain::user_management::value_objects::wallet_address::WalletAddressError;

/// Application layer error types
/// These wrap domain errors and add application-specific concerns
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] AppError),
    
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
    
    #[error("Feature not implemented: {feature}")]
    NotImplemented { feature: String },
    
    #[error("Business logic error: {message}")]
    BusinessLogic { message: String },
    
    #[error("Security error: {message}")]
    Security { message: String },
    
    #[error("Wallet address error: {0}")]
    WalletAddress(#[from] WalletAddressError),
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
    
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::NotImplemented {
            feature: feature.into(),
        }
    }
    
    pub fn business_logic(message: impl Into<String>) -> Self {
        Self::BusinessLogic {
            message: message.into(),
        }
    }
    
    pub fn security_error(message: impl Into<String>) -> Self {
        Self::Security {
            message: message.into(),
        }
    }

    // Aliases for backward compatibility
    #[allow(non_snake_case)]
    pub fn InfrastructureError(message: impl Into<String>) -> Self {
        Self::infrastructure(message)
    }
    
    #[allow(non_snake_case)]
    pub fn SecurityError(message: impl Into<String>) -> Self {
        Self::authorization(message.into())
    }
    
    #[allow(non_snake_case)]
    pub fn ValidationError(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::validation(field, message)
    }
    
    #[allow(non_snake_case)]
    pub fn AuthorizationError(action: impl Into<String>) -> Self {
        Self::authorization(action)
    }
    
    #[allow(non_snake_case)]
    pub fn NotImplemented(feature: impl Into<String>) -> Self {
        Self::not_implemented(feature)
    }
    
    #[allow(non_snake_case)]
    pub fn BusinessLogicError(message: impl Into<String>) -> Self {
        Self::business_logic(message)
    }
    
    #[allow(non_snake_case)]
    pub fn DomainError(message: impl Into<String>) -> Self {
        Self::Domain(crate::core::errors::AppError::business_rule_violation(message.into()))
    }
}

/// Result type for application operations
pub type ApplicationResult<T> = Result<T, ApplicationError>;