//! In-process adapter for `WalletRankingOffsetQuery`.
//!
//! Wraps `Arc<UnifiedPermissionService>` 1:1 and adapts the
//! legacy `i32` return type of
//! `UnifiedPermissionService::get_wallet_ranking_offset` into
//! the new `RankingOffset` value object. The conversion uses
//! `RankingOffset::from(i32)` so out-of-range inputs (e.g. from
//! a corrupt seed data) fall back to the free-plan default
//! instead of poisoning the query response.
//!
//! The plan-tier rank offset path is the *highest-risk single
//! line* in the wave-12 analytics lift (per audit-analytics.md
//! §10 Refactor #1). After this adapter lands, the analytics
//! handlers depend on `Arc<dyn WalletRankingOffsetQuery>`
//! instead of `Arc<UnifiedPermissionService>`, which is the
//! precondition for the analytics service to be lifted out of
//! the monolith.
//!
//! In-tree evidence:
//!   - `apps/backend/src/auth/unified_permission_service.rs`
//!     `get_wallet_ranking_offset` (line 795; the only direct
//!     caller before this wave).
//!   - `epsx_contracts::constants::FREE_PLAN_RANKING_OFFSET`
//!     — the value-object default and the SQL function's
//!     fallback match exactly.

use std::sync::Arc;

use async_trait::async_trait;
use epsx_contracts::value_objects::ranking_offset::RankingOffset;
use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;
use epsx_contracts::errors::AppResult;
use epsx_contracts::errors::AppError;
use epsx_identity_shared::prelude::AppError as InProcessAppError;

use crate::auth::UnifiedPermissionService;

#[derive(Clone)]
pub struct InProcessWalletRankingOffsetAdapter {
    service: Arc<UnifiedPermissionService>,
}

impl InProcessWalletRankingOffsetAdapter {
    pub fn new(service: Arc<UnifiedPermissionService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl WalletRankingOffsetQuery for InProcessWalletRankingOffsetAdapter {
    async fn get_wallet_ranking_offset(
        &self,
        wallet: &str,
    ) -> AppResult<RankingOffset> {
        // The underlying service lowercases the wallet internally;
        // we forward the call as-is. Returning the i32 and
        // converting via `RankingOffset::from` keeps the adapter
        // panic-free even if the SQL function ever returns a
        // negative or oversized value. The AppError conversion
        // mirrors the one in the authority adapter; see
        // ROADMAP §5 R4 for the collapse plan.
        let raw = self
            .service
            .get_wallet_ranking_offset(wallet)
            .await
            .map_err(shared_app_error_to_port)?;
        Ok(RankingOffset::from(raw))
    }
}

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

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;

    /// Compile-time smoke: assert the trait is dyn-compatible and
    /// the adapter can be wrapped in an `Arc<dyn …>`. The actual
    /// DB call needs a live pool so it stays in the integration
    /// test layer; the unit-test surface here is the API shape
    /// and the i32 → RankingOffset conversion.
    #[test]
    fn dyn_compatible() {
        fn _assert(_: Arc<dyn WalletRankingOffsetQuery>) {}
    }

    #[test]
    fn ranking_offset_conversion_clamps_out_of_range() {
        // Mirrors the in-process function returning a corrupted
        // value (e.g. negative). The adapter falls back to the
        // free-plan default rather than panicking.
        let r: RankingOffset = (-7).into();
        assert_eq!(r, RankingOffset::default());
        let r: RankingOffset = 99_999.into();
        assert_eq!(r, RankingOffset::default());
    }

    /// Error bridge coverage — same pattern as the authority
    /// adapter. Non-exhaustive match ensures future variants
    /// in `epsx_identity_shared::AppError` get explicit arms.
    #[test]
    fn shared_error_bridge_preserves_kind() {
        use epsx_contracts::errors::ErrorKind;
        use epsx_identity_shared::prelude::AppError as Shared;
        let cases = [
            (Shared::NotFound("x".into()), ErrorKind::AggregateNotFound),
            (Shared::DatabaseError("x".into()), ErrorKind::DatabaseError),
            (
                Shared::ValidationError("x".into()),
                ErrorKind::ValidationError,
            ),
            (
                Shared::AuthenticationError("x".into()),
                ErrorKind::AuthenticationError,
            ),
            (
                Shared::AuthorizationError("x".into()),
                ErrorKind::AuthorizationError,
            ),
            (
                Shared::ConfigurationError("x".into()),
                ErrorKind::ConfigurationError,
            ),
            (Shared::NetworkError("x".into()), ErrorKind::NetworkError),
            (Shared::RateLimitExceeded, ErrorKind::RateLimitExceeded),
            (
                Shared::ServiceUnavailable("x".into()),
                ErrorKind::ServiceUnavailable,
            ),
            (Shared::InternalError("x".into()), ErrorKind::InternalError),
            (Shared::Conflict("x".into()), ErrorKind::ConcurrencyConflict),
        ];
        for (shared, expected) in cases {
            let port = shared_app_error_to_port(shared);
            assert_eq!(port.kind, expected, "ErrorKind mapping for {expected:?}");
        }
    }
}
