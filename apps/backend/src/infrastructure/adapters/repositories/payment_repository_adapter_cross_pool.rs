//! Wave 11 / Track A — Cross-Pool Method Implementations
//!
//! BIG-BANG: migrated to sqlx (real). All diesel DSL queries replaced with
//! raw SQL via sqlx::query_as and sqlx::query. Cross-pool JOINs use single
//! `payments` schema (payments ⋈ plans in same DB; PAYMENTS_DATABASE_URL
//! falls back to primary in production).

use rust_decimal::prelude::*;
use chrono::{DateTime, Datelike, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::payment::repository_ports::{
    ActivateSubscriptionCommand, AnalyticsRollup, AnalyticsTrends, AnalyticsWindow,
    CreatePaymentCommand, DailyRevenueEntry, PaymentMethodEntry, PaymentRepositoryPort,
    PaymentRowWithPlanName, PaymentStats, PlanBreakdownEntry, SubmitTxValidation,
    Subscription, SubscriptionFilters,
};
use crate::domain::payment::{
    CryptoNetwork, CryptoPaymentDetails, Payment, PaymentAmount, PaymentId, PaymentMethod,
    PaymentReference, PaymentStatus, PlanId, TransactionHash,
};
use crate::domain::wallet_management::value_objects::WalletAddress;
use crate::infrastructure::models::payment::{PaymentDb, SubscriptionDb};

use super::payment_repository_adapter::PaymentRepositoryAdapter;

/// Shared `payments.*` + `payments.plans` JOIN row used by every `_with_plan_name` method.
#[derive(sqlx::FromRow)]
pub struct PaymentWithPlanRow {
    pub id: Uuid,
    pub payment_reference: String,
    pub transaction_hash: Option<String>,
    pub wallet_address: String,
    pub amount: bigdecimal::BigDecimal,
    pub currency: String,
    pub method: String,
    pub status: String,
    pub plan_id: Uuid,
    pub contract_address: Option<String>,
    pub token_address: Option<String>,
    pub block_number: Option<i64>,
    pub confirmations: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub network: Option<String>,
    pub plan_name: Option<String>,
}

impl PaymentWithPlanRow {
    pub fn into_domain(self) -> Result<(PaymentDb, Option<String>), String> {
        let db = PaymentDb {
            id: self.id,
            payment_reference: self.payment_reference,
            transaction_hash: self.transaction_hash,
            wallet_address: self.wallet_address,
            amount: self.amount,
            currency: self.currency,
            method: self.method,
            status: self.status,
            plan_id: self.plan_id,
            contract_address: self.contract_address,
            token_address: self.token_address,
            block_number: self.block_number,
            confirmations: self.confirmations,
            created_at: self.created_at,
            updated_at: self.updated_at,
            expires_at: self.expires_at,
            completed_at: self.completed_at,
            metadata: self.metadata,
            last_checked_at: self.last_checked_at,
            error_message: self.error_message,
            network: self.network,
        };
        Ok((db, self.plan_name))
    }
}

/// Shared `subscriptions.*` + `payments.plans` JOIN row for subscription listing.
#[derive(sqlx::FromRow)]
struct SubscriptionWithPlanRow {
    id: Uuid,
    wallet_address: String,
    plan_id: Uuid,
    payment_id: Option<Uuid>,
    status: String,
    started_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
    cancelled_at: Option<DateTime<Utc>>,
    auto_renew: Option<bool>,
    metadata: Option<serde_json::Value>,
    plan_name: Option<String>,
}

impl PaymentRepositoryAdapter {
    /// Build a domain `Payment` from a `PaymentDb` row.
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
        let transaction_hash = payment_db
            .transaction_hash
            .clone()
            .map(|hash| {
                TransactionHash::new(
                    hash,
                    crate::domain::payment::value_objects::Network::BinanceSmartChain,
                )
            })
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

    fn pool(&self) -> &PgPool {
        &self.db_pool
    }

    /// Single LEFT JOIN query against `payments ⋈ plans`.
    pub async fn get_tx_status_with_plan_name_impl(
        &self,
        tx_hash: &str,
    ) -> Result<Option<PaymentRowWithPlanName>, String> {
        let row: Option<PaymentWithPlanRow> = sqlx::query_as(
            r#"
            SELECT
                p.id, p.payment_reference, p.transaction_hash, p.wallet_address,
                p.amount, p.currency, p.method, p.status, p.plan_id,
                p.contract_address, p.token_address, p.block_number, p.confirmations,
                p.created_at, p.updated_at, p.expires_at, p.completed_at,
                p.metadata, p.last_checked_at, p.error_message, p.network,
                pl.name as plan_name
            FROM payments p
            LEFT JOIN payments.plans pl ON p.plan_id = pl.id
            WHERE p.transaction_hash = $1
            LIMIT 1
            "#,
        )
        .bind(tx_hash)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| format!("get_tx_status: {}", e))?;

        let Some(row) = row else { return Ok(None) };
        let (db, plan_name) = row.into_domain()?;
        Ok(Some(PaymentRowWithPlanName::from_db(&db, plan_name)))
    }

    /// N+1 fix: single LEFT JOIN query for paginated user payments.
    pub async fn list_user_payments_with_plan_names_impl(
        &self,
        wallet_address: &WalletAddress,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<PaymentRowWithPlanName>, String> {
        let per_page = per_page.clamp(1, 50);
        let page = page.max(1);
        let offset = ((page - 1) * per_page) as i64;
        let limit = per_page as i64;

        let rows: Vec<PaymentWithPlanRow> = sqlx::query_as(
            r#"
            SELECT
                p.id, p.payment_reference, p.transaction_hash, p.wallet_address,
                p.amount, p.currency, p.method, p.status, p.plan_id,
                p.contract_address, p.token_address, p.block_number, p.confirmations,
                p.created_at, p.updated_at, p.expires_at, p.completed_at,
                p.metadata, p.last_checked_at, p.error_message, p.network,
                pl.name as plan_name
            FROM payments p
            LEFT JOIN payments.plans pl ON p.plan_id = pl.id
            WHERE p.wallet_address = $1
            ORDER BY p.created_at DESC NULLS LAST
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(wallet_address.as_str())
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool())
        .await
        .map_err(|e| format!("list_user_payments: {}", e))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let (db, plan_name) = r.into_domain()?;
            out.push(PaymentRowWithPlanName::from_db(&db, plan_name));
        }
        Ok(out)
    }

    pub async fn get_admin_payment_details_with_plan_name_impl(
        &self,
        payment_id: PaymentId,
    ) -> Result<Option<PaymentRowWithPlanName>, String> {
        let row: Option<PaymentWithPlanRow> = sqlx::query_as(
            r#"
            SELECT
                p.id, p.payment_reference, p.transaction_hash, p.wallet_address,
                p.amount, p.currency, p.method, p.status, p.plan_id,
                p.contract_address, p.token_address, p.block_number, p.confirmations,
                p.created_at, p.updated_at, p.expires_at, p.completed_at,
                p.metadata, p.last_checked_at, p.error_message, p.network,
                pl.name as plan_name
            FROM payments p
            LEFT JOIN payments.plans pl ON p.plan_id = pl.id
            WHERE p.id = $1
            LIMIT 1
            "#,
        )
        .bind(payment_id.value())
        .fetch_optional(self.pool())
        .await
        .map_err(|e| format!("admin_payment_details: {}", e))?;

        let Some(row) = row else { return Ok(None) };
        let (db, plan_name) = row.into_domain()?;
        Ok(Some(PaymentRowWithPlanName::from_db(&db, plan_name)))
    }

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

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT s.id, s.wallet_address, s.plan_id, s.payment_id, s.status, s.started_at, s.expires_at, s.cancelled_at, s.auto_renew, s.metadata, pl.name as plan_name \
             FROM subscriptions s LEFT JOIN payments.plans pl ON s.plan_id = pl.id WHERE TRUE",
        );
        if let Some(w) = filters.wallet_address.as_ref() {
            qb.push(" AND s.wallet_address = ").push_bind(w.clone());
        }
        if let Some(p) = filters.plan_id {
            qb.push(" AND s.plan_id = ").push_bind(p);
        }
        if let Some(s) = filters.status.as_ref() {
            qb.push(" AND s.status = ").push_bind(s.clone());
        }
        qb.push(" ORDER BY s.started_at DESC NULLS LAST LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows: Vec<SubscriptionWithPlanRow> = qb
            .build_query_as()
            .fetch_all(self.pool())
            .await
            .map_err(|e| format!("list_admin_subs: {}", e))?;

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

    pub async fn list_admin_subscriptions_count_impl(
        &self,
        filters: SubscriptionFilters,
    ) -> Result<u64, String> {
        let mut qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) AS c FROM subscriptions WHERE TRUE");
        if let Some(w) = filters.wallet_address.as_ref() {
            qb.push(" AND wallet_address = ").push_bind(w.clone());
        }
        if let Some(p) = filters.plan_id {
            qb.push(" AND plan_id = ").push_bind(p);
        }
        if let Some(s) = filters.status.as_ref() {
            qb.push(" AND status = ").push_bind(s.clone());
        }

        let row: (i64,) = qb
            .build_query_as()
            .fetch_one(self.pool())
            .await
            .map_err(|e| format!("count: {}", e))?;
        Ok(row.0.max(0) as u64)
    }

    pub async fn get_analytics_rollup_impl(
        &self,
        window: AnalyticsWindow,
    ) -> Result<AnalyticsRollup, String> {
        let now = Utc::now();
        let since = match window {
            AnalyticsWindow::Last30Days => now - chrono::Duration::days(30),
            AnalyticsWindow::Last7Days => now - chrono::Duration::days(7),
            AnalyticsWindow::Last24Hours => now - chrono::Duration::hours(24),
            AnalyticsWindow::MonthToDate => now
                .date_naive()
                .with_day(1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                .unwrap_or(now),
        };

        #[derive(sqlx::FromRow)]
        struct DailyRevenueRow {
            payment_date: chrono::NaiveDate,
            daily_revenue: Option<rust_decimal::Decimal>,
            payment_count: i64,
        }
        let daily_rows: Vec<DailyRevenueRow> = sqlx::query_as(
            r#"
            SELECT DATE(created_at) as payment_date,
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
        .bind(since)
        .fetch_all(self.pool())
        .await
        .map_err(|e| format!("daily: {}", e))?;

        let daily_revenue: Vec<DailyRevenueEntry> = daily_rows
            .into_iter()
            .map(|r| DailyRevenueEntry {
                date: r.payment_date.format("%Y-%m-%d").to_string(),
                revenue: r.daily_revenue.map(|bd| bd.to_f64().unwrap_or(0.0)).unwrap_or(0.0),
                payment_count: r.payment_count as u32,
            })
            .collect();

        #[derive(sqlx::FromRow)]
        struct PlanBreakdownRow {
            plan_id: Uuid,
            total_revenue: Option<rust_decimal::Decimal>,
            subscription_count: i64,
            plan_name: Option<String>,
        }
        let plan_rows: Vec<PlanBreakdownRow> = sqlx::query_as(
            r#"
            SELECT p.plan_id as plan_id,
                   SUM(p.amount) as total_revenue,
                   COUNT(*) as subscription_count,
                   pl.name as plan_name
            FROM payments p
            LEFT JOIN payments.plans pl ON p.plan_id = pl.id
            WHERE p.created_at >= $1
              AND (p.status = 'completed' OR p.status = 'confirmed')
            GROUP BY p.plan_id, pl.name
            ORDER BY total_revenue DESC NULLS LAST
            LIMIT 20
            "#,
        )
        .bind(since)
        .fetch_all(self.pool())
        .await
        .map_err(|e| format!("plan_breakdown: {}", e))?;
        let plan_breakdown: Vec<PlanBreakdownEntry> = plan_rows
            .into_iter()
            .map(|r| {
                let rev = r.total_revenue.map(|bd| bd.to_f64().unwrap_or(0.0)).unwrap_or(0.0);
                let count = r.subscription_count as u32;
                let arpu = if count > 0 { rev / count as f64 } else { 0.0 };
                PlanBreakdownEntry {
                    plan_id: r.plan_id,
                    plan_name: r.plan_name.unwrap_or_else(|| "Unknown".to_string()),
                    total_revenue: rev,
                    subscription_count: count,
                    average_revenue_per_user: arpu,
                }
            })
            .collect();

        #[derive(sqlx::FromRow)]
        struct MethodRow {
            payment_method: String,
            payment_count: i64,
        }
        let method_rows: Vec<MethodRow> = sqlx::query_as(
            r#"
            SELECT method as payment_method, COUNT(*) as payment_count
            FROM payments
            WHERE created_at >= $1
            GROUP BY method
            ORDER BY payment_count DESC
            "#,
        )
        .bind(since)
        .fetch_all(self.pool())
        .await
        .map_err(|e| format!("methods: {}", e))?;
        let payment_methods: Vec<PaymentMethodEntry> = method_rows
            .into_iter()
            .map(|r| PaymentMethodEntry {
                currency: r.payment_method,
                payment_count: r.payment_count as u32,
                total_revenue: 0.0,
                success_rate: 100.0,
            })
            .collect();

        Ok(AnalyticsRollup {
            daily_revenue,
            plan_breakdown,
            payment_methods,
            trends: AnalyticsTrends::default(),
        })
    }

    pub async fn validate_submit_tx_impl(
        &self,
        plan_id: Uuid,
        _wallet_address: &WalletAddress,
    ) -> Result<SubmitTxValidation, String> {
        #[derive(sqlx::FromRow)]
        struct PlanRow {
            price: Option<bigdecimal::BigDecimal>,
            is_active: bool,
            plan_type: String,
            plan_metadata: serde_json::Value,
        }

        let plan: Option<PlanRow> = sqlx::query_as(
            "SELECT price, is_active, plan_type, plan_metadata FROM plans WHERE id = $1 LIMIT 1",
        )
        .bind(plan_id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| format!("validate_submit_tx: {}", e))?;

        let Some(p) = plan else {
            return Err("Plan not found".to_string());
        };

        let price_str = p.price.map(|bd| bd.to_string()).unwrap_or_else(|| "0".to_string());

        Ok(SubmitTxValidation {
            plan_price: price_str.clone(),
            is_active: p.is_active,
            plan_type: p.plan_type,
            plan_metadata: p.plan_metadata,
            effective_price: price_str,
        })
    }
}

// ============================================================================
// Trait impl for PaymentRepositoryPort
// ============================================================================

#[async_trait::async_trait]
impl PaymentRepositoryPort for PaymentRepositoryAdapter {
    async fn save(&self, payment: &Payment) -> Result<(), String> {
        self._save_impl(payment).await
    }

    async fn find_by_id(&self, payment_id: &PaymentId) -> Result<Option<Payment>, String> {
        self._find_by_id_impl(payment_id).await
    }

    async fn find_by_user(
        &self,
        wallet_address: &WalletAddress,
    ) -> Result<Vec<Payment>, String> {
        self._find_by_user_impl(wallet_address).await
    }

    async fn find_by_status(&self, status: PaymentStatus) -> Result<Vec<Payment>, String> {
        self._find_by_status_impl(status).await
    }

    async fn find_by_reference(
        &self,
        reference: &PaymentReference,
    ) -> Result<Option<Payment>, String> {
        self._find_by_reference_impl(reference).await
    }

    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Payment>, String> {
        self._find_by_date_range_impl(start, end).await
    }

    async fn find_expired_pending(
        &self,
        threshold: DateTime<Utc>,
    ) -> Result<Vec<Payment>, String> {
        self._find_expired_pending_impl(threshold).await
    }

    async fn update_status(
        &self,
        payment_id: &PaymentId,
        status: PaymentStatus,
    ) -> Result<(), String> {
        self._update_status_impl(payment_id, status).await
    }

    async fn delete(&self, payment_id: &PaymentId) -> Result<(), String> {
        self._delete_impl(payment_id).await
    }

    async fn get_user_payment_stats(
        &self,
        wallet_address: &WalletAddress,
    ) -> Result<PaymentStats, String> {
        self._get_user_payment_stats_impl(wallet_address).await
    }

    async fn get_tx_status_with_plan_name(
        &self,
        tx_hash: &str,
    ) -> Result<Option<PaymentRowWithPlanName>, String> {
        self.get_tx_status_with_plan_name_impl(tx_hash).await
    }

    async fn list_user_payments_with_plan_names(
        &self,
        wallet_address: &WalletAddress,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<PaymentRowWithPlanName>, String> {
        self.list_user_payments_with_plan_names_impl(wallet_address, page, per_page)
            .await
    }

    async fn get_admin_payment_details_with_plan_name(
        &self,
        payment_id: PaymentId,
    ) -> Result<Option<PaymentRowWithPlanName>, String> {
        self.get_admin_payment_details_with_plan_name_impl(payment_id)
            .await
    }

    async fn list_admin_subscriptions_with_plan_names(
        &self,
        filters: SubscriptionFilters,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<(Subscription, Option<String>)>, String> {
        self.list_admin_subscriptions_with_plan_names_impl(filters, page, per_page)
            .await
    }

    async fn list_admin_subscriptions_with_plan_names_paginated(
        &self,
        filters: SubscriptionFilters,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<(Subscription, Option<String>)>, u64), String> {
        let subs = self.list_admin_subscriptions_with_plan_names_impl(filters.clone(), page, per_page).await?;
        let count = self.list_admin_subscriptions_count_impl(filters).await?;
        Ok((subs, count))
    }

    async fn get_analytics_rollup(&self, window: AnalyticsWindow) -> Result<AnalyticsRollup, String> {
        self.get_analytics_rollup_impl(window).await
    }

    async fn validate_submit_tx(
        &self,
        plan_id: Uuid,
        wallet_address: &WalletAddress,
    ) -> Result<SubmitTxValidation, String> {
        self.validate_submit_tx_impl(plan_id, wallet_address).await
    }

    async fn create_payment(&self, cmd: CreatePaymentCommand) -> Result<Payment, String> {
        let payment_id = PaymentId::new();
        let reference = PaymentReference::new(cmd.payment_reference)
            .map_err(|e| format!("Invalid reference: {:?}", e))?;
        let wallet = WalletAddress::new(cmd.wallet_address)
            .map_err(|e| format!("Invalid wallet: {:?}", e))?;
        let amount = PaymentAmount::from_f64(
            cmd.amount.parse::<f64>().unwrap_or(0.0),
            cmd.currency,
        )
        .map_err(|e| format!("Invalid amount: {:?}", e))?;
        let status = match cmd.status.as_str() {
            "completed" | "confirmed" => PaymentStatus::Completed,
            "failed" => PaymentStatus::Failed,
            _ => PaymentStatus::Pending,
        };

        let payment = Payment::new(
            payment_id,
            reference,
            wallet,
            amount,
            status,
            cmd.transaction_hash.and_then(|h| TransactionHash::new(h).ok()),
            cmd.plan_id.to_string(),
            cmd.expires_at.unwrap_or_else(Utc::now),
            cmd.metadata.unwrap_or_else(|| serde_json::json!({})),
        )
        .map_err(|e| format!("Payment::new error: {:?}", e))?;

        self._save_impl(&payment).await?;
        Ok(payment)
    }

    async fn update_payment_status(
        &self,
        payment_id: PaymentId,
        new_status: PaymentStatus,
        _audit_note: Option<String>,
    ) -> Result<(), String> {
        self._update_status_impl(&payment_id, new_status).await
    }

    async fn grant_subscription(
        &self,
        cmd: ActivateSubscriptionCommand,
    ) -> Result<Subscription, String> {
        let sub_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(cmd.duration_days as i64);

        sqlx::query(
            r#"
            INSERT INTO subscriptions (id, wallet_address, plan_id, status, started_at, expires_at, auto_renew, metadata)
            VALUES ($1, $2, $3, 'active', $4, $5, false, '{}')
            "#,
        )
        .bind(sub_id)
        .bind(cmd.wallet_address.to_string())
        .bind(cmd.plan_id)
        .bind(now)
        .bind(expires_at)
        .execute(self.pool())
        .await
        .map_err(|e| format!("grant_subscription: {}", e))?;

        Ok(Subscription {
            id: sub_id,
            wallet_address: cmd.wallet_address.to_string(),
            plan_id: cmd.plan_id,
            payment_id: Some(cmd.payment_id.value()),
            status: "active".to_string(),
            started_at: Some(now),
            expires_at,
            cancelled_at: None,
            auto_renew: false,
            metadata: serde_json::json!({}),
        })
    }

    async fn revoke_subscription(
        &self,
        subscription_id: Uuid,
        _reason: Option<String>,
    ) -> Result<(), String> {
        sqlx::query(
            "UPDATE subscriptions SET status = 'cancelled', cancelled_at = NOW() WHERE id = $1",
        )
        .bind(subscription_id)
        .execute(self.pool())
        .await
        .map_err(|e| format!("revoke_subscription: {}", e))?;
        Ok(())
    }
}