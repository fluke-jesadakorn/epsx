// ============================================================================
// UNIFIED API RESPONSE FORMAT
// Standard response structure for all API endpoints
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Consistent response format across all endpoints
 * - Clear success/error indication
 * - Structured error information
 * - Frontend can handle all responses uniformly
 * - Backend makes all authorization decisions
 */

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

/// Unified API Response Structure
/// Used by all endpoints for consistent response format
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UnifiedApiResponse<T = Value> {
    /// Request success status
    pub success: bool,
    
    /// Response data (only present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    
    /// Error information (only present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    
    /// Response metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

/// Error Information Structure
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorInfo {
    /// HTTP status code
    pub code: u16,
    
    /// Human-readable error message
    pub message: String,
    
    /// Detailed reason/explanation
    pub reason: String,
    
    /// Error type/category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Response Metadata
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResponseMeta {
    /// Request timestamp
    pub timestamp: String,

    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// API version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Pagination info (for list endpoints)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,

    /// Permission context (what user can do)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<PermissionContext>,
}

/// Pagination Metadata
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginationMeta {
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Permission Context for Frontend Display
/// Backend tells frontend what user can do
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PermissionContext {
    /// User's role/tier level
    pub user_tier: String,
    
    /// Available actions for current context
    pub available_actions: Vec<String>,
    
    /// Restricted actions with reasons
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted_actions: Option<Vec<RestrictedAction>>,
    
    /// Feature access flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_access: Option<Value>,
}

/// Restricted Action Information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RestrictedAction {
    pub action: String,
    pub reason: String,
    pub required_tier: Option<String>,
}

impl<T> UnifiedApiResponse<T> {
    /// Create successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ResponseMeta::default()),
        }
    }

    /// Create successful response with metadata
    pub fn success_with_meta(data: T, meta: ResponseMeta) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(meta),
        }
    }

    /// Create successful response with pagination
    pub fn success_with_pagination(data: T, pagination: PaginationMeta) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ResponseMeta::default().with_pagination(pagination)),
        }
    }

    /// Create successful response with permissions context
    pub fn success_with_permissions(data: T, permissions: PermissionContext) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ResponseMeta::default().with_permissions(permissions)),
        }
    }

    /// Create successful response with pagination and permissions
    pub fn success_with_pagination_and_permissions(
        data: T,
        pagination: PaginationMeta,
        permissions: PermissionContext,
    ) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(
                ResponseMeta::default()
                    .with_pagination(pagination)
                    .with_permissions(permissions)
            ),
        }
    }

    /// Create successful response with message
    pub fn success_with_message(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ResponseMeta::default().with_message(message.to_string())),
        }
    }
    
    /// Create error response
    pub fn error(code: u16, message: &str, reason: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code,
                message: message.to_string(),
                reason: reason.to_string(),
                error_type: None,
                details: None,
            }),
            meta: Some(ResponseMeta::default()),
        }
    }
    
    /// Create error response with details
    pub fn error_with_details(
        code: u16,
        message: &str,
        reason: &str,
        error_type: &str,
        details: Value,
    ) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code,
                message: message.to_string(),
                reason: reason.to_string(),
                error_type: Some(error_type.to_string()),
                details: Some(details),
            }),
            meta: Some(ResponseMeta::default()),
        }
    }
    
    /// Create authentication error
    pub fn auth_error(reason: &str) -> Self {
        Self::error(401, "Authentication required", reason)
    }
    
    /// Create authorization error
    pub fn permission_error(required_permission: &str) -> Self {
        Self::error(
            403,
            "Permission denied",
            &format!("Required permission: {}", required_permission),
        )
    }
    
    /// Create validation error
    pub fn validation_error(details: Value) -> Self {
        Self::error_with_details(
            400,
            "Validation failed",
            "Request validation failed",
            "validation_error",
            details,
        )
    }
    
    /// Create not found error
    pub fn not_found(resource: &str) -> Self {
        Self::error(404, "Resource not found", &format!("{} not found", resource))
    }
    
    /// Create server error
    pub fn server_error(reason: &str) -> Self {
        Self::error(500, "Internal server error", reason)
    }
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id: None,
            version: Some("v1".to_string()),
            message: None,
            pagination: None,
            permissions: None,
        }
    }
}

impl ResponseMeta {
    /// Add request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Add message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Add pagination
    pub fn with_pagination(mut self, pagination: PaginationMeta) -> Self {
        self.pagination = Some(pagination);
        self
    }

    /// Add permission context
    pub fn with_permissions(mut self, permissions: PermissionContext) -> Self {
        self.permissions = Some(permissions);
        self
    }
}

impl PermissionContext {
    /// Create permission context from JWT permissions (permission-first approach)
    pub fn from_permissions(user_permissions: &[String]) -> Self {
        // Derive tier from permissions for display purposes only
        let user_tier = if user_permissions.iter().any(|p| p.starts_with("admin:")) {
            "admin".to_string()
        } else if user_permissions.iter().any(|p| p.contains("premium")) {
            "premium".to_string()
        } else if user_permissions.iter().any(|p| p.contains("standard")) {
            "standard".to_string()
        } else {
            "basic".to_string()
        };

        // Convert structured permissions to frontend action permissions
        let available_actions: Vec<String> = user_permissions
            .iter()
            .filter_map(|perm| {
                // Map backend permissions to frontend actions
                if perm.contains("analytics") && perm.contains("read") {
                    Some("view_analytics".to_string())
                } else if perm.contains("export") {
                    Some("export_data".to_string())
                } else if perm.contains("admin") && perm.contains("manage") {
                    Some("manage_users".to_string())
                } else if perm == "admin:*:*" {
                    Some("system_admin".to_string())
                } else {
                    None
                }
            })
            .collect();

        Self {
            user_tier,
            available_actions,
            restricted_actions: None,
            feature_access: None,
        }
    }
}

// Implementation of IntoResponse for automatic conversion
impl<T> IntoResponse for UnifiedApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            match &self.error {
                Some(err) => StatusCode::from_u16(err.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                None => StatusCode::INTERNAL_SERVER_ERROR,
            }
        };
        
        (status, Json(self)).into_response()
    }
}

/// Helper macro for creating success responses
#[macro_export]
macro_rules! success_response {
    ($data:expr) => {
        $crate::web::responses::unified_response::UnifiedApiResponse::success($data)
    };
    ($data:expr, $meta:expr) => {
        $crate::web::responses::unified_response::UnifiedApiResponse::success_with_meta($data, $meta)
    };
}

/// Helper macro for creating error responses
#[macro_export]
macro_rules! error_response {
    ($code:expr, $message:expr, $reason:expr) => {
        $crate::web::responses::unified_response::UnifiedApiResponse::<()>::error($code, $message, $reason)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_response() {
        let response = UnifiedApiResponse::success(json!({"test": "data"}));
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_error_response() {
        let response = UnifiedApiResponse::<()>::error(400, "Bad request", "Invalid input");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        
        let error = response.error.unwrap();
        assert_eq!(error.code, 400);
        assert_eq!(error.message, "Bad request");
    }

    // NOTE: Test disabled - from_user_tier method removed during refactoring
    /*
    #[test]
    fn test_permission_context() {
        let context = PermissionContext::from_user_tier("premium", &[]);
        assert_eq!(context.user_tier, "premium");
        assert!(context.available_actions.contains(&"export_data".to_string()));
    }
    */
}