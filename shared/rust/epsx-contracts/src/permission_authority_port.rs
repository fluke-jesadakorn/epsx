//! Permission authority port (Wave 10 — Track C, ROADMAP §5 R1).
//!
//! This is a **port**, not a policy engine. The actual RBAC enforcement
//! stays in the monolithic backend per the CLAUDE.md "Permissions &
//! Plan Logic — Backend Only" rule. The port lets non-auth contexts
//! (payments, analytics) reach the permission service through a stable
//! trait surface so the future `epsx-identity` (or HTTP / gRPC) adapter
//! can be substituted in without changing the call sites.
//!
//! DTOs are co-located in this file and derive `Serialize` /
//! `Deserialize` so the future identity service can serve the same
//! surface over HTTP. The `Permission` DTO is intentionally a small,
//! transport-friendly struct — it is **not** the in-process
//! `apps/backend::domain::wallet_management::value_objects::Permission`
//! value object (which has private fields and is not a transport
//! type).
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/ROADMAP.md` §5 R1
//!   - `docs/wave8-service-boundary/audit-payments.md` Refactor #2
//!   - `docs/wave8-service-boundary/audit-notifications.md` §6 (port
//!     pattern: notifications audit Refactor #1 is the same shape
//!     for `NotificationPort`)
//!   - In-tree adapter target:
//!     `apps/backend/src/auth/unified_permission_service.rs` (the
//!     canonical source, re-exported from `epsx-identity-shared`).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::AppResult;
use crate::value_objects::UserId;

/// DTO for a single permission that has been granted to a user.
///
/// Transport-friendly mirror of the in-process
/// `PermissionDetail` struct on `UnifiedPermissionService`. Includes
/// only the fields a caller actually needs to render or
/// re-issue a JWT — internal fields (cache state, audit linkage)
/// stay on the in-process type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    /// Canonical permission string, e.g. `"epsx:rankings:offset:5"`.
    pub permission_string: String,
    /// Stable ID for the underlying `permissions` row.
    /// Surfaced so the admin UI can deep-link into the row.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub permission_id: Option<String>,
    /// Where the permission came from: a plan assignment or a
    /// direct grant. The string form matches the
    /// `PermissionSource` enum in the in-process type.
    pub source_type: String,
    /// Display-friendly label for the source (plan name, admin
    /// display name, etc.).
    pub source_name: String,
    /// When the grant was first issued.
    pub granted_at: DateTime<Utc>,
    /// Optional expiry. `None` ⇒ permanent.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expires_at: Option<DateTime<Utc>>,
    /// `true` if there is no expiry, `false` if the grant is
    /// time-limited.
    pub is_permanent: bool,
}

/// Request payload for `PermissionAuthorityPort::grant_permission`.
///
/// Mirrors the in-process `UnifiedPermissionService::GrantPermissionRequest`
/// but with public-field serde-friendly types so the future identity
/// service can accept the same shape over HTTP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrantPermissionRequest {
    /// Wallet to grant the permission to. Always lowercased by the
    /// adapter before hitting the DB.
    pub wallet_address: String,
    /// Canonical permission string (e.g.
    /// `"epsx:rankings:offset:5"`).
    pub permission_string: String,
    /// Wallet or actor ID that issued the grant. `system_activation`
    /// is the conventional value for the plan-activation path.
    pub granted_by: String,
    /// Optional human-readable reason (audit log).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reason: Option<String>,
    /// Optional expiry timestamp.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Request payload for `PermissionAuthorityPort::revoke_permission`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokePermissionRequest {
    pub wallet_address: String,
    pub permission_string: String,
    pub revoked_by: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reason: Option<String>,
}

/// Permission authority port.
///
/// Cross-cutting abstraction for permission lifecycle operations
/// (grant / revoke / list). Lets non-auth domains (payments,
/// analytics, admin) depend on a stable trait instead of the
/// concrete `UnifiedPermissionService`.
///
/// **Wiring rules (CLAUDE.md):**
///   - The trait lives in `epsx-contracts` (kernel-level).
///   - The in-process adapter (`in_process_authority_adapter`)
///     delegates to `UnifiedPermissionService` 1:1. No new logic.
///   - A future `epsx-identity` binary can supply an HTTP / gRPC
///     adapter with the same trait surface; the in-process
///     callers swap the wiring in `simple_container.rs` and
///     `stateless_service_factory.rs` without code change.
///   - The actual RBAC *enforcement* (bearer middleware
///     `has_permission`) still has to live in every
///     business-service binary — see
///     `web/middleware/permission_validation_middleware.rs` and
///     the CLAUDE.md "Permissions & Plan Logic — Backend Only"
///     rule. This port is for the **management** path
///     (grant / revoke / list), not the read-side check.
#[async_trait]
pub trait PermissionAuthorityPort: Send + Sync {
    /// Grant a direct permission to a wallet.
    ///
    /// Idempotent: re-granting an existing permission updates the
    /// expiry and `granted_by` rather than inserting a duplicate
    /// row.
    async fn grant_permission(
        &self,
        req: GrantPermissionRequest,
    ) -> AppResult<()>;

    /// Revoke a previously granted direct permission. Returns
    /// `AppError::not_found` if the wallet does not currently
    /// hold the permission.
    async fn revoke_permission(
        &self,
        req: RevokePermissionRequest,
    ) -> AppResult<()>;

    /// List every effective permission currently held by `user_id`.
    /// Used by the admin overview and the JWT re-issue path.
    async fn get_user_permissions(
        &self,
        user_id: &UserId,
    ) -> AppResult<Vec<Permission>>;
}

// -----------------------------------------------------------------------------
// Compile-time object-safety probe.
//
// Catches accidental non-object-safe additions (e.g. generic methods,
// `Self` in argument position) at build time so the port can be
// wrapped in `Arc<dyn PermissionAuthorityPort>` from a hand-edited
// call site without runtime surprises.
// -----------------------------------------------------------------------------
#[cfg(test)]
mod object_safety {
    use super::*;
    use std::sync::Arc;

    /// Type-asserts that `PermissionAuthorityPort` is dyn-compatible
    /// by holding an `Arc<dyn PermissionAuthorityPort>` reference.
    /// If the trait ever gains a non-object-safe method, this
    /// function will fail to compile.
    fn _assert_object_safe(_: Arc<dyn PermissionAuthorityPort>) {}
}

#[cfg(test)]
mod dto_round_trip {
    use super::*;

    /// The DTOs are the wire format for the future identity
    /// service. A round-trip through serde must preserve every
    /// field exactly, including the `Option<…>` ones (which use
    /// `#[serde(skip_serializing_if = "Option::is_none", default)]`
    /// to keep the wire format clean).
    #[test]
    fn grant_request_round_trip() {
        let req = GrantPermissionRequest {
            wallet_address: "0xabc".to_string(),
            permission_string: "epsx:rankings:offset:5".to_string(),
            granted_by: "system_activation".to_string(),
            reason: Some("plan activation".to_string()),
            expires_at: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        // `expires_at` should be omitted from the JSON
        // representation (skip_serializing_if = "Option::is_none").
        assert!(
            !json.contains("expires_at"),
            "expires_at should be skipped when None, got: {json}"
        );
        let parsed: GrantPermissionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, parsed);
    }

    #[test]
    fn revoke_request_round_trip() {
        let req = RevokePermissionRequest {
            wallet_address: "0xabc".to_string(),
            permission_string: "epsx:rankings:offset:5".to_string(),
            revoked_by: "admin".to_string(),
            reason: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("reason"));
        let parsed: RevokePermissionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, parsed);
    }

    #[test]
    fn permission_dto_round_trip() {
        let p = Permission {
            permission_string: "epsx:rankings:offset:5".to_string(),
            permission_id: Some("9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d".to_string()),
            source_type: "plan".to_string(),
            source_name: "Pro Plan".to_string(),
            granted_at: chrono::Utc::now(),
            expires_at: None,
            is_permanent: true,
        };
        let json = serde_json::to_string(&p).unwrap();
        let parsed: Permission = serde_json::from_str(&json).unwrap();
        assert_eq!(p, parsed);
    }
}
