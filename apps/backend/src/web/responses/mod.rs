// ============================================================================
// WEB RESPONSES MODULE
// Unified response types and utilities for all API endpoints
// ============================================================================

pub mod unified_response;
pub mod wrappers;

// Export unified response types
pub use unified_response::{
    ErrorInfo, PaginationMeta, PermissionContext, ResponseMeta, RestrictedAction,
    UnifiedApiResponse,
};

// Export domain-specific response wrappers
pub use wrappers::{
    create_pagination, AdminResponse, AnalyticsResponse, AuthResponse, ToUnifiedResponse,
};

// Re-export macros
pub use crate::{error_response, success_response};
