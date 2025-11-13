// Unified Application Error System
// Comprehensive error handling for Web3-first EPSX platform

use std::collections::HashMap;
use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unified application error type for the entire EPSX platform
/// Provides structured error handling with Web3-specific support
#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize)]
#[serde(tag = "error_type", content = "details")]
pub enum AppError {
    // Domain-level errors (business logic)
    #[error("Business rule violation: {rule}")]
    BusinessRuleViolation { rule: String, context: ErrorContext },
    
    #[error("Entity not found: {entity_type} with identifier {id}")]
    EntityNotFound { 
        entity_type: String, 
        id: String,
        context: ErrorContext,
    },
    
    #[error("Validation error: {field} - {message}")]
    ValidationError {
        field: String,
        message: String,
        validation_type: ValidationType,
        context: ErrorContext,
    },
    
    #[error("Permission denied: {action} on {resource}")]
    PermissionDenied {
        action: String,
        resource: String,
        required_permissions: Vec<String>,
        context: ErrorContext,
    },
    
    #[error("Resource conflict: {resource} - {reason}")]
    ResourceConflict {
        resource: String,
        reason: String,
        context: ErrorContext,
    },
    
    // Web3-specific errors (blockchain and wallet operations)
    #[error("Blockchain RPC error: {message} on chain {chain_id}")]
    BlockchainRpcError {
        chain_id: u64,
        rpc_endpoint: String,
        message: String,
        error_code: Option<i32>,
        retry_count: u32,
        context: ErrorContext,
    },
    
    #[error("Wallet validation error: {wallet_address} - {reason}")]
    WalletValidationError {
        wallet_address: String,
        reason: String,
        validation_type: WalletValidationType,
        context: ErrorContext,
    },
    
    #[error("Web3 permission validation failed: {permission} for wallet {wallet_address}")]
    Web3PermissionValidationError {
        wallet_address: String,
        permission: String,
        permission_type: Web3PermissionType,
        blockchain_data: Option<String>,
        chain_id: u64,
        context: ErrorContext,
    },
    
    #[error("Smart contract interaction error: {contract_address} on chain {chain_id}")]
    SmartContractError {
        contract_address: String,
        chain_id: u64,
        function_name: Option<String>,
        transaction_hash: Option<String>,
        gas_used: Option<u64>,
        error_message: String,
        context: ErrorContext,
    },
    
    #[error("Cross-chain operation error: source chain {source_chain_id} to target chain {target_chain_id}")]
    CrossChainError {
        source_chain_id: u64,
        target_chain_id: u64,
        bridge_used: Option<String>,
        operation_type: String,
        error_message: String,
        context: ErrorContext,
    },
    
    // Infrastructure-level errors (database, network, etc.)
    #[error("Database error: {message}")]
    DatabaseError {
        message: String,
        error_code: Option<String>,
        table: Option<String>,
        query_type: Option<DatabaseOperation>,
        context: ErrorContext,
    },
    
    #[error("Cache error: {message}")]
    CacheError {
        message: String,
        cache_type: CacheType,
        key: Option<String>,
        operation: CacheOperation,
        context: ErrorContext,
    },
    
    #[error("Network error: {message}")]
    NetworkError {
        message: String,
        url: Option<String>,
        status_code: Option<u16>,
        timeout_ms: Option<u64>,
        retry_count: u32,
        context: ErrorContext,
    },
    
    #[error("Configuration error: {parameter} - {message}")]
    ConfigurationError {
        parameter: String,
        message: String,
        expected_type: Option<String>,
        context: ErrorContext,
    },
    
    #[error("Service unavailable: {service_name} - {reason}")]
    ServiceUnavailable {
        service_name: String,
        reason: String,
        health_status: ServiceHealthStatus,
        context: ErrorContext,
    },
    
    // Application-level errors (API, authentication, etc.)
    #[error("Authentication error: {message}")]
    AuthenticationError {
        message: String,
        auth_type: AuthenticationType,
        provider: Option<String>,
        context: ErrorContext,
    },
    
    #[error("Authorization error: insufficient privileges for {action} on {resource}")]
    AuthorizationError {
        action: String,
        resource: String,
        wallet_address: Option<String>,
        required_role: Option<String>,
        context: ErrorContext,
    },
    
