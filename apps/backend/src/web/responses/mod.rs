// ============================================================================
// WEB RESPONSES MODULE
// Unified response types and utilities for all API endpoints
// ============================================================================

pub mod unified_response;
pub mod wrappers;

// Export unified response types
pub use unified_response::{
    UnifiedApiResponse,
    ErrorInfo,
    ResponseMeta,
    PaginationMeta,
    PermissionContext,
    RestrictedAction,
};

// Export domain-specific response wrappers
pub use wrappers::{
    AdminResponse,
    AnalyticsResponse,
    AuthResponse,
    create_pagination,
    ToUnifiedResponse,
};

// Re-export macros
pub use crate::{success_response, error_response};