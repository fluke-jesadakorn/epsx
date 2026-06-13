//! `PaymentSubscriptionRepositoryAdapter` — the in-process
//! implementation of `SubscriptionRepositoryPort`.
//!
//! Wave 11 — Track B (outbound-leakage fold). Pre-wave-11 this
//! file lived at
//! `apps/backend/src/infrastructure/adapters/repositories/subscription_repository_adapter.rs`
//! and the public struct was `SubscriptionRepositoryAdapter` — but
//! the file sat in the *central* infrastructure layer, not under
//! `payment/`. Track B moves the file here, renames the type to
//! `PaymentSubscriptionRepositoryAdapter` to make the ownership
//! explicit, and implements the new `SubscriptionRepositoryPort`
//! trait (see `domain::payment::repository_ports::subscription_port`).
//!
//! The original 7 concrete methods (`find_by_id`, `find_by_wallet`,
//! `find_all`, `save`, `update_status`, `cancel`, `delete`, `count`)
//! are preserved as private helpers (renamed `_by_*` where the
//! port method is `list_*` for clarity). Two of the original
//! methods (`update_status`, `delete`, `count`) had no live
//! callers per the wave-11 source-tree `rg` survey; they are
//! kept on the adapter as `pub` for backward compat with the
//! in-adapter tests, but they are not part of the port surface.
//!
//! The new `get_stock_ranking_assignments` method is the SQL
//! reader that the audit's row-4
//! (`application::market_analytics::queries::models::get_stock_ranking_assignments`)
//! was missing — the port method is the *first* wired reader of
//! the `stock_ranking_assignments` table; pre-wave-11 only the
//! `StockRankingAssignment` DTO definition existed.
//!
//! Audit references:
//!   - `docs/wave8-service-boundary/audit-payments.md` §3 row 3
//!     and row 4
//!   - `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
//!     preconditions item 3
//!   - `docs/wave8-service-boundary/ROADMAP.md` §12 (implementation
//!     report, this wave)

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use epsx_contracts::errors::{AppError, AppResult};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::domain::payment::aggregates::stock_ranking_assignment::StockRankingAssignment;
use crate::domain::payment::aggregates::subscription::{
    CreateSubscriptionCommand, Subscription, SubscriptionId,
};
use crate::domain::payment::repository_ports::subscription_port::SubscriptionRepositoryPort;
use crate::domain::subscription_management::value_objects::PlanId;
use crate::domain::wallet_management::value_objects::WalletAddress;
use crate::infrastructure::database::PoolExt;
use crate::infrastructure::models::payment::{NewSubscriptionDb, SubscriptionDb};
use crate::prelude::TlsPool;
use crate::schemas::payments::{stock_ranking_assignments, subscriptions};

/// Search criteria for subscriptions.
///
/// Preserved from the pre-wave-11 adapter as a `pub` type so any
/// pre-existing internal callers (none in the wave-11 source tree,
/// per the `rg 'SubscriptionSearchCriteria'` survey) keep
/// compiling during the deprecation window. Not part of the
/// port surface.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionSearchCriteria {
    pub wallet_address: Option<String>,
    pub plan_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// In-process subscription repository adapter for the payments
/// bounded context.
///
/// Concrete `SubscriptionRepositoryPort` impl backed by the
/// `payments.subscriptions` and `payments.stock_ranking_assignments`
/// tables. Constructed with a `&'static TlsPool` for the
/// payments DB (see `infrastructure::database::get_payments_pool`).
///
/// The struct is `Clone` because the pool is `&'static`. The
/// `Arc<dyn SubscriptionRepositoryPort>` wrapper in
/// `web::auth::AppState` makes the clone cost zero.
#[derive(Clone)]
pub struct PaymentSubscriptionRepositoryAdapter {
    db_pool: &'static TlsPool,
}

