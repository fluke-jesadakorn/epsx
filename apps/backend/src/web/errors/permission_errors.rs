// ============================================================================
// COMPREHENSIVE PERMISSION ERROR SYSTEM (Phase 1.3)
// Structured error responses for backend-centric permission architecture
// Frontend/admin applications consume these standardized error responses
// ============================================================================

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Comprehensive permission error types - THE AUTHORITY for error classification
#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("Authentication required: {message}")]
    AuthenticationRequired { message: String, code: String },
    
    #[error("Permission denied: {permission} - {reason}")]
    PermissionDenied {
        permission: String,
        reason: String,
        suggested_actions: Vec<String>,
        upgrade_plan: Option<String>,
    },
    
    #[error("Permission expired: {permission} expired at {expired_at}")]
    PermissionExpired {
        permission: String,
        expired_at: chrono::DateTime<chrono::Utc>,
        renewal_url: Option<String>,
    },
    
    #[error("Usage limit exceeded: {current_usage}/{limit} for {permission}")]
    UsageLimitExceeded {
        permission: String,
        current_usage: u32,
        limit: u32,
        reset_at: Option<chrono::DateTime<chrono::Utc>>,
        upgrade_plan: Option<String>,
    },
    
    #[error("Insufficient permission plan: {current_plan} insufficient for {required_plan}")]
    InsufficientPlan {
        current_plan: String,
        required_plan: String,
        upgrade_url: Option<String>,
        benefits: Vec<String>,
    },
    
    #[error("Security restriction: {reason}")]
    SecurityRestriction {
        reason: String,
        risk_level: RiskLevel,
        contact_support: bool,
    },
    
    #[error("Resource not found: {resource_type} with id {resource_id}")]
    ResourceNotFound {
        resource_type: String,
        resource_id: String,
    },
    
    #[error("Validation error: {field} - {message}")]
    ValidationError {
        field: String,
        message: String,
        validation_code: String,
    },
    
    #[error("System error: Permission validation temporarily unavailable")]
    SystemError {
        error_id: String,
        retry_after: Option<u64>, // seconds
    },
    
    #[error("Rate limit exceeded: Too many permission requests")]
    RateLimitExceeded {
        retry_after: u64, // seconds
        daily_limit: Option<u32>,
    },
}

/// Risk level classification for security restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium, 
    High,
    Critical,
}

/// Usage period types for rate limiting
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UsagePeriod {
    Hourly,
    #[default]
    Daily,
    Monthly,
}

impl UsagePeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            UsagePeriod::Hourly => "hourly",
            UsagePeriod::Daily => "daily",
            UsagePeriod::Monthly => "monthly",
        }
    }
}

/// User context for enriched permission error responses
/// This allows handlers to pass actual user information for better error messages
#[derive(Debug, Clone, Default)]
pub struct PermissionErrorContext {
    /// The user's wallet address
    pub wallet_address: Option<String>,
    /// The user's current permission plan (e.g., "free", "starter", "professional")
    pub current_plan: Option<String>,
    /// The resource path being accessed
    pub resource_path: Option<String>,
    /// The HTTP method being used
    pub http_method: Option<String>,
    /// The usage period for rate limiting context
    pub usage_period: Option<UsagePeriod>,
}

impl PermissionErrorContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_wallet(mut self, wallet: impl Into<String>) -> Self {
        self.wallet_address = Some(wallet.into());
        self
    }

    pub fn with_plan(mut self, plan: impl Into<String>) -> Self {
        self.current_plan = Some(plan.into());
        self
    }

    pub fn with_resource(mut self, path: impl Into<String>, method: impl Into<String>) -> Self {
        self.resource_path = Some(path.into());
        self.http_method = Some(method.into());
        self
    }

    pub fn with_period(mut self, period: UsagePeriod) -> Self {
        self.usage_period = Some(period);
        self
    }
}

/// Standardized permission error response structure
/// This is THE FORMAT that frontend/admin applications expect
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionErrorResponse {
    /// Error classification for programmatic handling
    pub error_type: String,
    
    /// HTTP status code (401, 403, etc.)
    pub status_code: u16,
    
    /// Human-readable error message
    pub message: String,
    
    /// Detailed error information for debugging
    pub details: PermissionErrorDetails,
    
    /// Suggested actions for the user/developer
    pub suggested_actions: Vec<String>,
    
    /// User-friendly error message for UI display
    pub user_message: String,
    
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Unique error tracking ID
    pub error_id: String,
    
    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,
}

