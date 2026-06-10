// ============================================================================
// COMPREHENSIVE ERROR SYSTEM MODULE
// Structured error responses for backend-centric architecture
// ============================================================================

// Permission error system - THE AUTHORITY for permission-related errors
pub mod permission_errors;

// Re-export main types for easy access
pub use permission_errors::{
    PermissionError,
    PermissionErrorResponse,
    PermissionErrorDetails,
    RiskLevel,
    PermissionExpiryInfo,
    UsageInfo,
    SecurityInfo,
    UpgradeInfo,
};