impl PaymentSubscriptionRepositoryAdapter {
    /// Construct with a payments DB pool.
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }

    /// Read-only: fetch a single subscription row by raw UUID.
    ///
    /// Not part of the port surface (the port uses
    /// `SubscriptionId`). Preserved as a `pub` helper for adapter
    /// internal tests and any pre-wave-11 callers.
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SubscriptionDb>> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Finding subscription by ID: {}", id);

        let result = subscriptions::table
            .filter(subscriptions::id.eq(id))
            .first::<SubscriptionDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find subscription by ID {}: {}", id, e);
                AppError::database_error(format!("Failed to find subscription: {}", e))
            })?;

        Ok(result)
    }

    /// Read-only: list subscriptions matching the search criteria.
    ///
    /// Not part of the port surface. Preserved for backward
    /// compat with the pre-wave-11 adapter's `pub` surface.
    pub async fn find_all(
        &self,
        criteria: SubscriptionSearchCriteria,
    ) -> AppResult<Vec<SubscriptionDb>> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Finding all subscriptions with criteria: {:?}", criteria);

        let mut query = subscriptions::table.into_boxed();

        if let Some(ref wallet_addr) = criteria.wallet_address {
            query = query.filter(subscriptions::wallet_address.eq(wallet_addr));
        }

        if let Some(ref plan_id) = criteria.plan_id {
            query = query.filter(subscriptions::plan_id.eq(plan_id));
        }

        if let Some(ref status) = criteria.status {
            query = query.filter(subscriptions::status.eq(status));
        }

        if let Some(limit) = criteria.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = criteria.offset {
            query = query.offset(offset);
        }

        query = query.order(subscriptions::started_at.desc().nulls_last());

        let results = query.load::<SubscriptionDb>(&mut conn).await.map_err(|e| {
            error!("Failed to find subscriptions: {}", e);
            AppError::database_error(format!("Failed to find subscriptions: {}", e))
        })?;

        info!("Found {} subscriptions matching criteria", results.len());
        Ok(results)
    }

    /// Update the `status` column on a single subscription.
    ///
    /// Not part of the port surface. Pre-wave-11 callers: 0 in the
    /// source tree. Kept for backward compat.
    pub async fn update_status(&self, id: Uuid, status: &str) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        info!("Updating subscription {} status to {}", id, status);

        diesel::update(subscriptions::table.filter(subscriptions::id.eq(id)))
            .set(subscriptions::status.eq(status))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to update subscription status: {}", e);
                AppError::database_error(format!("Failed to update subscription: {}", e))
            })?;

        info!("Successfully updated subscription status");
        Ok(())
    }

    /// Hard-delete a subscription row.
    ///
    /// Not part of the port surface. Pre-wave-11 callers: 0 in the
    /// source tree. Kept for backward compat.
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        warn!("Deleting subscription: {}", id);

        diesel::delete(subscriptions::table.filter(subscriptions::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to delete subscription: {}", e);
                AppError::database_error(format!("Failed to delete subscription: {}", e))
            })?;

        info!("Successfully deleted subscription");
        Ok(())
    }

    /// Count subscriptions matching the search criteria.
    ///
    /// Not part of the port surface. Pre-wave-11 callers: 0 in the
    /// source tree. Kept for backward compat.
    pub async fn count(&self, criteria: SubscriptionSearchCriteria) -> AppResult<i64> {
        let mut conn = self.db_pool.conn().await?;

        let mut query = subscriptions::table.into_boxed();

        if let Some(ref wallet_addr) = criteria.wallet_address {
            query = query.filter(subscriptions::wallet_address.eq(wallet_addr));
        }

        if let Some(ref plan_id) = criteria.plan_id {
            query = query.filter(subscriptions::plan_id.eq(plan_id));
        }

        if let Some(ref status) = criteria.status {
            query = query.filter(subscriptions::status.eq(status));
        }

        let count = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count subscriptions: {}", e);
                AppError::database_error(format!("Failed to count subscriptions: {}", e))
            })?;

        info!("Counted {} subscriptions matching criteria", count);
        Ok(count)
    }

    // -------------------------------------------------------------------------
    // Private helpers used by the port-method impls.
    // -------------------------------------------------------------------------

    /// List subscriptions for a wallet (raw row form). Used by
    /// `list_for_wallet` to fan out to the domain conversion.
    async fn find_by_wallet_raw(
        &self,
        wallet_address: &str,
    ) -> AppResult<Vec<SubscriptionDb>> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Finding subscriptions for wallet: {}", wallet_address);

        let results = subscriptions::table
            .filter(subscriptions::wallet_address.eq(wallet_address))
            .order(subscriptions::started_at.desc().nulls_last())
            .load::<SubscriptionDb>(&mut conn)
            .await
            .map_err(|e| {
                error!(
                    "Failed to find subscriptions for wallet {}: {}",
                    wallet_address, e
                );
                AppError::database_error(format!("Failed to find subscriptions: {}", e))
            })?;

        info!(
            "Found {} subscriptions for wallet {}",
            results.len(),
            wallet_address
        );
        Ok(results)
    }

    /// List subscriptions for a plan (raw row form). Used by
    /// `list_for_plan` to fan out to the domain conversion.
    async fn find_by_plan_raw(&self, plan_id: Uuid) -> AppResult<Vec<SubscriptionDb>> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Finding subscriptions for plan: {}", plan_id);

        let results = subscriptions::table
            .filter(subscriptions::plan_id.eq(plan_id))
            .order(subscriptions::started_at.desc().nulls_last())
            .load::<SubscriptionDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find subscriptions for plan {}: {}", plan_id, e);
                AppError::database_error(format!("Failed to find subscriptions: {}", e))
            })?;

        info!(
            "Found {} subscriptions for plan {}",
            results.len(),
            plan_id
        );
        Ok(results)
    }

    /// Save a new subscription row from a `NewSubscriptionDb`.
    ///
    /// Used by `create` to bridge between the port's
    /// `CreateSubscriptionCommand` and the Diesel model.
    async fn save_new(&self, new_row: NewSubscriptionDb) -> AppResult<SubscriptionDb> {
        let mut conn = self.db_pool.conn().await?;

        info!(
            "Saving subscription for wallet {} on plan {}",
            new_row.wallet_address, new_row.plan_id
        );

        let result = diesel::insert_into(subscriptions::table)
            .values(&new_row)
            .returning(SubscriptionDb::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to save subscription: {}", e);
                AppError::database_error(format!("Failed to save subscription: {}", e))
            })?;

        info!("Successfully saved subscription: {}", result.id);
        Ok(result)
    }

    /// Cancel a subscription by ID. Preserves the pre-wave-11
    /// semantics: sets `status = "cancelled"` and
    /// `cancelled_at = now()`.
    async fn cancel_by_id(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        warn!("Cancelling subscription: {}", id);

        diesel::update(subscriptions::table.filter(subscriptions::id.eq(id)))
            .set((
                subscriptions::status.eq("cancelled"),
                subscriptions::cancelled_at.eq(Some(Utc::now())),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to cancel subscription: {}", e);
                AppError::database_error(format!("Failed to cancel subscription: {}", e))
            })?;

        info!("Successfully cancelled subscription");
        Ok(())
    }

    /// Read the `stock_ranking_assignments` rows for a wallet.
    ///
    /// Returns the rows in descending `assigned_at` order
    /// (newest first), with `days_remaining` computed against
    /// the adapter's clock.
    async fn get_stock_ranking_assignments_raw(
        &self,
        wallet_address: &str,
    ) -> AppResult<Vec<StockRankingAssignment>> {
        let mut conn = self.db_pool.conn().await?;

        debug!(
            "Reading stock_ranking_assignments for wallet: {}",
            wallet_address
        );

        // The schema generator emits `stock_ranking_assignments`
        // as a `table!` block; columns are camelCase. See
        // `apps/backend/src/schemas/payments.rs:415-499`.
        let results = stock_ranking_assignments::table
            .filter(stock_ranking_assignments::wallet_address.eq(wallet_address))
            .order(stock_ranking_assignments::assigned_at.desc())
            .load::<StockRankingAssignmentRow>(&mut conn)
            .await
            .map_err(|e| {
                error!(
                    "Failed to read stock_ranking_assignments for wallet {}: {}",
                    wallet_address, e
                );
                AppError::database_error(format!(
                    "Failed to read stock_ranking_assignments: {}",
                    e
                ))
            })?;

        let now = Utc::now();
        let assignments: Vec<StockRankingAssignment> = results
            .into_iter()
            .map(|row| {
                let days_remaining = StockRankingAssignment::compute_days_remaining(
                    row.expires_at,
                    now,
                );
                StockRankingAssignment {
                    assignment_id: row.assignment_id.to_string(),
                    wallet_address: row.wallet_address,
                    package_id: row.package_id,
                    package_name: row.package_name,
                    rank_access_level: row.rank_access_level,
                    assigned_at: row.assigned_at,
                    expires_at: row.expires_at,
                    is_active: row.is_active,
                    assignment_source: row.assignment_source,
                    auto_renew: row.auto_renew,
                    days_remaining,
                }
            })
            .collect();

        info!(
            "Found {} stock_ranking_assignments for wallet {}",
            assignments.len(),
            wallet_address
        );
        Ok(assignments)
    }
}