/// Detailed error information structure
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionErrorDetails {
    /// The permission that was being validated
    pub permission: Option<String>,
    
    /// The user ID that attempted the action
    pub wallet_address: Option<String>,
    
    /// The resource path that was accessed
    pub resource_path: Option<String>,
    
    /// HTTP method used
    pub http_method: Option<String>,
    
    /// Current user permission plan
    pub current_plan: Option<String>,
    
    /// Required permission plan for this action
    pub required_plan: Option<String>,
    
    /// Permission expiry information
    pub expiry_info: Option<PermissionExpiryInfo>,
    
    /// Usage limit information
    pub usage_info: Option<UsageInfo>,
    
    /// Security risk assessment
    pub security_info: Option<SecurityInfo>,
    
    /// Upgrade/renewal information
    pub upgrade_info: Option<UpgradeInfo>,
}

/// Permission expiry information
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionExpiryInfo {
    pub expired_at: chrono::DateTime<chrono::Utc>,
    pub grace_period_until: Option<chrono::DateTime<chrono::Utc>>,
    pub auto_renewal: bool,
    pub renewal_url: Option<String>,
}

/// Usage limit information
#[derive(Debug, Serialize, Deserialize)]
pub struct UsageInfo {
    pub current_usage: u32,
    pub limit: u32,
    pub period: String, // "daily", "monthly", "hourly"
    pub reset_at: Option<chrono::DateTime<chrono::Utc>>,
    pub usage_percentage: f32,
}

/// Security risk information
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub risk_level: RiskLevel,
    pub risk_factors: Vec<String>,
    pub automated_restriction: bool,
    pub manual_review_required: bool,
    pub contact_support_url: Option<String>,
}

/// Plan upgrade information
#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeInfo {
    pub current_plan: String,
    pub required_plan: String,
    pub upgrade_url: Option<String>,
    pub pricing_url: Option<String>,
    pub benefits: Vec<String>,
    pub trial_available: bool,
}

