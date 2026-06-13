//! Wave 11 / Track A — Cross-Pool Method Implementations
//!
//! This file adds the 8 new methods on `PaymentRepositoryAdapter`
//! that the cross-pool handler collapse in `web/payments/*` now
//! goes through. The methods are kept in a separate file from
//! `payment_repository_adapter.rs` (the existing 506-LOC port
//! implementation) so the diff stays surgical and the new code
//! is easy to find.
//!
//! ## Why all the methods talk to the payments pool only
//!
//! The wave-8 payments audit (audit-payments.md §4) flagged that
//! the two pools (`payments_pool` and the primary pool) today
//! are **the same Postgres database** in production — the
//! `PAYMENTS_DATABASE_URL` env var is unset in
//! `infrastructure/docker/.env.prod`, so the connection
//! manager falls back to the primary pool. After the wave-11
//! integration gate sets the env var to a dedicated
//! `epsx_payments_prod` database, the in-process impl still
//! runs against the `db_pool` (the "payments" pool in
//! production) but the `plans` table is replicated into the
//! payments schema by the wave-11 cutover SQL (integration
//! gate step 4). The JOIN `payments ⋈ plans` therefore becomes
//! a true single-pool query.
//!
//! Until the cutover lands, the JOINs in this file resolve the
//! `plans` row from the same Postgres database (because the two
//! pools share a schema today — the Diesel-side split is
//! `diesel_payments.toml` vs `diesel.toml`, which controls
//! which `table!` macros are generated in the `schemas::`
//! modules, but the SQL tables are still in the default
//! `public` schema). This is safe and is exactly the "one
//! cross-pool method" the audit's Refactor #1 talks about —
//! the future HTTP impl of the trait will serve the same
//! shape from a single pool without any cross-pool reacharound.
//!
//! ## Why these methods use `diesel::sql_query` (not typed JOINs)
//!
//! `diesel::table!` generates one module per table; Diesel
//! type-checks joins with `TableNotEqual` and rejects
//! cross-schema joins between two different `table!` modules.
//! `payments` and `plans` are two such modules (in
//! `schemas::payments` and `schemas::primary` respectively).
//! To avoid a type-system fight with Diesel — and to match the
//! `#[derive(QueryableByName)]` style the existing
//! `web/payments/admin_handlers/analytics_handlers.rs`
//! already uses — the cross-pool JOIN methods below use
//! `diesel::sql_query` with manually-written SELECT
//! statements. The typed-query-builder path is reserved for
//! the pure single-table methods (which stay in the original
//! `payment_repository_adapter.rs`).
//!
//! ## Plan-name enrichment
//!
//! Every `*_with_plan_name` method returns `(T, Option<String>)`
//! where the `Option<String>` is the plan name. The SQL
//! JOIN uses `LEFT JOIN plans ON payments.plan_id = plans.id`
//! so payments whose plan row has been deleted (admin hard
//! delete) still come back, with `plan_name = None`.

use crate::prelude::*;
use diesel::prelude::*;
use diesel::result::OptionalExtension;
use diesel_async::RunQueryDsl;
use bigdecimal::{BigDecimal, ToPrimitive};
use std::str::FromStr;
use chrono::{DateTime, Utc, Datelike};
use uuid::Uuid;
use tracing::debug;

use crate::domain::payment::{
    Payment, PaymentId, PaymentStatus, PaymentAmount, PaymentReference, TransactionHash,
};
use crate::domain::payment::repository_ports::{
    PaymentRepositoryPort, Subscription, AnalyticsRollup, AnalyticsWindow,
    DailyRevenueEntry, PlanBreakdownEntry, PaymentMethodEntry, AnalyticsTrends,
    SubmitTxValidation, CreatePaymentCommand, ActivateSubscriptionCommand,
    SubscriptionFilters,
};
use crate::domain::wallet_management::value_objects::WalletAddress;
use crate::infrastructure::models::payment::{PaymentDb, NewPaymentDb, SubscriptionDb};
use crate::schemas::payments::{payments, subscriptions};

use super::payment_repository_adapter::PaymentRepositoryAdapter;

// ============================================================================
// Inherent `_*_impl` methods on PaymentRepositoryAdapter
// ============================================================================
//
// These are the new methods (8 + helpers). The trait impl at
// the bottom of the file forward-calls them.

impl PaymentRepositoryAdapter {
    // -- Shared helpers -------------------------------------------------

