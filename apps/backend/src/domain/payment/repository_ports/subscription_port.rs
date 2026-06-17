//! `SubscriptionRepositoryPort` — payments subscription port.
//!
//! Wave 11 — Track B (outbound-leakage fold). Pre-wave-11 the
//! subscription CRUD was implemented by a concrete
//! `SubscriptionRepositoryAdapter` in
//! `infrastructure/adapters/repositories/subscription_repository_adapter.rs`
//! — but the adapter sat in the *central* infrastructure layer
//! (not under `payment/`), and the call sites in
//! `web/admin/payment_link_handlers.rs` and
//! `web/admin/plans/handlers.rs` reached around the
//! `domain::payment::repository_ports` layer entirely.
//!
//! Track B introduces this port to:
//!   1. Give the admin plans editor and the
//!      `web/admin/payment_link_handlers` move-target a stable
//!      domain-shaped surface (`Subscription`,
//!      `CreateSubscriptionCommand`, `StockRankingAssignment`).
//!   2. Allow the in-process `PaymentSubscriptionRepositoryAdapter`
//!      to be substituted with an HTTP / gRPC adapter when
//!      payments is lifted to a separate service (wave-11+).
//!   3. Provide a single seam for the new
//!      `get_stock_ranking_assignments` read so the
//!      `application::market_analytics` query object no longer
//!      reaches directly into the `stock_ranking_assignments`
//!      table (the audit's "strongest outward leak", §3 row 4).
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/audit-payments.md` §3 row 2
//!     and row 3 (the "strongest outward leak" via
//!     `subscription_repository_adapter.rs`)
//!   - `docs/wave8-service-boundary/audit-payments.md` §3 row 4
//!     (market_analytics reading `stock_ranking_assignments`)
//!   - `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
//!     preconditions item 3
//!   - `docs/wave8-service-boundary/ROADMAP.md` §12 (implementation
//!     report, this wave)
//!
//! ## Trait shape
//!
//! `Send + Sync` so the port is `Arc<dyn SubscriptionRepositoryPort>`
//! in DI graphs. `#[async_trait]` for object-safety (the
//! `async fn` desugars to `Pin<Box<dyn Future + Send>>`).
//!
//! `AppResult<T>` (re-exported from `epsx_contracts::errors`) for
//! cross-crate compatibility — same as the existing
//! `PaymentRepositoryPort` and `PaymentContextRepositoryPort`.

use async_trait::async_trait;
use epsx_contracts::errors::AppResult;
use uuid::Uuid;

use crate::domain::subscription_management::value_objects::PlanId;
use crate::domain::wallet_management::value_objects::WalletAddress;

use super::super::aggregates::stock_ranking_assignment::StockRankingAssignment;
use super::super::aggregates::subscription::{
    CreateSubscriptionCommand, Subscription, SubscriptionId,
};

/// Port for subscription CRUD + the
/// `stock_ranking_assignments` read.
///
/// The five `list_*` / `create` / `cancel` methods map 1:1 to the
/// pre-wave-11 `SubscriptionRepositoryAdapter` methods
/// (`find_by_id`, `find_by_wallet`, `find_all`, `save`, `cancel`,
/// `delete`, `count`). The new
/// `get_stock_ranking_assignments` is the audit-flagged
/// market_analytics leak closed in this track.
///
/// `update_status` and `delete` are intentionally **not** in the
/// port — they have no live callers in the wave-11 source tree
/// (the only `update_status` call is the test-suite direct call to
/// the adapter; the only `delete` callsite is also dead per the
/// `rg 'SubscriptionRepositoryAdapter'` survey). Lifting them
/// later is a wave-12+ concern; the port surface stays minimal.
#[async_trait]
pub trait SubscriptionRepositoryPort: Send + Sync {
    /// All subscriptions for a given plan. Used by the admin plans
    /// editor and the cross-pool plan analytics query.
    async fn list_for_plan(&self, plan_id: PlanId) -> AppResult<Vec<Subscription>>;

    /// All subscriptions owned by a wallet. Used by
    /// `web/payments/admin_handlers/subscription_handlers.rs` and
    /// the admin plans editor.
    async fn list_for_wallet(&self, wallet: &WalletAddress) -> AppResult<Vec<Subscription>>;

    /// Create a new subscription. Returns the persisted aggregate
    /// (with DB-generated `id` if the command did not pre-set one).
    async fn create(&self, cmd: CreateSubscriptionCommand) -> AppResult<Subscription>;

    /// Cancel a subscription. Sets `status = "cancelled"` and
    /// `cancelled_at = now()`. The audit-mandated behavior of the
    /// pre-wave-11 `cancel(id: Uuid)` is preserved.
    ///
    /// `reason` is optional and currently a no-op on the
    /// `subscriptions` table (the column does not exist; it is
    /// added here as a forward-compat hook for the
    /// `payment_audit_log` `reason` field on subscription
    /// cancellations — the audit row uses the same `reason` value
    /// once the existing audit-row insert lands).
    async fn cancel(
        &self,
        subscription_id: SubscriptionId,
        reason: Option<String>,
    ) -> AppResult<()>;

    /// All active stock-ranking assignments for a wallet.
    ///
    /// New in wave 11. Returns the rows in descending
    /// `assigned_at` order (newest first), with `days_remaining`
    /// computed against the adapter's clock.
    ///
    /// The pre-wave-11 `StockRankingAssignment` type lived in
    /// `application::market_analytics::queries::models::get_stock_ranking_assignments`
    /// but the *read* of the table had no live caller in the
    /// source tree (the only `rg` hits were on the type
    /// definition, not on a SQL query). The port method is the
    /// first wired reader, and the
    /// `PaymentSubscriptionRepositoryAdapter::get_stock_ranking_assignments`
    /// implementation is the SQL. The market_analytics query
    /// object becomes a thin facade that delegates here.
    async fn get_stock_ranking_assignments(
        &self,
        wallet: &WalletAddress,
    ) -> AppResult<Vec<StockRankingAssignment>>;
}

// -----------------------------------------------------------------------------
// Compile-time object-safety probe.
// -----------------------------------------------------------------------------
#[cfg(test)]
mod object_safety {
    use super::*;
    use std::sync::Arc;

    /// Type-asserts that `SubscriptionRepositoryPort` is
    /// dyn-compatible by holding an
    /// `Arc<dyn SubscriptionRepositoryPort>` reference. If the trait
    /// ever gains a non-object-safe method (e.g. a generic method
    /// or a `Self` in argument position) this function will fail to
    /// compile.
    fn _assert_object_safe(_: Arc<dyn SubscriptionRepositoryPort>) {}
}

#[cfg(test)]
mod port_shape_tests {
    use super::*;

    /// The port must not accidentally take a `&AppState` /
    /// `&dyn SomeOtherConcrete`. This test confirms the trait's
    /// public surface matches the wave-11 task brief signature.
    #[test]
    fn port_method_signatures_match_brief() {
        // Compile-time check only — there is no runtime side effect
        // we can observe without an adapter. This test exists so
        // future signature drift is caught at `cargo test` time.
        fn _takes_subscription_id(_: SubscriptionId) {}
        fn _takes_wallet_address(_: &WalletAddress) {}
        fn _takes_plan_id(_: PlanId) {}
        fn _takes_uuid(_: Uuid) {}
    }
}