    #[error("Rate limit exceeded: {limit_type} - {current}/{max} requests")]
    RateLimitExceeded {
        limit_type: String,
        current: u64,
        max: u64,
        reset_time: DateTime<Utc>,
        context: ErrorContext,
    },
    
    #[error("Concurrency error: version {expected} expected, got {actual}")]
    ConcurrencyConflict {
        expected: u64,
        actual: u64,
        entity_id: String,
        context: ErrorContext,
    },
    
    #[error("External service error: {service} - {message}")]
    ExternalServiceError {
        service: String,
        message: String,
        error_code: Option<String>,
        response_body: Option<String>,
        context: ErrorContext,
    },
    
    // System-level errors (internal server errors, panics, etc.)
    #[error("Internal server error: {message}")]
    InternalServerError {
        message: String,
        error_id: Uuid,
        stack_trace: Option<String>,
        context: ErrorContext,
    },
    
    #[error("Timeout error: operation {operation} exceeded {timeout_ms}ms")]
    TimeoutError {
        operation: String,
        timeout_ms: u64,
        elapsed_ms: u64,
        context: ErrorContext,
    },
}

/// Error context provides additional information about where and when the error occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Unique error ID for tracing
    pub error_id: Uuid,
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// Service/component where error occurred
    pub component: String,
    /// Operation being performed when error occurred
    pub operation: String,
    /// User/wallet address if available
    pub user_context: Option<String>,
    /// Request ID for tracing across services
    pub request_id: Option<String>,
    /// Additional metadata specific to the error
    pub metadata: HashMap<String, String>,
    /// Chain ID for Web3 operations
    pub chain_id: Option<u64>,
    /// Environment (dev, staging, prod)
    pub environment: Option<String>,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            error_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            component: "unknown".to_string(),
            operation: "unknown".to_string(),
            user_context: None,
            request_id: None,
            metadata: HashMap::new(),
            chain_id: None,
            environment: std::env::var("ENVIRONMENT").ok(),
        }
    }
}

impl ErrorContext {
    pub fn new(component: &str, operation: &str) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            component: component.to_string(),
            operation: operation.to_string(),
            user_context: None,
            request_id: None,
            metadata: HashMap::new(),
            chain_id: None,
            environment: std::env::var("ENVIRONMENT").ok(),
        }
    }
    
    pub fn with_user(mut self, user_context: String) -> Self {
        self.user_context = Some(user_context);
        self
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
    
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    Required,
    Format,
    Range,
    Pattern,
    Length,
    Custom(String),
    Web3Address,
    Web3Signature,
    ChainId,
    ContractAddress,
}

/// Wallet validation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletValidationType {
    AddressFormat,
    SignatureVerification,
    NonceValidation,
    ChainMismatch,
    InsufficientBalance,
    ContractInteraction,
}

/// Web3 permission types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Web3PermissionType {
    Manual,
    NftGated,
    TokenGated,
    DaoGovernance,
    CrossChain,
}

/// Database operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseOperation {
    Select,
    Insert,
    Update,
    Delete,
    Transaction,
    Migration,
}

/// Cache types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheType {
    Redis,
    Memory,
    Distributed,
}

/// Cache operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheOperation {
    Get,
    Set,
    Delete,
    Expire,
    Flush,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationType {
    Web3Wallet,
    ApiKey,
    // Legacy auth types removed - using Web3 wallet-first system
    // JWT,
    // OAuth,
    // Basic,
}

impl AppError {
    // Convenience constructors for common error types
    
    pub fn business_rule_violation(rule: impl Into<String>) -> Self {
        Self::BusinessRuleViolation {
            rule: rule.into(),
            context: ErrorContext::default(),
        }
    }
    