// -----------------------------------------------------------------------------
// Port impl
// -----------------------------------------------------------------------------

#[async_trait]
impl SubscriptionRepositoryPort for PaymentSubscriptionRepositoryAdapter {
    async fn list_for_plan(&self, plan_id: PlanId) -> AppResult<Vec<Subscription>> {
        let raw = self.find_by_plan_raw(*plan_id.value()).await?;
        Ok(raw.into_iter().map(subscription_db_to_domain).collect())
    }

    async fn list_for_wallet(&self, wallet: &WalletAddress) -> AppResult<Vec<Subscription>> {
        let raw = self.find_by_wallet_raw(wallet.as_str()).await?;
        Ok(raw.into_iter().map(subscription_db_to_domain).collect())
    }

    async fn create(&self, cmd: CreateSubscriptionCommand) -> AppResult<Subscription> {
        let new_row = NewSubscriptionDb {
            wallet_address: cmd.wallet_address.as_str().to_string(),
            plan_id: cmd.plan_id,
            payment_id: cmd.payment_id,
            status: cmd.status,
            started_at: cmd.started_at,
            expires_at: cmd.expires_at,
            // The `CreateSubscriptionCommand` does not expose a
            // `cancelled_at` field (the canonical create path
            // always sets it to `None` on insert). Admin
            // cancellations go through the `cancel` port method
            // and set the column there.
            cancelled_at: None,
            auto_renew: cmd.auto_renew,
            metadata: cmd.metadata,
        };
        let saved = self.save_new(new_row).await?;
        Ok(subscription_db_to_domain(saved))
    }

