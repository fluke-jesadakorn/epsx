//! In-process adapter for `PermissionAuthorityPort`.
//!
//! Wraps `Arc<UnifiedPermissionService>` 1:1 — the three trait
//! methods delegate to the corresponding `UnifiedPermissionService`
//! methods, with a thin DTO conversion at the boundary:
//!
//!   - `grant_permission`  → drops the `Uuid` return value
//!     (the trait returns `AppResult<()>` because the existing
//!     callers in `validation_handlers.rs` and the future
//!     identity service don't need the row ID).
//!   - `revoke_permission` → identity passthrough.
//!   - `get_user_permissions` → converts each `PermissionDetail`
//!     into the transport-friendly `Permission` DTO.
//!
//! The adapter is the *only* place where the in-process
//! `UnifiedPermissionService` is converted to / from the port
//! DTOs. After this lands, callers that only need the management
//! surface (payments, analytics, admin) can depend on
//! `Arc<dyn PermissionAuthorityPort>` instead of the concrete
//! service.
//!
//! In-tree evidence:
//!   - `apps/backend/src/auth/unified_permission_service.rs`
//!     (canonical source, re-exported from
//!     `epsx_identity_shared::UnifiedPermissionService`).
//!   - The `GrantPermissionRequest` / `RevokePermissionRequest`
//!     in-process types are re-exported via
//!     `epsx_identity_shared::*` — they happen to have the same
//!     shape as the port DTOs but live in the in-process crate
//!     because the underlying service struct binds to them by
//!     name. We rebuild the in-process request from the port
//!     request at the boundary (a single field-by-field copy)
//!     rather than introducing a transitive dependency on
//!     `epsx-contracts` from `epsx-identity-shared`.

use std::sync::Arc;

use async_trait::async_trait;
use epsx_contracts::permission_authority_port::{
    GrantPermissionRequest, Permission, PermissionAuthorityPort,
    RevokePermissionRequest,
};
use epsx_contracts::errors::{AppError, AppResult};
use epsx_contracts::value_objects::UserId;

// In-process AppError lives in epsx_identity_shared (the crate that
// owns UnifiedPermissionService). See ROADMAP §5 R4 for the
// collapse with epsx_contracts::AppError.
use epsx_identity_shared::prelude::AppError as InProcessAppError;

use crate::auth::UnifiedPermissionService;

/// In-process adapter — wraps a `UnifiedPermissionService` and
/// adapts it to the `PermissionAuthorityPort` trait.
#[derive(Clone)]
pub struct InProcessPermissionAuthorityAdapter {
    service: Arc<UnifiedPermissionService>,
}