/// Implementation of permission error responses
impl PermissionError {
    /// Convert permission error to standardized response format
    pub fn to_error_response(&self) -> PermissionErrorResponse {
        let error_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now();
        
        match self {
            PermissionError::AuthenticationRequired { message, code } => {
                PermissionErrorResponse {
                    error_type: "authentication_required".to_string(),
                    status_code: 401,
                    message: format!("Authentication required: {}", message),
                    details: PermissionErrorDetails {
                        permission: None,
                        wallet_address: None,
                        resource_path: None,
                        http_method: None,
                        current_plan: None,
                        required_plan: None,
                        expiry_info: None,
                        usage_info: None,
                        security_info: None,
                        upgrade_info: None,
                    },
                    suggested_actions: vec![
                        "Sign in with your wallet address".to_string(),
                        "Ensure Web3 wallet is connected".to_string(),
                        "Check authentication headers".to_string(),
                    ],
                    user_message: "Please sign in to access this feature".to_string(),
                    timestamp,
                    error_id,
                    context: HashMap::from([
                        ("auth_code".to_string(), serde_json::Value::String(code.clone())),
                    ]),
                }
            }
            
            PermissionError::PermissionDenied { permission, reason, suggested_actions, upgrade_plan } => {
                let mut context = HashMap::new();
                if let Some(tier) = upgrade_plan {
                    context.insert("upgrade_plan".to_string(), serde_json::Value::String(tier.clone()));
                }
                
                PermissionErrorResponse {
                    error_type: "permission_denied".to_string(),
                    status_code: 403,
                    message: format!("Permission denied: {} - {}", permission, reason),
                    details: PermissionErrorDetails {
                        permission: Some(permission.clone()),
                        wallet_address: None,
                        resource_path: None,
                        http_method: None,
                        current_plan: None,
                        required_plan: upgrade_plan.clone(),
                        expiry_info: None,
                        usage_info: None,
                        security_info: None,
                        upgrade_info: upgrade_plan.as_ref().map(|tier| UpgradeInfo {
                            current_plan: "unknown".to_string(),
                            required_plan: tier.clone(),
                            upgrade_url: Some("/payment".to_string()),
                            pricing_url: Some("/payment".to_string()),
                            benefits: vec![
                                format!("Access to {} permission", permission),
                                "Enhanced features".to_string(),
                            ],
                            trial_available: true,
                        }),
                    },
                    suggested_actions: suggested_actions.clone(),
                    user_message: format!("You don't have permission to access this feature. {}", 
                        if upgrade_plan.is_some() { "Consider upgrading your plan." } else { "" }),
                    timestamp,
                    error_id,
                    context,
                }
            }
            
            PermissionError::PermissionExpired { permission, expired_at, renewal_url } => {
                PermissionErrorResponse {
                    error_type: "permission_expired".to_string(),
                    status_code: 403,
                    message: format!("Permission expired: {} expired at {}", permission, expired_at),
                    details: PermissionErrorDetails {
                        permission: Some(permission.clone()),
                        wallet_address: None,
                        resource_path: None,
                        http_method: None,
                        current_plan: None,
                        required_plan: None,
                        expiry_info: Some(PermissionExpiryInfo {
                            expired_at: *expired_at,
                            grace_period_until: None,
                            auto_renewal: false,
                            renewal_url: renewal_url.clone(),
                        }),
                        usage_info: None,
                        security_info: None,
                        upgrade_info: None,
                    },
                    suggested_actions: vec![
                        "Renew your subscription".to_string(),
                        "Contact support for extension".to_string(),
                        "Upgrade to a higher permission plan".to_string(),
                    ],
                    user_message: "Your permission has expired. Please renew your subscription to continue.".to_string(),
                    timestamp,
                    error_id,
                    context: HashMap::new(),
                }
            }
            
            PermissionError::UsageLimitExceeded { permission, current_usage, limit, reset_at, upgrade_plan } => {
                let usage_percentage = (*current_usage as f32 / *limit as f32) * 100.0;
                
                PermissionErrorResponse {
                    error_type: "usage_limit_exceeded".to_string(),
                    status_code: 429,
                    message: format!("Usage limit exceeded: {}/{} for {}", current_usage, limit, permission),
                    details: PermissionErrorDetails {
                        permission: Some(permission.clone()),
                        wallet_address: None,
                        resource_path: None,
                        http_method: None,
                        current_plan: None,
                        required_plan: upgrade_plan.clone(),
                        expiry_info: None,
                        usage_info: Some(UsageInfo {
                            current_usage: *current_usage,
                            limit: *limit,
                            period: "daily".to_string(), // TODO: Make configurable
                            reset_at: *reset_at,
                            usage_percentage,
                        }),
                        security_info: None,
                        upgrade_info: upgrade_plan.as_ref().map(|tier| UpgradeInfo {
                            current_plan: "basic".to_string(), // TODO: Get from user context
                            required_plan: tier.clone(),
                            upgrade_url: Some("/payment".to_string()),
                            pricing_url: Some("/payment".to_string()),
                            benefits: vec![
                                "Higher usage limits".to_string(),
                                "Unlimited access".to_string(),
                            ],
                            trial_available: true,
                        }),
                    },
                    suggested_actions: vec![
                        format!("Wait until {} for limit reset", 
                            reset_at.map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string()).unwrap_or_default()),
                        "Upgrade your plan for higher limits".to_string(),
                        "Optimize your usage patterns".to_string(),
                    ],
                    user_message: "You've reached your daily usage limit. Please wait for the reset or upgrade your plan.".to_string(),
                    timestamp,
                    error_id,
                    context: HashMap::from([
                        ("usage_percentage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(usage_percentage as f64).unwrap())),
                    ]),
                }
            }
            
            PermissionError::InsufficientPlan { current_plan, required_plan, upgrade_url, benefits } => {
                PermissionErrorResponse {
                    error_type: "insufficient_plan".to_string(),
                    status_code: 403,
                    message: format!("Insufficient permission plan: {} required, you have {}", required_plan, current_plan),
                    details: PermissionErrorDetails {
                        permission: None,
                        wallet_address: None,
                        resource_path: None,
                        http_method: None,
                        current_plan: Some(current_plan.clone()),
                        required_plan: Some(required_plan.clone()),
                        expiry_info: None,
                        usage_info: None,
                        security_info: None,
                        upgrade_info: Some(UpgradeInfo {
                            current_plan: current_plan.clone(),
                            required_plan: required_plan.clone(),
                            upgrade_url: upgrade_url.clone(),
                            pricing_url: Some("/payment".to_string()),
                            benefits: benefits.clone(),
                            trial_available: true,
                        }),
                    },
                    suggested_actions: vec![
                        format!("Upgrade to {} permission plan", required_plan),
                        "View pricing options".to_string(),
                        "Start a free trial".to_string(),
                    ],
                    user_message: format!("This feature requires {} permission plan. Upgrade your plan to access it.", required_plan),
                    timestamp,
                    error_id,
                    context: HashMap::new(),
                }
            }
            
            PermissionError::SecurityRestriction { reason, risk_level, contact_support } => {
                PermissionErrorResponse {
                    error_type: "security_restriction".to_string(),
                    status_code: 403,
                    message: format!("Security restriction: {}", reason),
                    details: PermissionErrorDetails {
                        permission: None,
                        wallet_address: None,
                        resource_path: None,
                        http_method: None,
                        current_plan: None,
                        required_plan: None,
                        expiry_info: None,
                        usage_info: None,
                        security_info: Some(SecurityInfo {
                            risk_level: risk_level.clone(),
                            risk_factors: vec![reason.clone()],
                            automated_restriction: true,
                            manual_review_required: matches!(risk_level, RiskLevel::High | RiskLevel::Critical),
                            contact_support_url: if *contact_support { Some("/support".to_string()) } else { None },
                        }),
                        upgrade_info: None,
                    },
                    suggested_actions: if *contact_support {
                        vec![
                            "Contact support for assistance".to_string(),
                            "Verify your identity".to_string(),
                            "Review security requirements".to_string(),
                        ]
                    } else {
                        vec![
                            "Try again later".to_string(),
                            "Verify your request parameters".to_string(),
                        ]
                    },
                    user_message: "Access temporarily restricted for security reasons.".to_string(),
                    timestamp,
                    error_id,
                    context: HashMap::from([
                        ("risk_level".to_string(), serde_json::Value::String(format!("{:?}", risk_level))),
                    ]),
                }
            }
            
            PermissionError::SystemError { error_id: sys_error_id, retry_after } => {
                PermissionErrorResponse {
                    error_type: "system_error".to_string(),
                    status_code: 500,
                    message: "Permission validation temporarily unavailable".to_string(),
                    details: PermissionErrorDetails {
                        permission: None,
                        wallet_address: None,
                        resource_path: None,
                        http_method: None,
                        current_plan: None,
                        required_plan: None,
                        expiry_info: None,
                        usage_info: None,
                        security_info: None,
                        upgrade_info: None,
                    },
                    suggested_actions: vec![
                        format!("Retry after {} seconds", retry_after.unwrap_or(30)),
                        "Check system status".to_string(),
                        "Contact support if issue persists".to_string(),
                    ],
                    user_message: "Service temporarily unavailable. Please try again in a few moments.".to_string(),
                    timestamp,
                    error_id: sys_error_id.clone(),
                    context: HashMap::from([
                        ("retry_after".to_string(), serde_json::Value::Number(serde_json::Number::from(retry_after.unwrap_or(30)))),
                    ]),
                }
            }
            
            // Handle other error types similarly...
            _ => {
                PermissionErrorResponse {
                    error_type: "unknown_error".to_string(),
                    status_code: 500,
                    message: self.to_string(),
                    details: PermissionErrorDetails {
                        permission: None,
                        wallet_address: None,
                        resource_path: None,
                        http_method: None,
                        current_plan: None,
                        required_plan: None,
                        expiry_info: None,
                        usage_info: None,
                        security_info: None,
                        upgrade_info: None,
                    },
                    suggested_actions: vec![
                        "Try again later".to_string(),
                        "Contact support".to_string(),
                    ],
                    user_message: "An unexpected error occurred.".to_string(),
                    timestamp,
                    error_id,
                    context: HashMap::new(),
                }
            }
        }
    }

