// kernel extraction wave9 — epsx-identity-shared (Track B)
//
// This crate is the **Shape B (shared library)** extraction of
// `apps/backend/src/auth/*` per the wave8 auth audit. The source
// files moved here are still reachable from `apps/backend` via a
// re-export shim at `apps/backend/src/auth/mod.rs`.
//
// Crate name: `epsx-identity-shared` (renamed from `epsx-identity`
// to avoid collision with `services/identity` binary crate, also
// named `epsx-identity`).
//
// Constraint compliance:
//   - No network split (Shape A is not in scope for this track).
//   - `core::permissions::has_permission` hot path stays callable
//     in-process in the backend binary (CLAUDE.md).
//   - No new workspace dependencies are added by this crate.

pub mod config;
pub mod constants;
pub mod core;
pub mod infrastructure;
pub mod prelude;

// Auth domain modules (moved from apps/backend/src/auth/*)
pub mod auth_service;
pub mod challenge_service;
pub mod granular_permissions;
pub mod key_manager;
pub mod refresh_token_digest;
pub mod token_service;
pub mod unified_permission_service;
pub mod verification_service;

// ============================================================================
// EXPORTS — UNIFIED PERMISSION SYSTEM
// ============================================================================

pub use unified_permission_service::{
    AssignPlanRequest, GrantPermissionRequest, PermissionDetail,
    PermissionSource as UnifiedPermissionSource, PermissionStats as UnifiedPermissionStats,
    RemovePlanRequest, RevokePermissionRequest, UnifiedPermissionService,
};

pub use auth_service::{
    UnifiedWeb3AuthService, Web3AuthError, Web3AuthResult, Web3Challenge, Web3Permission,
    Web3PermissionType, Web3VerificationRequest,
};

pub use token_service::{
    AccessTokenClaims, IdTokenClaims, OpenIDTokenError, OpenIDTokenResponse, OpenIDTokenService,
    RefreshTokenInfo, Web3AuthTokenRequest,
};

pub use granular_permissions::{
    GranularPermissionClaim, GranularPermissionError, GranularPermissionSet,
    PermissionSource as GranularPermissionSource, PermissionValidationResult,
    ValidationContext as GranularValidationContext,
};
pub use key_manager::KeyManager;
pub use refresh_token_digest::{
    DigestedRefreshToken, IssuedRefreshToken, RefreshTokenCredential, RefreshTokenDigest,
    RefreshTokenDigestError, RefreshTokenKeyring,
};

pub use prelude::TlsPool;