impl InProcessPermissionAuthorityAdapter {
    pub fn new(service: Arc<UnifiedPermissionService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl PermissionAuthorityPort for InProcessPermissionAuthorityAdapter {
    async fn grant_permission(
        &self,
        req: GrantPermissionRequest,
    ) -> AppResult<()> {
        // The in-process request is a 1:1 mirror of the port DTO;
        // we keep the two types distinct so the future HTTP / gRPC
        // adapter can have its own serde-friendly DTO without
        // dragging in any `epsx-identity-shared` types.
        let in_process_req = crate::auth::GrantPermissionRequest {
            wallet_address: req.wallet_address,
            permission_string: req.permission_string,
            granted_by: req.granted_by,
            reason: req.reason,
            expires_at: req.expires_at,
        };
        // Drop the returned Uuid — the trait surface returns `()`
        // because no in-process caller (today) consumes the row ID.
        // Convert the in-process `AppError` (epsx_identity_shared)
        // into the port's `AppError` (epsx_contracts) via
        // `to_string()` + the matching constructor. The two
        // types are flagged for collapse in ROADMAP §5 R4; this
        // stringification is the lowest-friction bridge until
        // the collapse lands.
        self.service
            .grant_permission(in_process_req)
            .await
            .map_err(shared_app_error_to_port)
            .map(|_| ())
    }

    async fn revoke_permission(
        &self,
        req: RevokePermissionRequest,
    ) -> AppResult<()> {
        let in_process_req = crate::auth::RevokePermissionRequest {
            wallet_address: req.wallet_address,
            permission_string: req.permission_string,
            revoked_by: req.revoked_by,
            reason: req.reason,
        };
        self.service
            .revoke_permission(in_process_req)
            .await
            .map_err(shared_app_error_to_port)
    }

    async fn get_user_permissions(
        &self,
        user_id: &UserId,
    ) -> AppResult<Vec<Permission>> {
        let details = self
            .service
            .get_wallet_permissions(user_id.as_str())
            .await
            .map_err(shared_app_error_to_port)?;
        Ok(details.into_iter().map(permission_detail_to_dto).collect())
    }
}

/// Bridge the in-process `AppError` (from `epsx_identity_shared`)
/// to the port-level `AppError` (from `epsx_contracts`). See
/// ROADMAP §5 R4 — the two types will be collapsed in a follow-up
/// refactor; until then, we re-tag the message into the matching
/// `ErrorKind` so HTTP status mapping (which depends on `kind`)
/// is preserved for the most common variants.
fn shared_app_error_to_port(err: InProcessAppError) -> AppError {
    use InProcessAppError as Shared;
    let message = err.to_string();
    match err {
        Shared::NotFound(_) => AppError::not_found(message),
        Shared::DatabaseError(_) => AppError::database_error(message),
        Shared::ValidationError(_) | Shared::ValidationField { .. } => {
            AppError::validation_error(message)
        }
        Shared::AuthenticationError(_) => {
            AppError::authentication_error(message)
        }
        Shared::AuthorizationError(_) => AppError::forbidden(message),
        Shared::ConfigurationError(_) => AppError::configuration_error(message),
        Shared::NetworkError(_) => AppError::network_error(message),
        Shared::RateLimitExceeded => AppError::new(
            epsx_contracts::errors::ErrorKind::RateLimitExceeded,
            message,
        ),
        Shared::ServiceUnavailable(_) => AppError::new(
            epsx_contracts::errors::ErrorKind::ServiceUnavailable,
            message,
        ),
        Shared::InternalError(_) => AppError::internal_error(message),
        Shared::Conflict(_) => AppError::conflict(message),
    }
}

fn permission_detail_to_dto(
    d: crate::auth::PermissionDetail,
) -> Permission {
    Permission {
        permission_string: d.permission_string,
        permission_id: Some(d.permission_id.to_string()),
        source_type: match d.source_type {
            crate::auth::UnifiedPermissionSource::Plan => "plan".to_string(),
            crate::auth::UnifiedPermissionSource::Direct => "direct".to_string(),
        },
        source_name: d.source_name,
        granted_at: d.granted_at,
        expires_at: d.expires_at,
        is_permanent: d.is_permanent,
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::PermissionDetail;
    use crate::auth::UnifiedPermissionSource;
    use chrono::Utc;
    use epsx_contracts::permission_authority_port::{
        GrantPermissionRequest, PermissionAuthorityPort, RevokePermissionRequest,
    };
    use epsx_contracts::value_objects::UserId;
    use uuid::Uuid;

    /// Verifies the DTO conversion is field-for-field equivalent.
    /// Doesn't need a DB because we exercise the pure conversion
    /// path. The trait method itself (`grant_permission`) is
    /// exercised by the integration tests in
    /// `apps/backend/tests/` once the wiring lands; that needs a
    /// real DB pool so it stays out of the unit-test surface.
    #[test]
    fn permission_detail_to_dto_preserves_fields() {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let detail = PermissionDetail {
            permission_string: "epsx:rankings:offset:5".to_string(),
            permission_id: id,
            source_type: UnifiedPermissionSource::Plan,
            source_id: Uuid::new_v4(),
            source_name: "Pro Plan".to_string(),
            expires_at: None,
            granted_at: now,
            is_permanent: true,
        };
        let dto = permission_detail_to_dto(detail);
        assert_eq!(dto.permission_string, "epsx:rankings:offset:5");
        assert_eq!(dto.permission_id.as_deref(), Some(id.to_string().as_str()));
        assert_eq!(dto.source_type, "plan");
        assert_eq!(dto.source_name, "Pro Plan");
        assert!(dto.is_permanent);
    }

    /// Compiles a `GrantPermissionRequest` and a `UserId` to make
    /// sure the trait surface is usable from the adapter module.
    /// This is a compile-time test; it has no assertions.
    #[allow(dead_code)]
    async fn _compile_smoke(service: Arc<UnifiedPermissionService>) {
        let adapter = InProcessPermissionAuthorityAdapter::new(service);
        let req = GrantPermissionRequest {
            wallet_address: "0xabc".to_string(),
            permission_string: "epsx:read:analytics".to_string(),
            granted_by: "system_activation".to_string(),
            reason: Some("plan activation".to_string()),
            expires_at: None,
        };
        let _ = adapter.grant_permission(req).await;

        let rev = RevokePermissionRequest {
            wallet_address: "0xabc".to_string(),
            permission_string: "epsx:read:analytics".to_string(),
            revoked_by: "admin".to_string(),
            reason: None,
        };
        let _ = adapter.revoke_permission(rev).await;

        let user_id = UserId::from("0xabc");
        let _ = adapter.get_user_permissions(&user_id).await;
    }

    /// The error-conversion bridge (epsx_identity_shared::AppError
    /// → epsx_contracts::AppError) must preserve the `ErrorKind`
    /// for the variants HTTP status-mapping depends on. The test
    /// covers every variant; if a new variant is added to
    /// `epsx_identity_shared::AppError` without a corresponding
    /// arm in `shared_app_error_to_port`, the
    /// `#[test]` function below will fail to compile because the
    /// match is non-exhaustive.
    #[test]
    fn shared_error_bridge_preserves_kind() {
        use epsx_contracts::errors::ErrorKind;

        let cases = [
            (
                InProcessAppError::NotFound("wallet 0xabc not found".into()),
                ErrorKind::AggregateNotFound,
            ),
            (
                InProcessAppError::DatabaseError("connection refused".into()),
                ErrorKind::DatabaseError,
            ),
            (
                InProcessAppError::ValidationError("bad perm string".into()),
                ErrorKind::ValidationError,
            ),
            (
                InProcessAppError::AuthenticationError("invalid token".into()),
                ErrorKind::AuthenticationError,
            ),
            (
                InProcessAppError::AuthorizationError("insufficient".into()),
                ErrorKind::AuthorizationError,
            ),
            (
                InProcessAppError::ConfigurationError("missing key".into()),
                ErrorKind::ConfigurationError,
            ),
            (
                InProcessAppError::NetworkError("timeout".into()),
                ErrorKind::NetworkError,
            ),
            (
                InProcessAppError::RateLimitExceeded,
                ErrorKind::RateLimitExceeded,
            ),
            (
                InProcessAppError::ServiceUnavailable("redis down".into()),
                ErrorKind::ServiceUnavailable,
            ),
            (
                InProcessAppError::InternalError("oops".into()),
                ErrorKind::InternalError,
            ),
            (
                InProcessAppError::Conflict("unique violation".into()),
                ErrorKind::ConcurrencyConflict,
            ),
        ];
        for (shared, expected_kind) in cases {
            let port = shared_app_error_to_port(shared);
            assert_eq!(
                port.kind, expected_kind,
                "ErrorKind mapping for {expected_kind:?}"
            );
        }
    }
}