    pub fn entity_not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::EntityNotFound {
            entity_type: entity_type.into(),
            id: id.into(),
            context: ErrorContext::default(),
        }
    }
    
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
            validation_type: ValidationType::Custom("generic".to_string()),
            context: ErrorContext::default(),
        }
    }
    
    pub fn web3_validation_error(
        field: impl Into<String>,
        message: impl Into<String>,
        validation_type: ValidationType,
    ) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
            validation_type,
            context: ErrorContext::default(),
        }
    }
    
    pub fn permission_denied(action: impl Into<String>, resource: impl Into<String>) -> Self {
        Self::PermissionDenied {
            action: action.into(),
            resource: resource.into(),
            required_permissions: Vec::new(),
            context: ErrorContext::default(),
        }
    }
    
    pub fn blockchain_rpc_error(
        chain_id: u64,
        rpc_endpoint: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::BlockchainRpcError {
            chain_id,
            rpc_endpoint: rpc_endpoint.into(),
            message: message.into(),
            error_code: None,
            retry_count: 0,
            context: ErrorContext::default(),
        }
    }
    
    pub fn wallet_validation_error(
        wallet_address: impl Into<String>,
        reason: impl Into<String>,
        validation_type: WalletValidationType,
    ) -> Self {
        Self::WalletValidationError {
            wallet_address: wallet_address.into(),
            reason: reason.into(),
            validation_type,
            context: ErrorContext::default(),
        }
    }
    
    pub fn web3_permission_validation_error(
        wallet_address: impl Into<String>,
        permission: impl Into<String>,
        permission_type: Web3PermissionType,
        chain_id: u64,
    ) -> Self {
        Self::Web3PermissionValidationError {
            wallet_address: wallet_address.into(),
            permission: permission.into(),
            permission_type,
            blockchain_data: None,
            chain_id,
            context: ErrorContext::default(),
        }
    }
    
    pub fn database_error(message: impl Into<String>) -> Self {
        Self::DatabaseError {
            message: message.into(),
            error_code: None,
            table: None,
            query_type: None,
            context: ErrorContext::default(),
        }
    }
    
    pub fn infrastructure_error(message: impl Into<String>) -> Self {
        Self::InternalServerError {
            message: message.into(),
            error_id: Uuid::new_v4(),
            stack_trace: None,
            context: ErrorContext::default(),
        }
    }
    
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
            url: None,
            status_code: None,
            timeout_ms: None,
            retry_count: 0,
            context: ErrorContext::default(),
        }
    }
    
    pub fn authentication_error(
        message: impl Into<String>,
        auth_type: AuthenticationType,
    ) -> Self {
        Self::AuthenticationError {
            message: message.into(),
            auth_type,
            provider: None,
            context: ErrorContext::default(),
        }
    }
    
    pub fn timeout_error(operation: impl Into<String>, timeout_ms: u64, elapsed_ms: u64) -> Self {
        Self::TimeoutError {
            operation: operation.into(),
            timeout_ms,
            elapsed_ms,
            context: ErrorContext::default(),
        }
    }
    
    // Context manipulation methods
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        match &mut self {
            AppError::BusinessRuleViolation { context: c, .. } => *c = context,
            AppError::EntityNotFound { context: c, .. } => *c = context,
            AppError::ValidationError { context: c, .. } => *c = context,
            AppError::PermissionDenied { context: c, .. } => *c = context,
            AppError::ResourceConflict { context: c, .. } => *c = context,
            AppError::BlockchainRpcError { context: c, .. } => *c = context,
            AppError::WalletValidationError { context: c, .. } => *c = context,
            AppError::Web3PermissionValidationError { context: c, .. } => *c = context,
            AppError::SmartContractError { context: c, .. } => *c = context,
            AppError::CrossChainError { context: c, .. } => *c = context,
            AppError::DatabaseError { context: c, .. } => *c = context,
            AppError::CacheError { context: c, .. } => *c = context,
            AppError::NetworkError { context: c, .. } => *c = context,
            AppError::ConfigurationError { context: c, .. } => *c = context,
            AppError::ServiceUnavailable { context: c, .. } => *c = context,
            AppError::AuthenticationError { context: c, .. } => *c = context,
            AppError::AuthorizationError { context: c, .. } => *c = context,
            AppError::RateLimitExceeded { context: c, .. } => *c = context,
            AppError::ConcurrencyConflict { context: c, .. } => *c = context,
            AppError::ExternalServiceError { context: c, .. } => *c = context,
            AppError::InternalServerError { context: c, .. } => *c = context,
            AppError::TimeoutError { context: c, .. } => *c = context,
        }
        self
    }
    
    pub fn with_component(mut self, component: &str) -> Self {
        self.get_context_mut().component = component.to_string();
        self
    }
    
    pub fn with_operation(mut self, operation: &str) -> Self {
        self.get_context_mut().operation = operation.to_string();
        self
    }
    
    pub fn with_user_context(mut self, user_context: String) -> Self {
        self.get_context_mut().user_context = Some(user_context);
        self
    }
    
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.get_context_mut().chain_id = Some(chain_id);
        self
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.get_context_mut().metadata.insert(key, value);
    }
    
    // Utility methods
    pub fn error_id(&self) -> Uuid {
        self.get_context().error_id
    }
    
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AppError::NetworkError { .. } |
            AppError::BlockchainRpcError { .. } |
            AppError::TimeoutError { .. } |
            AppError::ServiceUnavailable { .. } |
            AppError::CacheError { .. }
        )
    }
    
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            AppError::ValidationError { .. } |
            AppError::BusinessRuleViolation { .. } |
            AppError::PermissionDenied { .. } |
            AppError::AuthenticationError { .. } |
            AppError::AuthorizationError { .. } |
            AppError::WalletValidationError { .. }
        )
    }
    
    pub fn is_system_error(&self) -> bool {
        !self.is_user_error()
    }
    
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::InternalServerError { .. } => ErrorSeverity::Critical,
            AppError::DatabaseError { .. } => ErrorSeverity::High,
            AppError::ServiceUnavailable { .. } => ErrorSeverity::High,
            AppError::BlockchainRpcError { .. } => ErrorSeverity::Medium,
            AppError::Web3PermissionValidationError { .. } => ErrorSeverity::Medium,
            AppError::NetworkError { .. } => ErrorSeverity::Medium,
            AppError::TimeoutError { .. } => ErrorSeverity::Medium,
            AppError::AuthenticationError { .. } => ErrorSeverity::Medium,
            AppError::ValidationError { .. } => ErrorSeverity::Low,
            AppError::BusinessRuleViolation { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
    
    // Helper method to get context reference
    fn get_context(&self) -> &ErrorContext {
        match self {
            AppError::BusinessRuleViolation { context, .. } => context,
            AppError::EntityNotFound { context, .. } => context,
            AppError::ValidationError { context, .. } => context,
            AppError::PermissionDenied { context, .. } => context,
            AppError::ResourceConflict { context, .. } => context,
            AppError::BlockchainRpcError { context, .. } => context,
            AppError::WalletValidationError { context, .. } => context,
            AppError::Web3PermissionValidationError { context, .. } => context,
            AppError::SmartContractError { context, .. } => context,
            AppError::CrossChainError { context, .. } => context,
            AppError::DatabaseError { context, .. } => context,
            AppError::CacheError { context, .. } => context,
            AppError::NetworkError { context, .. } => context,
            AppError::ConfigurationError { context, .. } => context,
            AppError::ServiceUnavailable { context, .. } => context,
            AppError::AuthenticationError { context, .. } => context,
            AppError::AuthorizationError { context, .. } => context,
            AppError::RateLimitExceeded { context, .. } => context,
            AppError::ConcurrencyConflict { context, .. } => context,
            AppError::ExternalServiceError { context, .. } => context,
            AppError::InternalServerError { context, .. } => context,
            AppError::TimeoutError { context, .. } => context,
        }
    }
    
    // Helper method to get mutable context reference
    fn get_context_mut(&mut self) -> &mut ErrorContext {
        match self {
            AppError::BusinessRuleViolation { context, .. } => context,
            AppError::EntityNotFound { context, .. } => context,
            AppError::ValidationError { context, .. } => context,
            AppError::PermissionDenied { context, .. } => context,
            AppError::ResourceConflict { context, .. } => context,
            AppError::BlockchainRpcError { context, .. } => context,
            AppError::WalletValidationError { context, .. } => context,
            AppError::Web3PermissionValidationError { context, .. } => context,
            AppError::SmartContractError { context, .. } => context,
            AppError::CrossChainError { context, .. } => context,
            AppError::DatabaseError { context, .. } => context,
            AppError::CacheError { context, .. } => context,
            AppError::NetworkError { context, .. } => context,
            AppError::ConfigurationError { context, .. } => context,
            AppError::ServiceUnavailable { context, .. } => context,
            AppError::AuthenticationError { context, .. } => context,
            AppError::AuthorizationError { context, .. } => context,
            AppError::RateLimitExceeded { context, .. } => context,
            AppError::ConcurrencyConflict { context, .. } => context,
            AppError::ExternalServiceError { context, .. } => context,
            AppError::InternalServerError { context, .. } => context,
            AppError::TimeoutError { context, .. } => context,
        }
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Result type for application operations
pub type AppResult<T> = Result<T, AppError>;

// Note: Legacy domain_error module has been removed in favor of unified AppError system

/// Conversion from Value Object errors
impl From<super::value_object::ValueObjectError> for AppError {
    fn from(error: super::value_object::ValueObjectError) -> Self {
        let validation_type = match &error {
            super::value_object::ValueObjectError::InvalidFormat(_) => ValidationType::Format,
            super::value_object::ValueObjectError::OutOfRange(_) => ValidationType::Range,
            super::value_object::ValueObjectError::Required(_) => ValidationType::Required,
            super::value_object::ValueObjectError::ValidationFailed(_) => ValidationType::Custom("validation_failed".to_string()),
        };
        
        AppError::ValidationError {
            field: "value_object".to_string(),
            message: error.to_string(),
            validation_type,
            context: ErrorContext::default(),
        }
    }
}

/// Conversion from reqwest HTTP errors
impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        let url = error.url().map(|u| u.to_string());
        let status_code = error.status().map(|s| s.as_u16());
        let timeout_ms = if error.is_timeout() { Some(30000) } else { None };
        
        AppError::NetworkError {
            message: error.to_string(),
            url,
            status_code,
            timeout_ms,
            retry_count: 0,
            context: ErrorContext::new("http_client", "request"),
        }
    }
}