    /// Convert permission error to response with enriched user context
    /// This method uses actual user information instead of hardcoded defaults
    pub fn to_error_response_with_context(&self, ctx: &PermissionErrorContext) -> PermissionErrorResponse {
        let mut response = self.to_error_response();
        
        // Enrich with wallet address
        if let Some(wallet) = &ctx.wallet_address {
            response.details.wallet_address = Some(wallet.clone());
        }
        
        // Enrich with current plan (replacing hardcoded "basic")
        if let Some(plan) = &ctx.current_plan {
            response.details.current_plan = Some(plan.clone());
            
            // Also update upgrade_info if present
            if let Some(ref mut upgrade_info) = response.details.upgrade_info {
                upgrade_info.current_plan = plan.clone();
            }
        }
        
        // Enrich with resource path
        if let Some(path) = &ctx.resource_path {
            response.details.resource_path = Some(path.clone());
        }
        
        // Enrich with HTTP method
        if let Some(method) = &ctx.http_method {
            response.details.http_method = Some(method.clone());
        }
        
        // Update usage period if specified
        if let Some(period) = &ctx.usage_period {
            if let Some(ref mut usage_info) = response.details.usage_info {
                usage_info.period = period.as_str().to_string();
            }
        }
        
        response
    }
}

/// Axum IntoResponse implementation for permission errors
impl IntoResponse for PermissionError {
    fn into_response(self) -> Response {
        let error_response = self.to_error_response();
        let status_code = StatusCode::from_u16(error_response.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        (status_code, Json(error_response)).into_response()
    }
}

/// Helper functions for creating common permission errors
impl PermissionError {
    /// Create authentication required error
    pub fn authentication_required(message: impl Into<String>) -> Self {
        Self::AuthenticationRequired {
            message: message.into(),
            code: "AUTH_REQUIRED".to_string(),
        }
    }
    