    /// Build a domain `Payment` from a `PaymentDb` row.
    /// Duplicates the original `payment_to_domain` (which is
    /// `fn`, not `pub`) so this file can be a self-contained
    /// port-extension. Kept in sync — the original
    /// implementation is the source of truth; if the
    /// `Currency::USD` fallback or the `PaymentStatus`
    /// match arms change, this helper must change too. The
    /// duplication is one file; the refactor is local.
    pub(crate) fn row_to_domain(&self, payment_db: PaymentDb) -> Result<Payment, String> {
        let amount_decimal = rust_decimal::Decimal::from_str(&payment_db.amount.to_string())
            .unwrap_or(rust_decimal::Decimal::ZERO);
        let currency = match payment_db.currency.as_str() {
            "USD" => crate::domain::payment::value_objects::Currency::USD,
            "USDT" => crate::domain::payment::value_objects::Currency::USDT,
            "USDC" => crate::domain::payment::value_objects::Currency::USDC,
            "ETH" => crate::domain::payment::value_objects::Currency::ETH,
            "BTC" => crate::domain::payment::value_objects::Currency::BTC,
            "BNB" => crate::domain::payment::value_objects::Currency::BNB,
            "TRX" => crate::domain::payment::value_objects::Currency::TRX,
            _ => crate::domain::payment::value_objects::Currency::USD,
        };
        let amount = PaymentAmount::new(amount_decimal, currency)
            .map_err(|e| format!("Invalid payment amount: {}", e))?;
        let payment_id = PaymentId::from_uuid(payment_db.id);
        let payment_reference = PaymentReference::from_string(&payment_db.payment_reference)
            .map_err(|e| format!("Invalid payment reference: {}", e))?;
        let transaction_hash = payment_db.transaction_hash.clone()
            .map(|hash| TransactionHash::new(hash, crate::domain::payment::value_objects::Network::BinanceSmartChain))
            .transpose()
            .map_err(|e| format!("Invalid transaction hash: {}", e))?;
        let status = match payment_db.status.as_str() {
            "created" => PaymentStatus::Created,
            "awaiting_payment" | "awaiting" | "pending" => PaymentStatus::AwaitingPayment,
            "pending_verification" => PaymentStatus::PendingVerification,
            "verifying" => PaymentStatus::Verifying,
            "verification_failed" => PaymentStatus::VerificationFailed,
            "confirmed" => PaymentStatus::Confirmed,
            "processing" => PaymentStatus::Processing,
            "completed" => PaymentStatus::Completed,
            "failed" => PaymentStatus::Failed,
            "cancelled" => PaymentStatus::Cancelled,
            "refunding" => PaymentStatus::Refunding,
            "refunded" => PaymentStatus::Refunded,
            _ => return Err(format!("Invalid payment status: {}", payment_db.status)),
        };
        let wallet_address = WalletAddress::new(&payment_db.wallet_address)
            .map_err(|e| format!("Invalid wallet address: {}", e))?;
        let created_at = payment_db.created_at.unwrap_or_else(Utc::now);
        Payment::new(
            payment_id,
            payment_reference,
            wallet_address,
            amount,
            status,
            transaction_hash,
            payment_db.plan_id.to_string(),
            created_at,
            payment_db.metadata.clone().unwrap_or(serde_json::json!({})),
        )
        .map_err(|e| format!("Failed to create payment aggregate: {}", e))
    }

    fn sub_row_to_domain(&self, sub: SubscriptionDb) -> Subscription {
        Subscription {
            id: sub.id,
            wallet_address: sub.wallet_address,
            plan_id: sub.plan_id,
            payment_id: sub.payment_id,
            status: sub.status,
            started_at: sub.started_at,
            expires_at: sub.expires_at,
            cancelled_at: sub.cancelled_at,
            auto_renew: sub.auto_renew.unwrap_or(false),
            metadata: sub.metadata.unwrap_or(serde_json::json!({})),
        }
    }

    // -- 8 new port methods ---------------------------------------------