/// Conversion from serde JSON errors
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::validation_error("json", format!("JSON parsing error: {}", error))
    }
}

/// Conversion from Diesel database errors (for migrated repositories)
impl From<diesel::result::Error> for AppError {
    fn from(error: diesel::result::Error) -> Self {
        use diesel::result::Error as DieselError;
        use diesel::result::DatabaseErrorKind;

        match error {
            DieselError::NotFound => AppError::entity_not_found("database_record", "query_result"),
            DieselError::DatabaseError(kind, info) => {
                let message = format!("Database error ({:?}): {}", kind, info.message());
                let error_code = info.statement_position().map(|p| p.to_string());

                // Map specific database errors
                match kind {
                    DatabaseErrorKind::UniqueViolation => AppError::ResourceConflict {
                        resource: "database_record".to_string(),
                        reason: message,
                        context: ErrorContext::new("diesel_repository", "constraint_violation"),
                    },
                    DatabaseErrorKind::ForeignKeyViolation => AppError::validation_error(
                        "foreign_key",
                        format!("Foreign key constraint violation: {}", info.message())
                    ),
                    _ => AppError::DatabaseError {
                        message,
                        error_code,
                        table: None,
                        query_type: None,
                        context: ErrorContext::new("diesel_repository", "query"),
                    },
                }
            },
            _ => AppError::DatabaseError {
                message: error.to_string(),
                error_code: None,
                table: None,
                query_type: None,
                context: ErrorContext::new("diesel_repository", "query"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_error_creation() {
        let error = AppError::validation_error("Invalid wallet address format");
        assert!(error.is_user_error());
        assert_eq!(error.severity(), ErrorSeverity::Low);
    }
    
    #[test]
    fn test_web3_error_creation() {
        let error = AppError::blockchain_rpc_error(1, "https://mainnet.infura.io", "Connection timeout");
        assert!(error.is_system_error());
        assert!(error.is_retryable());
        assert_eq!(error.severity(), ErrorSeverity::Medium);
    }
    
    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("wallet_service", "validate_signature")
            .with_user("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string())
            .with_chain_id(1);
            
        assert_eq!(context.component, "wallet_service");
        assert_eq!(context.operation, "validate_signature");
        assert!(context.user_context.is_some());
        assert_eq!(context.chain_id, Some(1));
    }
    
    #[test]
    fn test_domain_error_conversion() {
        let domain_error = super::super::domain_error::AppError::business_rule_violation("Test rule");
        let app_error: AppError = domain_error.into();
        
        matches!(app_error, AppError::BusinessRuleViolation { .. });
    }
}