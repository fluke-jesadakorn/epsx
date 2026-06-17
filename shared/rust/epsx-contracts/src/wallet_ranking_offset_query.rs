//! Wallet ranking offset query port (Wave 10 — Track C, ROADMAP §5 R6).
//!
//! Read-only cross-cutting query: given a wallet address, return the
//! plan-tier ranking offset. This is the "what's this wallet's tier?"
//! query that analytics uses to decide how many rankings rows to
//! show. Per CLAUDE.md, the *enforcement* of "this wallet may not see
//! rank < 100" must remain local in any backend that consumes
//! analytics; the read-only query is delegable.
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/ROADMAP.md` §5 R6
//!   - `docs/wave8-service-boundary/audit-analytics.md` §10 Refactor #1
//!   - In-tree adapter target:
//!     `apps/backend/src/auth/unified_permission_service.rs`
//!     `get_wallet_ranking_offset` (returns `i32`).
//!
//! The `RankingOffset` value object is a *new* type in
//! `epsx-contracts::value_objects::ranking_offset` — it didn't
//! previously exist as a value object (the underlying function
//! returns `i32`), so creating it is part of the wave-10
//! relocation. The audit's claim that "RankingOffset value object
//! already exists in
//! `apps/backend/src/domain/market_analytics/value_objects/`" is
//! aspirational; the path the audit listed does not have a
//! `ranking_offset.rs` file at HEAD `9f794784`. The wrapper
//! adds documentation, validation, and a `Serialize` /
//! `Deserialize` representation that the future identity service
//! can serve over the wire.

use async_trait::async_trait;

use crate::errors::AppResult;
use crate::value_objects::ranking_offset::RankingOffset;

#[async_trait]
pub trait WalletRankingOffsetQuery: Send + Sync {
    /// Return the plan-tier ranking offset for `wallet`. Lower
    /// numbers mean "this wallet sees higher ranks"; the
    /// `FREE_PLAN_RANKING_OFFSET` constant in
    /// `epsx_contracts::constants` is the floor for
    /// unauthenticated / Free Plan users.
    ///
    /// The caller passes a lowercased, EIP-55-checksummed or
    /// plain-hex wallet address; the adapter is responsible for
    /// any further normalization.
    async fn get_wallet_ranking_offset(
        &self,
        wallet: &str,
    ) -> AppResult<RankingOffset>;
}

#[cfg(test)]
mod object_safety {
    use super::*;
    use std::sync::Arc;

    /// Type-asserts that `WalletRankingOffsetQuery` is dyn-compatible
    /// by holding an `Arc<dyn WalletRankingOffsetQuery>` reference.
    /// If the trait ever gains a non-object-safe method, this
    /// function will fail to compile.
    fn _assert_object_safe(_: Arc<dyn WalletRankingOffsetQuery>) {}
}