    /// Create permission denied error with upgrade suggestion
    pub fn permission_denied_with_upgrade(
        permission: impl Into<String>,
        reason: impl Into<String>,
        upgrade_plan: impl Into<String>,
    ) -> Self {
        let plan = upgrade_plan.into();
        Self::PermissionDenied {
            permission: permission.into(),
            reason: reason.into(),
            suggested_actions: vec![
                format!("Upgrade to {} plan", plan),
                "View pricing options".to_string(),
            ],
            upgrade_plan: Some(plan),
        }
    }
    
    /// Create usage limit exceeded error
    pub fn usage_limit_exceeded(
        permission: impl Into<String>,
        current: u32,
        limit: u32,
        reset_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self::UsageLimitExceeded {
            permission: permission.into(),
            current_usage: current,
            limit,
            reset_at,
            upgrade_plan: Some("premium".to_string()),
        }
    }
    
    /// Create security restriction error
    pub fn security_restriction(reason: impl Into<String>, risk_level: RiskLevel) -> Self {
        let contact_support = matches!(risk_level, RiskLevel::High | RiskLevel::Critical);
        Self::SecurityRestriction {
            reason: reason.into(),
            risk_level,
            contact_support,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_denied_error_response() {
        let error = PermissionError::permission_denied_with_upgrade(
            "admin:users:create",
            "Insufficient permission plan level",
            "professional"
        );
        
        let response = error.to_error_response();
        assert_eq!(response.error_type, "permission_denied");
        assert_eq!(response.status_code, 403);
        assert!(!response.suggested_actions.is_empty());
        assert!(response.details.upgrade_info.is_some());
    }
    
    #[test]
    fn test_usage_limit_exceeded_response() {
        let error = PermissionError::usage_limit_exceeded(
            "epsx:analytics:read",
            95,
            100,
            Some(chrono::Utc::now() + chrono::Duration::hours(1))
        );
        
        let response = error.to_error_response();
        assert_eq!(response.error_type, "usage_limit_exceeded");
        assert_eq!(response.status_code, 429);
        assert!(response.details.usage_info.is_some());
        assert_eq!(response.details.usage_info.as_ref().unwrap().usage_percentage, 95.0);
    }
    
    #[test]
    fn test_security_restriction_response() {
        let error = PermissionError::security_restriction(
            "Unusual access pattern detected",
            RiskLevel::High
        );
        
        let response = error.to_error_response();
        assert_eq!(response.error_type, "security_restriction");
        assert_eq!(response.status_code, 403);
        assert!(response.details.security_info.is_some());
        assert!(response.details.security_info.as_ref().unwrap().manual_review_required);
    }

    #[test]
    fn test_error_response_with_context() {
        let error = PermissionError::permission_denied_with_upgrade(
            "epsx:analytics:read",
            "Upgrade required for this feature",
            "professional"
        );
        
        let ctx = PermissionErrorContext::new()
            .with_wallet("0x1234567890abcdef1234567890abcdef12345678")
            .with_plan("starter")
            .with_resource("/api/analytics/rankings", "GET");
        
        let response = error.to_error_response_with_context(&ctx);
        
        assert_eq!(response.error_type, "permission_denied");
        assert_eq!(response.details.wallet_address, Some("0x1234567890abcdef1234567890abcdef12345678".to_string()));
        assert_eq!(response.details.current_plan, Some("starter".to_string()));
        assert_eq!(response.details.resource_path, Some("/api/analytics/rankings".to_string()));
        assert_eq!(response.details.http_method, Some("GET".to_string()));
        
        // The upgrade_info should also be updated with the actual plan
        if let Some(upgrade_info) = &response.details.upgrade_info {
            assert_eq!(upgrade_info.current_plan, "starter");
        }
    }

    #[test]
    fn test_usage_period_enum() {
        assert_eq!(UsagePeriod::Hourly.as_str(), "hourly");
        assert_eq!(UsagePeriod::Daily.as_str(), "daily");
        assert_eq!(UsagePeriod::Monthly.as_str(), "monthly");
    }
}