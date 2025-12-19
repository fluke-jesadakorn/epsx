// Response Wrappers - Domain-Specific Response Types
// All delegate to UnifiedApiResponse for consistency

use super::unified_response::{UnifiedApiResponse, PaginationMeta, PermissionContext};
use axum::{response::IntoResponse, Json};
use serde::Serialize;

// ============================================================================
// ADMIN RESPONSE WRAPPER
// ============================================================================

/// Admin Response Type - Delegates to UnifiedApiResponse
/// Provides admin-specific convenience methods
pub struct AdminResponse;

impl AdminResponse {
    /// Success response for admin operations
    pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
        UnifiedApiResponse::success(data)
    }

    /// Success with message
    pub fn success_with_message<T: Serialize>(data: T, message: &str) -> impl IntoResponse {
        UnifiedApiResponse::success_with_message(data, message)
    }

    /// Success with pagination (for list endpoints)
    pub fn success_with_pagination<T: Serialize>(data: T, pagination: PaginationMeta) -> impl IntoResponse {
        UnifiedApiResponse::success_with_pagination(data, pagination)
    }

    /// Created response (201)
    pub fn created<T: Serialize>(data: T, message: &str) -> impl IntoResponse {
        (
            axum::http::StatusCode::CREATED,
            UnifiedApiResponse::success_with_message(data, message)
        )
    }

    /// No content response (204)
    pub fn no_content() -> impl IntoResponse {
        axum::http::StatusCode::NO_CONTENT
    }

    /// Bad request error (400)
    pub fn bad_request(reason: &str) -> impl IntoResponse {
        UnifiedApiResponse::<()>::error(400, "Bad request", reason)
    }

    /// Unauthorized error (401)
    pub fn unauthorized(reason: &str) -> impl IntoResponse {
        UnifiedApiResponse::<()>::auth_error(reason)
    }

    /// Forbidden error (403)
    pub fn forbidden(required_permission: &str) -> impl IntoResponse {
        UnifiedApiResponse::<()>::permission_error(required_permission)
    }

    /// Not found error (404)
    pub fn not_found(resource: &str) -> impl IntoResponse {
        UnifiedApiResponse::<()>::not_found(resource)
    }

    /// Server error (500)
    pub fn server_error(reason: &str) -> impl IntoResponse {
        UnifiedApiResponse::<()>::server_error(reason)
    }

    /// Conflict error (409) - for duplicate resources
    pub fn conflict(reason: &str) -> impl IntoResponse {
        UnifiedApiResponse::<()>::error(409, "Conflict", reason)
    }
}

// ============================================================================
// ANALYTICS RESPONSE WRAPPER
// ============================================================================

/// Analytics Response Type - Delegates to UnifiedApiResponse
/// Provides analytics-specific convenience methods
pub struct AnalyticsResponse;

impl AnalyticsResponse {
    /// Success response for analytics data
    pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
        UnifiedApiResponse::success(data)
    }

    /// Success with pagination (most analytics endpoints use this)
    pub fn success_with_pagination<T: Serialize>(data: T, pagination: PaginationMeta) -> impl IntoResponse {
        UnifiedApiResponse::success_with_pagination(data, pagination)
    }

    /// Success with pagination and permissions context
    pub fn success_with_pagination_and_permissions<T: Serialize>(
        data: T,
        pagination: PaginationMeta,
        permissions: PermissionContext,
    ) -> impl IntoResponse {
        UnifiedApiResponse::success_with_pagination_and_permissions(data, pagination, permissions)
    }

    /// Success with permissions (for single-item endpoints)
    pub fn success_with_permissions<T: Serialize>(data: T, permissions: PermissionContext) -> impl IntoResponse {
        UnifiedApiResponse::success_with_permissions(data, permissions)
    }

    /// Permission denied with upgrade suggestion
    pub fn permission_denied_with_upgrade(required_tier: &str) -> impl IntoResponse {
        UnifiedApiResponse::<()>::error(
            403,
            "Permission denied",
            &format!("This feature requires {} tier or higher", required_tier),
        )
    }

    /// Rate limit exceeded
    pub fn rate_limit_exceeded(retry_after: u64) -> impl IntoResponse {
        let response = UnifiedApiResponse::<()>::error(
            429,
            "Rate limit exceeded",
            &format!("Please retry after {} seconds", retry_after),
        );
        (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            Json(response)
        )
    }
}

// ============================================================================
// AUTH RESPONSE WRAPPER
// ============================================================================

/// Auth Response Type - Delegates to UnifiedApiResponse
/// Provides auth-specific convenience methods
pub struct AuthResponse;

impl AuthResponse {
    /// Success response for auth operations
    pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
        UnifiedApiResponse::success(data)
    }

    /// Success with message
    pub fn success_with_message<T: Serialize>(data: T, message: &str) -> impl IntoResponse {
        UnifiedApiResponse::success_with_message(data, message)
    }

    /// Invalid credentials
    pub fn invalid_credentials() -> impl IntoResponse {
        UnifiedApiResponse::<()>::error(
            401,
            "Invalid credentials",
            "The provided credentials are invalid",
        )
    }

    /// Invalid signature
    pub fn invalid_signature() -> impl IntoResponse {
        UnifiedApiResponse::<()>::error(
            401,
            "Invalid signature",
            "The provided signature could not be verified",
        )
    }

    /// Session expired
    pub fn session_expired() -> impl IntoResponse {
        UnifiedApiResponse::<()>::error(
            401,
            "Session expired",
            "Your session has expired. Please sign in again",
        )
    }

    /// Challenge expired
    pub fn challenge_expired() -> impl IntoResponse {
        UnifiedApiResponse::<()>::error(
            400,
            "Challenge expired",
            "The authentication challenge has expired. Please request a new one",
        )
    }
}

// ============================================================================
// PAGINATION HELPER
// ============================================================================

/// Helper to create pagination metadata from query parameters
pub fn create_pagination(
    page: u32,
    limit: u32,
    total: u64,
) -> PaginationMeta {
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
    let has_next = page < total_pages;
    let has_prev = page > 1;

    PaginationMeta {
        page,
        limit,
        total,
        total_pages,
        has_next,
        has_prev,
    }
}

// ============================================================================
// CONVERSION HELPERS
// ============================================================================

/// Convert legacy response format to unified format
pub trait ToUnifiedResponse {
    type Output;
    fn to_unified(self) -> Self::Output;
}

// Example implementation for a common pattern
impl<T: Serialize> ToUnifiedResponse for (bool, T, Option<String>) {
    type Output = UnifiedApiResponse<T>;

    fn to_unified(self) -> Self::Output {
        let (success, data, message) = self;
        if success {
            match message {
                Some(msg) => UnifiedApiResponse::success_with_message(data, &msg),
                None => UnifiedApiResponse::success(data),
            }
        } else {
            // This shouldn't happen with this pattern, but handle it
            UnifiedApiResponse::success(data)
        }
    }
}
