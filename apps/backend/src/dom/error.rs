// Domain error types with enhanced context

use thiserror::Error;
// Removed Role import as roles are no longer used
use crate::core::errors::{AppError, ErrorKind, ErrorContextBuilder};

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Cannot upgrade from {current} to {target}")]
    PackageTierUpgradeNotAllowed { current: String, target: String },
    
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

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        let (kind, message, context) = match &err {
            DomainError::PackageTierUpgradeNotAllowed { current, target } => (
                ErrorKind::BusinessRuleViolation,
                format!("Cannot upgrade from {} to {}", current, target),
                ErrorContextBuilder::new("package_tier_upgrade", "domain")
                    .metadata("current_tier", current.clone())
                    .metadata("target_tier", target.clone())
                    .build()
            ),
            DomainError::InvalidEmail(email) => (
                ErrorKind::ValidationError,
                format!("Invalid email format: {}", email),
                ErrorContextBuilder::new("email_validation", "domain")
                    .metadata("email_provided", email.clone())
                    .build()
            ),
            DomainError::PermissionDenied(resource) => (
                ErrorKind::AuthorizationError,
                format!("Access denied to resource: {}", resource),
                ErrorContextBuilder::new("permission_check", "domain")
                    .metadata("resource", resource.clone())
                    .build()
            ),
            DomainError::ModuleNotFound(module_id) => (
                ErrorKind::AggregateNotFound,
                format!("Module not found: {}", module_id),
                ErrorContextBuilder::new("module_lookup", "domain")
                    .metadata("module_id", module_id.clone())
                    .build()
            ),
            DomainError::AssignmentNotFound(assignment_id) => (
                ErrorKind::AggregateNotFound,
                format!("Assignment not found: {}", assignment_id),
                ErrorContextBuilder::new("assignment_lookup", "domain")
                    .metadata("assignment_id", assignment_id.clone())
                    .build()
            ),
            DomainError::ApiKeyNotFound(key_id) => (
                ErrorKind::AggregateNotFound,
                format!("API key not found: {}", key_id),
                ErrorContextBuilder::new("api_key_lookup", "domain")
                    .metadata("key_id", key_id.clone())
                    .build()
            ),
            DomainError::DatabaseError(details) => (
                ErrorKind::DatabaseError,
                format!("Database operation failed: {}", details),
                ErrorContextBuilder::new("database_operation", "domain")
                    .metadata("operation_details", details.clone())
                    .build()
            ),
            DomainError::ValidationError(details) => (
                ErrorKind::ValidationError,
                format!("Domain validation failed: {}", details),
                ErrorContextBuilder::new("domain_validation", "domain")
                    .metadata("validation_details", details.clone())
                    .build()
            ),
            DomainError::InternalError(details) => (
                ErrorKind::InternalError,
                format!("Domain internal error: {}", details),
                ErrorContextBuilder::new("internal_operation", "domain")
                    .metadata("error_details", details.clone())
                    .build()
            ),
        };
        
        AppError::new(kind, message).with_context(context)
    }
}

/// Enhanced IAM-specific errors with better context
#[derive(Debug, Error)]
pub enum IamError {
    #[error("Role '{role_name}' already exists")]
    RoleAlreadyExists { role_name: String },
    
    #[error("Role '{role_name}' not found")] 
    RoleNotFound { role_name: String },
    
    #[error("Permission '{permission}' not found")]
    PermissionNotFound { permission: String },
    
    #[error("User '{user_id}' already has role '{role_name}'")]
    UserAlreadyHasRole { user_id: String, role_name: String },
    
    #[error("Cannot remove last admin role from user '{user_id}'")]
    CannotRemoveLastAdmin { user_id: String },
    
    #[error("Circular dependency detected in role hierarchy")]
    CircularRoleDependency { roles: Vec<String> },
    
    #[error("Permission evaluation failed: {details}")]
    PermissionEvaluationFailed { details: String },
}

impl From<IamError> for AppError {
    fn from(err: IamError) -> Self {
        let (kind, message, context) = match &err {
            IamError::RoleAlreadyExists { role_name } => (
                ErrorKind::BusinessRuleViolation,
                format!("Role '{}' already exists", role_name),
                ErrorContextBuilder::new("create_role", "iam")
                    .metadata("role_name", role_name.clone())
                    .metadata("conflict_type", "duplicate_role")
                    .build()
            ),
            IamError::RoleNotFound { role_name } => (
                ErrorKind::AggregateNotFound,
                format!("Role '{}' not found", role_name),
                ErrorContextBuilder::new("find_role", "iam")
                    .metadata("role_name", role_name.clone())
                    .build()
            ),
            IamError::PermissionNotFound { permission } => (
                ErrorKind::AggregateNotFound,
                format!("Permission '{}' not found", permission),
                ErrorContextBuilder::new("find_permission", "iam")
                    .metadata("permission", permission.clone())
                    .build()
            ),
            IamError::UserAlreadyHasRole { user_id, role_name } => (
                ErrorKind::BusinessRuleViolation,
                format!("User '{}' already has role '{}'", user_id, role_name),
                ErrorContextBuilder::new("assign_role", "iam")
                    .metadata("user_id", user_id.clone())
                    .metadata("role_name", role_name.clone())
                    .build()
            ),
            IamError::CannotRemoveLastAdmin { user_id } => (
                ErrorKind::BusinessRuleViolation,
                format!("Cannot remove last admin role from user '{}'", user_id),
                ErrorContextBuilder::new("remove_role", "iam")
                    .metadata("user_id", user_id.clone())
                    .metadata("safety_violation", "last_admin_protection")
                    .build()
            ),
            IamError::CircularRoleDependency { roles } => (
                ErrorKind::BusinessRuleViolation,
                format!("Circular dependency detected in roles: {:?}", roles),
                ErrorContextBuilder::new("role_hierarchy_check", "iam")
                    .metadata("involved_roles", format!("{:?}", roles))
                    .build()
            ),
            IamError::PermissionEvaluationFailed { details } => (
                ErrorKind::InternalError,
                format!("Permission evaluation failed: {}", details),
                ErrorContextBuilder::new("permission_evaluation", "iam")
                    .metadata("evaluation_details", details.clone())
                    .build()
            ),
        };
        
        AppError::new(kind, message).with_context(context)
    }
}