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

pub mod prelude;
pub mod core;
pub mod config;
pub mod constants;
pub mod schemas;
pub mod infrastructure;
pub mod connection;

// Auth domain modules (moved from apps/backend/src/auth/*)
pub mod auth_service;
pub mod challenge_service;
pub mod verification_service;
pub mod token_service;
pub mod key_manager;
pub mod granular_permissions;
pub mod unified_permission_service;

// ============================================================================
// EXPORTS — UNIFIED PERMISSION SYSTEM
// ============================================================================

pub use unified_permission_service::{
    UnifiedPermissionService, PermissionDetail, PermissionSource as UnifiedPermissionSource,
    PermissionStats as UnifiedPermissionStats, GrantPermissionRequest, RevokePermissionRequest,
    AssignPlanRequest, RemovePlanRequest,
};

pub use auth_service::{
    UnifiedWeb3AuthService, Web3Challenge, Web3VerificationRequest, Web3AuthResult,
    Web3Permission, Web3PermissionType, Web3AuthError,
};

pub use token_service::{
    OpenIDTokenService, OpenIDTokenResponse, AccessTokenClaims, IdTokenClaims,
    Web3AuthTokenRequest, OpenIDTokenError, RefreshTokenInfo,
};

pub use key_manager::KeyManager;
pub use granular_permissions::{
    GranularPermissionClaim, PermissionSource as GranularPermissionSource, GranularPermissionSet,
    PermissionValidationResult, ValidationContext as GranularValidationContext,
    GranularPermissionError,
};

pub use prelude::TlsPool;
