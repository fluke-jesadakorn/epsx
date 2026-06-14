//! `FreePlanRankingOffsetService` — the day-1 gRPC impl of
//! `epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery`.
//!
//! Wave 13a Track A ships this as a **stub** that returns the
//! free-plan offset for every wallet. The same behavior as the
//! wave-12 in-process `FreePlanWalletRankingOffsetQuery` in
//! `apps/analytics/src/main.rs:122-141` — the difference is that
//! the gRPC version goes over the wire via tonic instead of being
//! inlined in the analytics binary's address space.
//!
//! Spec: `docs/wave8-service-boundary/ROADMAP.md` §17.1 (created
//! by this track).
//!
//! ## Why a stub?
//!
//! The full tier-aware impl requires the
//! `wallet_plan_assignments` table (a future `epsx-identity`
//! PostgreSQL schema) and a permission service that resolves
//! "what's the minimum offset across this wallet's active plans?".
//! Both depend on the `epsx-identity` (dioxus-microservices
//! `services/identity/`) binary's migration to a standalone
//! PostgreSQL database — that migration is wave-14+ work. For
//! day 1, the stub preserves the same fallback behavior the
//! monolith's `web/analytics/eps/cache.rs:78-81` already uses:
//! "if we can't determine the wallet's tier, default to
//! `FREE_PLAN_RANKING_OFFSET` (100)".
//!
//! ## Wire-shape contract
//!
//! The `GetWalletRankingOffsetResponse.offset` field is `int32`
//! and equals `RankingOffset::value()` on the server side. The
//! client's `RankingOffset::from(i32)` conversion clamps
//! out-of-range values to the free-plan default, so a buggy
//! server (returning `-1` or `> 1000`) cannot crash a future
//! client — it falls back to the same default the stub returns.
//!
//! ## Error mapping
//!
//! The port returns `AppResult<RankingOffset>`; gRPC requires
//! `Result<tonic::Response<...>, tonic::Status>`. Day 1 never
//! returns `Err(_)` (the stub always succeeds with the
//! free-plan offset). When the tier-aware impl lands in a
//! future wave, transient DB errors will map to
//! `tonic::Status::unavailable("...")` (the standard
//! `UNAVAILABLE` gRPC code) so the client's fallback contract
//! (timeout → fall back to free-plan offset) can trigger.

use async_trait::async_trait;
use epsx_contracts::errors::{AppError, AppResult};
use epsx_contracts::value_objects::ranking_offset::RankingOffset;
use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;
use tonic::Status;
use tracing::{info, instrument};

// The generated gRPC types live in `$OUT_DIR/identity.rs` and
// are included via `tonic::include_proto!` in `main.rs` — but the
// `IdentityService` impl below only needs the request / response
// shapes, which the gRPC trait imports automatically when we
// `impl Identity for FreePlanRankingOffsetService`.

/// STUB impl — returns free-plan offset for every wallet.
///
/// Future waves swap this for a tier-aware impl that reads
/// from the `wallet_plan_assignments` table. The
/// `WalletRankingOffsetQuery` port is the stable contract;
/// this struct is one of N possible impls.
#[derive(Debug, Default, Clone, Copy)]
pub struct FreePlanRankingOffsetService;

#[async_trait]
impl WalletRankingOffsetQuery for FreePlanRankingOffsetService {
    #[instrument(skip(self), fields(wallet = %wallet))]
    async fn get_wallet_ranking_offset(
        &self,
        wallet: &str,
    ) -> AppResult<RankingOffset> {
        info!(
            wallet = %wallet,
            "FreePlanRankingOffsetService: returning free-plan offset (wave 13a stub; \
             wave-13+ will read from wallet_plan_assignments)"
        );
        Ok(RankingOffset::free_plan())
    }
}

/// Map the port's `AppError` to a gRPC `Status`. Day-1 never
/// triggers this (the stub never errors), but the helper is
/// here so the future tier-aware impl can reuse it.
///
/// Mapping:
///   - `AppError` with `kind: ValidationError` → `Status::invalid_argument`
///   - `AppError` with `kind: AggregateNotFound` → `Status::not_found`
///   - any other `AppError` → `Status::internal` (with the
///     message redacted; full error stays in the server logs)
pub fn map_app_error_to_status(err: AppError) -> Status {
    use epsx_contracts::errors::ErrorKind;
    match err.kind {
        ErrorKind::ValidationError => Status::invalid_argument(err.message),
        ErrorKind::AggregateNotFound => Status::not_found(err.message),
        // All other error kinds (DB, internal, external) collapse
        // to `internal` so we don't leak infra details to the
        // gRPC client. The correlation_id stays in the server log
        // for ops.
        _ => {
            tracing::error!(
                correlation_id = %err.correlation_id,
                kind = ?err.kind,
                "FreePlanRankingOffsetService: internal error"
            );
            Status::internal("internal error")
        }
    }
}