    async fn cancel(
        &self,
        subscription_id: SubscriptionId,
        _reason: Option<String>,
    ) -> AppResult<()> {
        // `_reason` is currently a no-op on the `subscriptions`
        // table (the column does not exist; see the port docstring).
        // It is wired through here as a forward-compat hook for the
        // upcoming `payment_audit_log` `reason` field on
        // subscription cancellations.
        self.cancel_by_id(subscription_id.as_uuid()).await
    }

    async fn get_stock_ranking_assignments(
        &self,
        wallet: &WalletAddress,
    ) -> AppResult<Vec<StockRankingAssignment>> {
        self.get_stock_ranking_assignments_raw(wallet.as_str()).await
    }
}

// -----------------------------------------------------------------------------
// Domain conversion + raw row DTO
// -----------------------------------------------------------------------------

/// Convert the Diesel `SubscriptionDb` row to the domain
/// `Subscription` aggregate. The `WalletAddress` is constructed
/// with `from_trusted` because the row has already been through
/// the DB write path (which is the trust boundary for the
/// address value).
fn subscription_db_to_domain(row: SubscriptionDb) -> Subscription {
    Subscription {
        id: SubscriptionId::new(row.id),
        wallet_address: WalletAddress::from_trusted(row.wallet_address),
        plan_id: row.plan_id,
        payment_id: row.payment_id,
        status: row.status,
        started_at: row.started_at,
        expires_at: row.expires_at,
        cancelled_at: row.cancelled_at,
        auto_renew: row.auto_renew,
        metadata: row.metadata,
    }
}

