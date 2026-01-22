// ============================================================================
// ADMIN API RESPONSE FORMAT
// Standardized response structure for all admin endpoints
// ============================================================================

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

/// Standardized Admin API Response Structure
/// Used by all admin endpoints for consistent response format
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminApiResponse<T> {
    /// Request success status
    pub success: bool,
    
    /// Response data (only present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    
    /// Error message (only present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    /// Human-readable message
    pub message: String,
    
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Admin-specific metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_meta: Option<AdminMetadata>,
}

/// Admin-specific metadata for operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminMetadata {
    /// Operation performed
    pub operation: String,
    
    /// Admin user who performed the operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performed_by: Option<String>,
    
    /// Pagination info for list operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationInfo>,
    
    /// Permission context for the admin user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<AdminPermissionContext>,
    
    /// Additional operation metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Pagination information for list responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginationInfo {
    pub page: i32,
    pub limit: i32,
    pub total: i32,
    pub total_pages: i32,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

/// Admin permission context
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminPermissionContext {
    /// Admin user tier/plan
    pub admin_plan: String,
    
    /// Available admin actions
    pub available_actions: Vec<String>,
    
    /// Restricted admin actions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted_actions: Option<Vec<String>>,
}

impl<T> AdminApiResponse<T> {
    /// Create successful response
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: message.to_string(),
            timestamp: Utc::now(),
            admin_meta: None,
        }
    }
    
    /// Create successful response with admin metadata
    pub fn success_with_meta(data: T, message: &str, admin_meta: AdminMetadata) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: message.to_string(),
            timestamp: Utc::now(),
            admin_meta: Some(admin_meta),
        }
    }
    
    /// Create error response
    pub fn error(error_message: &str, user_message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error_message.to_string()),
            message: user_message.to_string(),
            timestamp: Utc::now(),
            admin_meta: None,
        }
    }
    
    /// Create authentication error
    pub fn auth_error() -> Self {
        Self::error(
            "Authentication required",
            "Admin authentication is required to access this resource"
        )
    }
    
    /// Create permission error
    pub fn permission_error(required_permission: &str) -> Self {
        Self::error(
            &format!("Permission denied: {}", required_permission),
            "You don't have permission to perform this action"
        )
    }
    
    /// Create validation error
    pub fn validation_error(details: &str) -> Self {
        Self::error(
            &format!("Validation failed: {}", details),
            "The request contains invalid data"
        )
    }
    
    /// Create not found error
    pub fn not_found(resource: &str) -> Self {
        Self::error(
            &format!("{} not found", resource),
            &format!("The requested {} was not found", resource)
        )
    }
    
    /// Create server error
    pub fn server_error() -> Self {
        Self::error(
            "Internal server error",
            "An unexpected error occurred. Please try again later."
        )
    }
}

impl AdminMetadata {
    /// Create metadata for list operation
    pub fn list_operation(operation: &str, pagination: PaginationInfo) -> Self {
        Self {
            operation: operation.to_string(),
            performed_by: None,
            pagination: Some(pagination),
            permissions: None,
            metadata: None,
        }
    }
    
    /// Create metadata for CRUD operation
    pub fn crud_operation(operation: &str, performed_by: Option<String>) -> Self {
        Self {
            operation: operation.to_string(),
            performed_by,
            pagination: None,
            permissions: None,
            metadata: None,
        }
    }
    
    /// Add admin permissions context
    pub fn with_permissions(mut self, permissions: AdminPermissionContext) -> Self {
        self.permissions = Some(permissions);
        self
    }
    
    /// Add additional metadata
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl AdminPermissionContext {
    /// Create admin permission context
    pub fn new(admin_plan: &str, available_actions: Vec<String>) -> Self {
        Self {
            admin_plan: admin_plan.to_string(),
            available_actions,
            restricted_actions: None,
        }
    }
    
    /// Add restricted actions
    pub fn with_restrictions(mut self, restricted_actions: Vec<String>) -> Self {
        self.restricted_actions = Some(restricted_actions);
        self
    }
}

// Implementation of IntoResponse for automatic conversion
impl<T> IntoResponse for AdminApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            // Determine status code based on error type
            if self.error.as_ref().is_some_and(|e| e.contains("Authentication")) {
                StatusCode::UNAUTHORIZED
            } else if self.error.as_ref().is_some_and(|e| e.contains("Permission")) {
                StatusCode::FORBIDDEN
            } else if self.error.as_ref().is_some_and(|e| e.contains("Validation")) {
                StatusCode::BAD_REQUEST
            } else if self.error.as_ref().is_some_and(|e| e.contains("not found")) {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        
        (status, Json(self)).into_response()
    }
}

/// Type alias for JSON response with AdminApiResponse
pub type AdminJsonResponse<T> = Json<AdminApiResponse<T>>;

/// Helper macros for creating admin responses
#[macro_export]
macro_rules! admin_success {
    ($data:expr, $message:expr) => {
        $crate::web::admin::responses::AdminApiResponse::success($data, $message)
    };
    ($data:expr, $message:expr, $meta:expr) => {
        $crate::web::admin::responses::AdminApiResponse::success_with_meta($data, $message, $meta)
    };
}

#[macro_export]
macro_rules! admin_error {
    ($error:expr, $message:expr) => {
        $crate::web::admin::responses::AdminApiResponse::<()>::error($error, $message)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_admin_success_response() {
        let response = AdminApiResponse::success(json!({"test": "data"}), "Operation successful");
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.message, "Operation successful");
    }

    #[test]
    fn test_admin_error_response() {
        let response = AdminApiResponse::<()>::permission_error("admin:users:read");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
        assert!(response.error.unwrap().contains("Permission denied"));
    }

    #[test]
    fn test_admin_metadata() {
        let pagination = PaginationInfo {
            page: 1,
            limit: 10,
            total: 100,
            total_pages: 10,
            has_next_page: true,
            has_previous_page: false,
        };
        
        let meta = AdminMetadata::list_operation("list_users", pagination);
        assert_eq!(meta.operation, "list_users");
        assert!(meta.pagination.is_some());
    }
}