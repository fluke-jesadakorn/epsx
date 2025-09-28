// ============================================================================
// WEB RESPONSES MODULE
// Unified response types and utilities for all API endpoints
// ============================================================================

pub mod unified_response;

// Export unified response types
pub use unified_response::{
    UnifiedApiResponse,
    ErrorInfo,
    ResponseMeta,
    PaginationMeta,
    PermissionContext,
    RestrictedAction,
};

// Re-export macros
pub use crate::{success_response, error_response};