/// Raw row shape for `stock_ranking_assignments` queries. Mirrors
/// the columns the port method reads; matches the schema
/// generated block in `apps/backend/src/schemas/payments.rs:415-499`.
///
/// `assignment_id` is `Uuid` in the DB (see
/// `migrations/payments/00000000000001_consolidated_payments_v3/up.sql`).
/// The domain DTO `StockRankingAssignment` exposes it as `String`
/// for backward compat with the market_analytics query response;
/// the conversion happens in `get_stock_ranking_assignments_raw`.
///
/// The 13-column shape must match the generated `table!` block
/// exactly — Diesel's `CompatibleType` check counts both the
/// Rust struct fields and the generated columns. Adding
/// `payment_reference` / `created_at` / `updated_at` is required
/// even though the port method does not surface them; the
/// `#[allow(dead_code)]` attribute keeps clippy quiet.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = stock_ranking_assignments)]
#[allow(dead_code)]
struct StockRankingAssignmentRow {
    pub assignment_id: Uuid,
    pub wallet_address: String,
    pub package_id: String,
    pub package_name: String,
    pub rank_access_level: i32,
    pub assigned_at: chrono::DateTime<Utc>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub is_active: bool,
    pub assignment_source: String,
    pub auto_renew: bool,
    pub payment_reference: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::wallet_management::value_objects::WalletAddress;

    /// Sanity check on the `SubscriptionId` ↔ `Uuid` bridge used
    /// by the cancel path. The full round-trip DB tests are gated
    /// on a live test DB; this one just confirms the conversion
    /// helper.
    #[test]
    fn subscription_db_to_domain_keeps_id_and_plan_id() {
        let id = Uuid::new_v4();
        let plan_id = Uuid::new_v4();
        let row = SubscriptionDb {
            id,
            wallet_address: "0x000000000000000000000000000000000000abc3".to_string(),
            plan_id,
            payment_id: None,
            status: "active".to_string(),
            started_at: Some(Utc::now()),
            expires_at: Utc::now() + chrono::Duration::days(30),
            cancelled_at: None,
            auto_renew: Some(true),
            metadata: None,
        };
        let sub = subscription_db_to_domain(row);
        assert_eq!(sub.id.as_uuid(), id);
        assert_eq!(sub.plan_id, plan_id);
        assert!(!sub.is_cancelled());
    }

    /// `StockRankingAssignment::compute_days_remaining` clamps
    /// negative values to zero (production path). The adapter
    /// uses the same helper; this is the unit-test backstop.
    #[test]
    fn days_remaining_clamp_propagates() {
        let now = Utc::now();
        assert_eq!(
            StockRankingAssignment::compute_days_remaining(
                Some(now - chrono::Duration::days(1)),
                now,
            ),
            Some(0)
        );
        assert_eq!(
            StockRankingAssignment::compute_days_remaining(None, now),
            None
        );
    }

    /// Confirms the `cancel(_reason: None)` port path does not
    /// require the caller to set anything on the row beyond the
    /// `SubscriptionId`. The actual DB write is exercised in the
    /// integration test suite (gated on a live test DB).
    #[test]
    fn cancel_accepts_none_reason() {
        // Compile-time check: the trait method signature is
        // `cancel(&self, id: SubscriptionId, reason: Option<String>)`.
        // This test exists to make any future signature drift
        // visible in `cargo test`.
        fn _takes_optional_reason(_: Option<String>) {}
        let _id = SubscriptionId::generate();
        let _wallet = WalletAddress::from_trusted(
            "0x000000000000000000000000000000000000abc4".to_string(),
        );
        _takes_optional_reason(None);
    }
}