    /// Replaces `web/payments/get_tx_status_handler.rs:121-137`.
    /// Single LEFT JOIN query against `payments ⋈ plans`.
    pub async fn get_tx_status_with_plan_name_impl(
        &self,
        tx_hash: &str,
    ) -> Result<Option<(Payment, Option<String>)>, String> {
        let mut conn = self.conn().await?;
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            payment_reference: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            transaction_hash: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            amount: BigDecimal,
            #[diesel(sql_type = diesel::sql_types::Text)]
            currency: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            method: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            plan_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            contract_address: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            token_address: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            block_number: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
            confirmations: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            created_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            updated_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            expires_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            completed_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            metadata: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            last_checked_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            error_message: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            network: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            plan_name: Option<String>,
        }
        let sql = r#"
            SELECT
                p.id, p.payment_reference, p.transaction_hash, p.wallet_address,
                p.amount, p.currency, p.method, p.status, p.plan_id,
                p.contract_address, p.token_address, p.block_number, p.confirmations,
                p.created_at, p.updated_at, p.expires_at, p.completed_at,
                p.metadata, p.last_checked_at, p.error_message, p.network,
                pl.name as plan_name
            FROM payments p
            LEFT JOIN plans pl ON p.plan_id = pl.id
            WHERE p.transaction_hash = $1
            LIMIT 1
        "#;
        let row: Option<Row> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Text, _>(tx_hash)
            .get_result::<Row>(&mut conn)
            .await
            .optional()
            .map_err(|e| format!("get_tx_status: {}", e))?;
        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };
        let db = PaymentDb {
            id: row.id,
            payment_reference: row.payment_reference,
            transaction_hash: row.transaction_hash,
            wallet_address: row.wallet_address,
            amount: row.amount,
            currency: row.currency,
            method: row.method,
            status: row.status,
            plan_id: row.plan_id,
            contract_address: row.contract_address,
            token_address: row.token_address,
            block_number: row.block_number,
            confirmations: row.confirmations,
            created_at: row.created_at,
            updated_at: row.updated_at,
            expires_at: row.expires_at,
            completed_at: row.completed_at,
            metadata: row.metadata,
            last_checked_at: row.last_checked_at,
            error_message: row.error_message,
            network: row.network,
        };
        let payment = self.row_to_domain(db)?;
        Ok(Some((payment, row.plan_name)))
    }

    /// Replaces `web/payments/user_payment_handlers.rs:144-166`.
    /// Single LEFT JOIN query — the N+1 fix. The
    /// `tests::n_plus_one_user_payments` test pins the query
    /// count to 1 for a 50-row page.
    pub async fn list_user_payments_with_plan_names_impl(
        &self,
        wallet_address: &WalletAddress,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<(Payment, Option<String>)>, String> {
        let per_page = per_page.clamp(1, 50);
        let page = page.max(1);
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let mut conn = self.conn().await?;
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)] payment_reference: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] transaction_hash: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)] wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Numeric)] amount: BigDecimal,
            #[diesel(sql_type = diesel::sql_types::Text)] currency: String,
            #[diesel(sql_type = diesel::sql_types::Text)] method: String,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
            #[diesel(sql_type = diesel::sql_types::Uuid)] plan_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] contract_address: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] token_address: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)] block_number: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)] confirmations: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] created_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] updated_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] expires_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] completed_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)] metadata: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] last_checked_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] error_message: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] network: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] plan_name: Option<String>,
        }
        let sql = r#"
            SELECT
                p.id, p.payment_reference, p.transaction_hash, p.wallet_address,
                p.amount, p.currency, p.method, p.status, p.plan_id,
                p.contract_address, p.token_address, p.block_number, p.confirmations,
                p.created_at, p.updated_at, p.expires_at, p.completed_at,
                p.metadata, p.last_checked_at, p.error_message, p.network,
                pl.name as plan_name
            FROM payments p
            LEFT JOIN plans pl ON p.plan_id = pl.id
            WHERE p.wallet_address = $1
            ORDER BY p.created_at DESC NULLS LAST
            LIMIT $2 OFFSET $3
        "#;
        let rows: Vec<Row> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Text, _>(wallet_address.as_str())
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<Row>(&mut conn)
            .await
            .map_err(|e| format!("list_user_payments: {}", e))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let db = PaymentDb {
                id: r.id,
                payment_reference: r.payment_reference,
                transaction_hash: r.transaction_hash,
                wallet_address: r.wallet_address,
                amount: r.amount,
                currency: r.currency,
                method: r.method,
                status: r.status,
                plan_id: r.plan_id,
                contract_address: r.contract_address,
                token_address: r.token_address,
                block_number: r.block_number,
                confirmations: r.confirmations,
                created_at: r.created_at,
                updated_at: r.updated_at,
                expires_at: r.expires_at,
                completed_at: r.completed_at,
                metadata: r.metadata,
                last_checked_at: r.last_checked_at,
                error_message: r.error_message,
                network: r.network,
            };
            out.push((self.row_to_domain(db)?, r.plan_name));
        }
        Ok(out)
    }

    /// Replaces `web/payments/admin_handlers/payment_handlers.rs:249-270`.
    pub async fn get_admin_payment_details_with_plan_name_impl(
        &self,
        payment_id: PaymentId,
    ) -> Result<Option<(Payment, Option<String>)>, String> {
        let mut conn = self.conn().await?;
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)] payment_reference: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] transaction_hash: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)] wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Numeric)] amount: BigDecimal,
            #[diesel(sql_type = diesel::sql_types::Text)] currency: String,
            #[diesel(sql_type = diesel::sql_types::Text)] method: String,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
            #[diesel(sql_type = diesel::sql_types::Uuid)] plan_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] contract_address: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] token_address: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)] block_number: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)] confirmations: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] created_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] updated_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] expires_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] completed_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)] metadata: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] last_checked_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] error_message: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] network: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] plan_name: Option<String>,
        }
        let sql = r#"
            SELECT
                p.id, p.payment_reference, p.transaction_hash, p.wallet_address,
                p.amount, p.currency, p.method, p.status, p.plan_id,
                p.contract_address, p.token_address, p.block_number, p.confirmations,
                p.created_at, p.updated_at, p.expires_at, p.completed_at,
                p.metadata, p.last_checked_at, p.error_message, p.network,
                pl.name as plan_name
            FROM payments p
            LEFT JOIN plans pl ON p.plan_id = pl.id
            WHERE p.id = $1
            LIMIT 1
        "#;
        let row: Option<Row> = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(payment_id.value())
            .get_result::<Row>(&mut conn)
            .await
            .optional()
            .map_err(|e| format!("admin_payment_details: {}", e))?;
        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };
        let db = PaymentDb {
            id: row.id,
            payment_reference: row.payment_reference,
            transaction_hash: row.transaction_hash,
            wallet_address: row.wallet_address,
            amount: row.amount,
            currency: row.currency,
            method: row.method,
            status: row.status,
            plan_id: row.plan_id,
            contract_address: row.contract_address,
            token_address: row.token_address,
            block_number: row.block_number,
            confirmations: row.confirmations,
            created_at: row.created_at,
            updated_at: row.updated_at,
            expires_at: row.expires_at,
            completed_at: row.completed_at,
            metadata: row.metadata,
            last_checked_at: row.last_checked_at,
            error_message: row.error_message,
            network: row.network,
        };
        let payment = self.row_to_domain(db)?;
        Ok(Some((payment, row.plan_name)))
    }

    /// Replaces `web/payments/admin_handlers/subscription_handlers.rs:32-101`.
    pub async fn list_admin_subscriptions_with_plan_names_impl(
        &self,
        filters: SubscriptionFilters,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<(Subscription, Option<String>)>, String> {
        let per_page = per_page.clamp(1, 200);
        let page = page.max(1);
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let mut conn = self.conn().await?;
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)] wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Uuid)] plan_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] payment_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] started_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] expires_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] cancelled_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)] auto_renew: Option<bool>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)] metadata: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] plan_name: Option<String>,
        }
        // Branch on which filters are set. 8 combinations of
        // 3 optional filters. This sidesteps Diesel's typestate
        // builder (which would force 8 different static SQL
        // strings anyway). The handler calls it with at most
        // 2 filters in practice.
        let rows: Vec<Row> = match (
            filters.wallet_address.as_ref(),
            filters.plan_id,
            filters.status.as_ref(),
        ) {
            (Some(w), Some(p), Some(s)) => diesel::sql_query(
                "SELECT s.id, s.wallet_address, s.plan_id, s.payment_id, s.status, s.started_at, s.expires_at, s.cancelled_at, s.auto_renew, s.metadata, pl.name as plan_name \
                 FROM subscriptions s LEFT JOIN plans pl ON s.plan_id = pl.id \
                 WHERE s.wallet_address = $1 AND s.plan_id = $2 AND s.status = $3 \
                 ORDER BY s.started_at DESC NULLS LAST LIMIT $4 OFFSET $5",
            )
            .bind::<diesel::sql_types::Text, _>(w)
            .bind::<diesel::sql_types::Uuid, _>(p)
            .bind::<diesel::sql_types::Text, _>(s)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<Row>(&mut conn).await.map_err(|e| format!("list_admin_subs (3f): {}", e))?,
            (Some(w), Some(p), None) => diesel::sql_query(
                "SELECT s.id, s.wallet_address, s.plan_id, s.payment_id, s.status, s.started_at, s.expires_at, s.cancelled_at, s.auto_renew, s.metadata, pl.name as plan_name \
                 FROM subscriptions s LEFT JOIN plans pl ON s.plan_id = pl.id \
                 WHERE s.wallet_address = $1 AND s.plan_id = $2 \
                 ORDER BY s.started_at DESC NULLS LAST LIMIT $3 OFFSET $4",
            )
            .bind::<diesel::sql_types::Text, _>(w)
            .bind::<diesel::sql_types::Uuid, _>(p)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<Row>(&mut conn).await.map_err(|e| format!("list_admin_subs (wp): {}", e))?,
            (Some(w), None, Some(s)) => diesel::sql_query(
                "SELECT s.id, s.wallet_address, s.plan_id, s.payment_id, s.status, s.started_at, s.expires_at, s.cancelled_at, s.auto_renew, s.metadata, pl.name as plan_name \
                 FROM subscriptions s LEFT JOIN plans pl ON s.plan_id = pl.id \
                 WHERE s.wallet_address = $1 AND s.status = $2 \
                 ORDER BY s.started_at DESC NULLS LAST LIMIT $3 OFFSET $4",
            )
            .bind::<diesel::sql_types::Text, _>(w)
            .bind::<diesel::sql_types::Text, _>(s)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<Row>(&mut conn).await.map_err(|e| format!("list_admin_subs (ws): {}", e))?,
            (None, Some(p), Some(s)) => diesel::sql_query(
                "SELECT s.id, s.wallet_address, s.plan_id, s.payment_id, s.status, s.started_at, s.expires_at, s.cancelled_at, s.auto_renew, s.metadata, pl.name as plan_name \
                 FROM subscriptions s LEFT JOIN plans pl ON s.plan_id = pl.id \
                 WHERE s.plan_id = $1 AND s.status = $2 \
                 ORDER BY s.started_at DESC NULLS LAST LIMIT $3 OFFSET $4",
            )
            .bind::<diesel::sql_types::Uuid, _>(p)
            .bind::<diesel::sql_types::Text, _>(s)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<Row>(&mut conn).await.map_err(|e| format!("list_admin_subs (ps): {}", e))?,
            (Some(w), None, None) => diesel::sql_query(
                "SELECT s.id, s.wallet_address, s.plan_id, s.payment_id, s.status, s.started_at, s.expires_at, s.cancelled_at, s.auto_renew, s.metadata, pl.name as plan_name \
                 FROM subscriptions s LEFT JOIN plans pl ON s.plan_id = pl.id \
                 WHERE s.wallet_address = $1 \
                 ORDER BY s.started_at DESC NULLS LAST LIMIT $2 OFFSET $3",
            )
            .bind::<diesel::sql_types::Text, _>(w)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<Row>(&mut conn).await.map_err(|e| format!("list_admin_subs (w): {}", e))?,
            (None, Some(p), None) => diesel::sql_query(
                "SELECT s.id, s.wallet_address, s.plan_id, s.payment_id, s.status, s.started_at, s.expires_at, s.cancelled_at, s.auto_renew, s.metadata, pl.name as plan_name \
                 FROM subscriptions s LEFT JOIN plans pl ON s.plan_id = pl.id \
                 WHERE s.plan_id = $1 \
                 ORDER BY s.started_at DESC NULLS LAST LIMIT $2 OFFSET $3",
            )
            .bind::<diesel::sql_types::Uuid, _>(p)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<Row>(&mut conn).await.map_err(|e| format!("list_admin_subs (p): {}", e))?,
            (None, None, Some(s)) => diesel::sql_query(
                "SELECT s.id, s.wallet_address, s.plan_id, s.payment_id, s.status, s.started_at, s.expires_at, s.cancelled_at, s.auto_renew, s.metadata, pl.name as plan_name \
                 FROM subscriptions s LEFT JOIN plans pl ON s.plan_id = pl.id \
                 WHERE s.status = $1 \
                 ORDER BY s.started_at DESC NULLS LAST LIMIT $2 OFFSET $3",
            )
            .bind::<diesel::sql_types::Text, _>(s)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<Row>(&mut conn).await.map_err(|e| format!("list_admin_subs (s): {}", e))?,
            (None, None, None) => diesel::sql_query(
                "SELECT s.id, s.wallet_address, s.plan_id, s.payment_id, s.status, s.started_at, s.expires_at, s.cancelled_at, s.auto_renew, s.metadata, pl.name as plan_name \
                 FROM subscriptions s LEFT JOIN plans pl ON s.plan_id = pl.id \
                 ORDER BY s.started_at DESC NULLS LAST LIMIT $1 OFFSET $2",
            )
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<Row>(&mut conn).await.map_err(|e| format!("list_admin_subs (_): {}", e))?,
        };

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let sub_db = SubscriptionDb {
                id: r.id,
                wallet_address: r.wallet_address,
                plan_id: r.plan_id,
                payment_id: r.payment_id,
                status: r.status,
                started_at: r.started_at,
                expires_at: r.expires_at,
                cancelled_at: r.cancelled_at,
                auto_renew: r.auto_renew,
                metadata: r.metadata,
            };
            out.push((self.sub_row_to_domain(sub_db), r.plan_name));
        }
        Ok(out)
    }

    /// Replaces the 4 sql_query blocks in
    /// `web/payments/admin_handlers/analytics_handlers.rs:39-44`.
    pub async fn get_analytics_rollup_impl(
        &self,
        window: AnalyticsWindow,
    ) -> Result<AnalyticsRollup, String> {
        let mut conn = self.conn().await?;

        let now = Utc::now();
        let since = match window {
            AnalyticsWindow::Last30Days => now - chrono::Duration::days(30),
            AnalyticsWindow::Last7Days => now - chrono::Duration::days(7),
            AnalyticsWindow::Last24Hours => now - chrono::Duration::hours(24),
            AnalyticsWindow::MonthToDate => {
                let month_start = now.date_naive()
                    .with_day(1)
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                    .unwrap_or(now);
                month_start
            }
        };

        // 1. Daily revenue
        #[derive(QueryableByName)]
        struct DailyRevenueRow {
            #[diesel(sql_type = diesel::sql_types::Date)]
            payment_date: chrono::NaiveDate,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
            daily_revenue: Option<BigDecimal>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            payment_count: i64,
        }
        let daily_rows = diesel::sql_query(
            r#"
            SELECT
                DATE(created_at) as payment_date,
                SUM(amount) as daily_revenue,
                COUNT(*) as payment_count
            FROM payments
            WHERE created_at >= $1
              AND (status = 'completed' OR status = 'confirmed')
            GROUP BY DATE(created_at)
            ORDER BY payment_date DESC
            LIMIT 30
            "#,
        )
        .bind::<diesel::sql_types::Timestamptz, _>(since)
        .load::<DailyRevenueRow>(&mut conn)
        .await
        .map_err(|e| format!("daily: {}", e))?;
        let daily_revenue: Vec<DailyRevenueEntry> = daily_rows.into_iter().map(|r| DailyRevenueEntry {
            date: r.payment_date.format("%Y-%m-%d").to_string(),
            revenue: r.daily_revenue.map(|bd| bd.to_f64().unwrap_or(0.0)).unwrap_or(0.0),
            payment_count: r.payment_count as u32,
        }).collect();

        // 2. Plan breakdown — JOIN plans inline
        #[derive(QueryableByName)]
        struct PlanBreakdownRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            plan_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
            total_revenue: Option<BigDecimal>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            subscription_count: i64,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            plan_name: Option<String>,
        }
        let plan_rows = diesel::sql_query(
            r#"
            SELECT
                p.plan_id as plan_id,
                SUM(p.amount) as total_revenue,
                COUNT(*) as subscription_count,
                pl.name as plan_name
            FROM payments p
            LEFT JOIN plans pl ON p.plan_id = pl.id
            WHERE p.status = 'completed' OR p.status = 'confirmed'
            GROUP BY p.plan_id, pl.name
            ORDER BY total_revenue DESC NULLS LAST
            LIMIT 10
            "#,
        )
        .load::<PlanBreakdownRow>(&mut conn)
        .await
        .map_err(|e| format!("plan: {}", e))?;
        let plan_breakdown: Vec<PlanBreakdownEntry> = plan_rows.into_iter().map(|r| {
            let revenue = r.total_revenue.map(|bd| bd.to_f64().unwrap_or(0.0)).unwrap_or(0.0);
            let count = r.subscription_count as u32;
            let arpu = if count > 0 { revenue / count as f64 } else { 0.0 };
            PlanBreakdownEntry {
                plan_id: r.plan_id,
                plan_name: r.plan_name.unwrap_or_else(|| "Unknown Plan".to_string()),
                total_revenue: revenue,
                subscription_count: count,
                average_revenue_per_user: arpu,
            }
        }).collect();

        // 3. Payment methods
        #[derive(QueryableByName)]
        struct PaymentMethodRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            currency: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            payment_count: i64,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
            total_revenue: Option<BigDecimal>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            successful_count: i64,
        }
        let method_rows = diesel::sql_query(
            r#"
            SELECT
                currency,
                COUNT(*) as payment_count,
                SUM(CASE WHEN status IN ('completed', 'confirmed') THEN amount ELSE 0 END) as total_revenue,
                SUM(CASE WHEN status IN ('completed', 'confirmed') THEN 1 ELSE 0 END) as successful_count
            FROM payments
            GROUP BY currency
            ORDER BY payment_count DESC
            "#,
        )
        .load::<PaymentMethodRow>(&mut conn)
        .await
        .map_err(|e| format!("method: {}", e))?;
        let payment_methods: Vec<PaymentMethodEntry> = method_rows.into_iter().map(|r| {
            let total = r.payment_count as f64;
            let success = r.successful_count as f64;
            PaymentMethodEntry {
                currency: r.currency,
                payment_count: r.payment_count as u32,
                total_revenue: r.total_revenue.map(|bd| bd.to_f64().unwrap_or(0.0)).unwrap_or(0.0),
                success_rate: if total > 0.0 { (success / total) * 100.0 } else { 0.0 },
            }
        }).collect();

        // 4. Trends
        #[derive(QueryableByName)]
        struct PeriodStats {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
            total_revenue: Option<BigDecimal>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_count: i64,
        }
        let current_period = diesel::sql_query(
            r#"
            SELECT COALESCE(SUM(amount), 0) as total_revenue, COUNT(*) as total_count
            FROM payments
            WHERE created_at >= $1 AND (status = 'completed' OR status = 'confirmed')
            "#,
        )
        .bind::<diesel::sql_types::Timestamptz, _>(since)
        .get_result::<PeriodStats>(&mut conn)
        .await
        .map_err(|e| format!("current: {}", e))?;
        let previous_period = diesel::sql_query(
            r#"
            SELECT COALESCE(SUM(amount), 0) as total_revenue, COUNT(*) as total_count
            FROM payments
            WHERE created_at >= $1 AND created_at < $2 AND (status = 'completed' OR status = 'confirmed')
            "#,
        )
        .bind::<diesel::sql_types::Timestamptz, _>(since - (now - since))
        .bind::<diesel::sql_types::Timestamptz, _>(since)
        .get_result::<PeriodStats>(&mut conn)
        .await
        .map_err(|e| format!("previous: {}", e))?;
        let current_rev = current_period.total_revenue.map(|bd| bd.to_f64().unwrap_or(0.0)).unwrap_or(0.0);
        let previous_rev = previous_period.total_revenue.map(|bd| bd.to_f64().unwrap_or(0.0)).unwrap_or(0.0);
        let growth_rate = if previous_rev > 0.0 {
            ((current_rev - previous_rev) / previous_rev) * 100.0
        } else if current_rev > 0.0 {
            100.0
        } else {
            0.0
        };

        #[derive(QueryableByName)]
        struct SubMetrics {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Float8>)]
            avg_sub_length: Option<f64>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            cancelled_count: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            active_count: i64,
        }
        let sub_metrics = diesel::sql_query(
            r#"
            SELECT
                AVG(EXTRACT(EPOCH FROM (expires_at - started_at)) / 86400.0) FILTER (WHERE status = 'active') as avg_sub_length,
                COUNT(*) FILTER (WHERE status = 'cancelled' AND cancelled_at >= $1) as cancelled_count,
                COUNT(*) FILTER (WHERE status = 'active') as active_count
            FROM subscriptions
            "#,
        )
        .bind::<diesel::sql_types::Timestamptz, _>(since)
        .get_result::<SubMetrics>(&mut conn)
        .await
        .map_err(|e| format!("sub_metrics: {}", e))?;
        let avg_sub_length = sub_metrics.avg_sub_length.unwrap_or(30.0);
        let cancelled_count = sub_metrics.cancelled_count;
        let active_count = sub_metrics.active_count.max(1);
        let churn_rate = (cancelled_count as f64 / active_count as f64) * 100.0;
        let avg_payment = if current_period.total_count > 0 {
            current_rev / current_period.total_count as f64
        } else {
            0.0
        };
        let clv = avg_payment * (avg_sub_length / 30.0);
        let trends = AnalyticsTrends {
            growth_rate,
            churn_rate,
            average_subscription_length: avg_sub_length,
            customer_lifetime_value: clv,
        };

        Ok(AnalyticsRollup {
            daily_revenue,
            plan_breakdown,
            payment_methods,
            trends,
        })
    }

    /// Replaces `web/payments/submit_tx_handler.rs:158-184` +
    /// `285-294`. Reads plans (cross-schema via SQL) and
    /// wallet_credits (payments schema) in one call.
    pub async fn validate_submit_tx_impl(
        &self,
        plan_id: Uuid,
        wallet_address: &WalletAddress,
    ) -> Result<SubmitTxValidation, String> {
        let mut conn = self.conn().await?;

        #[derive(QueryableByName)]
        struct PlanRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
            price: Option<BigDecimal>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
            #[diesel(sql_type = diesel::sql_types::Text)]
            plan_type: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            plan_metadata: Option<serde_json::Value>,
        }
        let plan: PlanRow = diesel::sql_query(
            "SELECT COALESCE(price, 0) as price, is_active, COALESCE(plan_type, 'subscription') as plan_type, COALESCE(plan_metadata, '{}'::jsonb) as plan_metadata FROM plans WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(plan_id)
        .get_result::<PlanRow>(&mut conn)
        .await
        .optional()
        .map_err(|e| format!("plan: {}", e))?
        .ok_or_else(|| "Plan not found".to_string())?;

        #[derive(QueryableByName)]
        struct BalanceRow {
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            bal: BigDecimal,
        }
        let balance: BigDecimal = diesel::sql_query(
            "SELECT COALESCE((SELECT balance FROM wallet_credits WHERE wallet_address = $1), 0) as bal",
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address.as_str())
        .get_result::<BalanceRow>(&mut conn)
        .await
        .map(|r| r.bal)
        .unwrap_or_else(|_| BigDecimal::from(0));

        let effective_price = plan.plan_metadata.as_ref()
            .and_then(|m| m.get("promotion"))
            .and_then(|p| serde_json::from_value::<crate::domain::subscription_management::promotion::Promotion>(p.clone()).ok())
            .map(|promo| {
                let bp = plan.price.as_ref().map(|p| p.to_f64().unwrap_or(0.0)).unwrap_or(0.0);
                let ep = promo.calculate_effective_price(bp);
                BigDecimal::from_str(&format!("{:.2}", ep))
                    .unwrap_or_else(|_| plan.price.clone().unwrap_or(BigDecimal::from(0)))
            })
            .or_else(|| plan.price.clone())
            .unwrap_or(BigDecimal::from(0));

        Ok(SubmitTxValidation {
            plan_price: plan.price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string()),
            is_active: plan.is_active,
            plan_type: plan.plan_type,
            plan_metadata: plan.plan_metadata.unwrap_or(serde_json::json!({})),
            effective_price: effective_price.to_string(),
        })
    }

    /// Replaces the inline `INSERT INTO payments` blocks in
    /// `web/payments/submit_tx_handler.rs:339-414`. Returns the
    /// newly-created `Payment` aggregate.
    pub async fn create_payment_impl(
        &self,
        cmd: CreatePaymentCommand,
    ) -> Result<Payment, String> {
        let amount = BigDecimal::from_str(&cmd.amount)
            .map_err(|e| format!("amount: {}", e))?;
        let mut conn = self.conn().await?;

        let new = NewPaymentDb {
            payment_reference: cmd.payment_reference.clone(),
            wallet_address: cmd.wallet_address.clone(),
            amount: amount.clone(),
            currency: cmd.currency.clone(),
            method: cmd.method.clone(),
            status: cmd.status.clone(),
            plan_id: cmd.plan_id,
            contract_address: None,
            token_address: None,
            block_number: None,
            confirmations: Some(0),
            expires_at: cmd.expires_at,
            metadata: cmd.metadata.clone().unwrap_or(serde_json::json!({})),
        };
        let mut complete: PaymentDb = diesel::insert_into(payments::table)
            .values(&new)
            .returning(PaymentDb::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| format!("insert: {}", e))?;

        if let Some(tx) = cmd.transaction_hash.clone() {
            diesel::update(payments::table.filter(payments::id.eq(complete.id)))
                .set(payments::transaction_hash.eq(Some(tx)))
                .execute(&mut conn)
                .await
                .map_err(|e| format!("update tx_hash: {}", e))?;
            complete.transaction_hash = cmd.transaction_hash;
        }
        if let Some(net) = cmd.network.clone() {
            diesel::update(payments::table.filter(payments::id.eq(complete.id)))
                .set(payments::network.eq(Some(net)))
                .execute(&mut conn)
                .await
                .map_err(|e| format!("update network: {}", e))?;
            complete.network = cmd.network;
        }
        if let Some(c) = cmd.completed_at {
            diesel::update(payments::table.filter(payments::id.eq(complete.id)))
                .set(payments::completed_at.eq(Some(c)))
                .execute(&mut conn)
                .await
                .map_err(|e| format!("update completed_at: {}", e))?;
            complete.completed_at = Some(c);
        }
        self.row_to_domain(complete)
    }

    /// Replaces the inline `UPDATE payments SET status = ...` blocks.
    pub async fn update_payment_status_impl(
        &self,
        payment_id: PaymentId,
        new_status: PaymentStatus,
        audit_note: Option<String>,
    ) -> Result<(), String> {
        let mut conn = self.conn().await?;
        let status_str = new_status.as_str();
        let completed_at = match new_status {
            PaymentStatus::Completed | PaymentStatus::Failed | PaymentStatus::Refunded => Some(Utc::now()),
            _ => None,
        };
        diesel::update(payments::table.filter(payments::id.eq(payment_id.value())))
            .set((
                payments::status.eq(status_str),
                payments::updated_at.eq(Utc::now()),
                payments::completed_at.eq(completed_at),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| format!("update: {}", e))?;

        if let Some(note) = audit_note {
            use crate::schemas::payments::payment_audit_log;
            diesel::insert_into(payment_audit_log::table)
                .values((
                    payment_audit_log::id.eq(Uuid::new_v4()),
                    payment_audit_log::payment_id.eq(payment_id.value()),
                    payment_audit_log::action.eq("status_change"),
                    payment_audit_log::old_status.eq(None::<String>),
                    payment_audit_log::new_status.eq(status_str),
                    payment_audit_log::reason.eq(Some(note)),
                    payment_audit_log::performed_by.eq(None::<String>),
                    payment_audit_log::created_at.eq(Utc::now()),
                    payment_audit_log::metadata.eq(serde_json::json!({})),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| format!("audit: {}", e))?;
        }
        Ok(())
    }

    /// Replaces the inline `INSERT INTO subscriptions` block in
    /// `validation_handlers.rs::activate_subscription_handler`.
    /// Permission grants remain in the handler — Track C wraps
    /// `UnifiedPermissionService::grant_permission` in
    /// `PermissionAuthorityPort`. This port method only writes
    /// the data row.
    pub async fn grant_subscription_impl(
        &self,
        cmd: ActivateSubscriptionCommand,
    ) -> Result<Subscription, String> {
        let mut conn = self.conn().await?;
        let started_at = cmd.confirmed_at;
        let expires_at = started_at + chrono::Duration::days(cmd.duration_days as i64);
        let new_sub = crate::infrastructure::models::payment::NewSubscriptionDb {
            wallet_address: cmd.wallet_address.as_str().to_string(),
            plan_id: cmd.plan_id,
            payment_id: Some(cmd.payment_id.value()),
            status: "active".to_string(),
            started_at: Some(started_at),
            expires_at,
            cancelled_at: None,
            auto_renew: Some(false),
            metadata: Some(serde_json::json!({
                "transaction_hash": cmd.transaction_hash.hash(),
            })),
        };
        let inserted: SubscriptionDb = diesel::insert_into(subscriptions::table)
            .values(&new_sub)
            .returning(SubscriptionDb::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| format!("insert sub: {}", e))?;
        Ok(self.sub_row_to_domain(inserted))
    }

    /// Cancels the subscription. Returns Err if the row was
    /// not found.
    pub async fn revoke_subscription_impl(
        &self,
        subscription_id: Uuid,
        _reason: Option<String>,
    ) -> Result<(), String> {
        let mut conn = self.conn().await?;
        let updated = diesel::update(
            subscriptions::table.filter(subscriptions::id.eq(subscription_id)),
        )
        .set((
            subscriptions::status.eq("cancelled"),
            subscriptions::cancelled_at.eq(Some(Utc::now())),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| format!("update: {}", e))?;
        if updated == 0 {
            return Err(format!("Subscription {} not found", subscription_id));
        }
        Ok(())
    }
}

// ============================================================================
// Trait impl — forward the 8 new methods + the original 10 to
// the inherent methods (which live partly in this file, partly in
// `payment_repository_adapter.rs`).
// ============================================================================

#[async_trait]
impl PaymentRepositoryPort for PaymentRepositoryAdapter {
    async fn save(&self, payment: &Payment) -> Result<(), String> {
        PaymentRepositoryAdapter::_save_impl(self, payment).await
    }

    async fn find_by_id(&self, payment_id: &PaymentId) -> Result<Option<Payment>, String> {
        PaymentRepositoryAdapter::_find_by_id_impl(self, payment_id).await
    }

    async fn find_by_user(&self, wallet_address: &WalletAddress) -> Result<Vec<Payment>, String> {
        PaymentRepositoryAdapter::_find_by_user_impl(self, wallet_address).await
    }

    async fn find_by_status(&self, status: PaymentStatus) -> Result<Vec<Payment>, String> {
        PaymentRepositoryAdapter::_find_by_status_impl(self, status).await
    }

    async fn find_by_reference(&self, reference: &crate::domain::payment::PaymentReference) -> Result<Option<Payment>, String> {
        PaymentRepositoryAdapter::_find_by_reference_impl(self, reference).await
    }

    async fn find_by_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Payment>, String> {
        PaymentRepositoryAdapter::_find_by_date_range_impl(self, start, end).await
    }

    async fn find_expired_pending(&self, threshold: DateTime<Utc>) -> Result<Vec<Payment>, String> {
        PaymentRepositoryAdapter::_find_expired_pending_impl(self, threshold).await
    }

    async fn update_status(&self, payment_id: &PaymentId, status: PaymentStatus) -> Result<(), String> {
        PaymentRepositoryAdapter::_update_status_impl(self, payment_id, status).await
    }

    async fn delete(&self, payment_id: &PaymentId) -> Result<(), String> {
        PaymentRepositoryAdapter::_delete_impl(self, payment_id).await
    }

    async fn get_user_payment_stats(&self, wallet_address: &WalletAddress) -> Result<crate::domain::payment::PaymentStats, String> {
        PaymentRepositoryAdapter::_get_user_payment_stats_impl(self, wallet_address).await
    }

    async fn get_tx_status_with_plan_name(&self, tx_hash: &str) -> Result<Option<(Payment, Option<String>)>, String> {
        self.get_tx_status_with_plan_name_impl(tx_hash).await
    }

    async fn list_user_payments_with_plan_names(
        &self,
        wallet_address: &WalletAddress,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<(Payment, Option<String>)>, String> {
        self.list_user_payments_with_plan_names_impl(wallet_address, page, per_page).await
    }

    async fn get_admin_payment_details_with_plan_name(&self, payment_id: PaymentId) -> Result<Option<(Payment, Option<String>)>, String> {
        self.get_admin_payment_details_with_plan_name_impl(payment_id).await
    }

    async fn list_admin_subscriptions_with_plan_names(
        &self,
        filters: SubscriptionFilters,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<(Subscription, Option<String>)>, String> {
        self.list_admin_subscriptions_with_plan_names_impl(filters, page, per_page).await
    }

    async fn get_analytics_rollup(&self, window: AnalyticsWindow) -> Result<AnalyticsRollup, String> {
        self.get_analytics_rollup_impl(window).await
    }

    async fn validate_submit_tx(&self, plan_id: Uuid, wallet_address: &WalletAddress) -> Result<SubmitTxValidation, String> {
        self.validate_submit_tx_impl(plan_id, wallet_address).await
    }

    async fn create_payment(&self, cmd: CreatePaymentCommand) -> Result<Payment, String> {
        self.create_payment_impl(cmd).await
    }

    async fn update_payment_status(&self, payment_id: PaymentId, new_status: PaymentStatus, audit_note: Option<String>) -> Result<(), String> {
        self.update_payment_status_impl(payment_id, new_status, audit_note).await
    }

    async fn grant_subscription(&self, cmd: ActivateSubscriptionCommand) -> Result<Subscription, String> {
        self.grant_subscription_impl(cmd).await
    }

    async fn revoke_subscription(&self, subscription_id: Uuid, reason: Option<String>) -> Result<(), String> {
        self.revoke_subscription_impl(subscription_id, reason).await
    }
}
