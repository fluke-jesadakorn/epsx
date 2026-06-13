// Payment Bounded Context Repository Ports
// These define the interfaces for data persistence in the Payment bounded context

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Payment, PaymentId, PaymentStatus, PaymentAmount,
    TransactionHash, CryptoAddress, PaymentReference
};
use crate::domain::wallet_management::value_objects::WalletAddress;

// ============================================================================
// Wave 11 / Track A — Cross-Pool Port Methods
// ============================================================================
//
// The 8 methods below (`get_tx_status_with_plan_name`,
// `list_user_payments_with_plan_names`, `get_admin_payment_details_with_plan_name`,
// `list_admin_subscriptions_with_plan_names`, `get_analytics_rollup`,
// `validate_submit_tx`, `grant_subscription`, `revoke_subscription`,
// `create_payment`, `update_payment_status`) collapse the
// `payments_pool` + `get_diesel_pool()` cross-pool join pattern
// in the 8 web/payments/* handler sites that the wave-8 payments
// audit (audit-payments.md §4) flagged.
//
// The DTOs that travel across the trait boundary (e.g.
// `SubscriptionFilters`, `AnalyticsWindow`, `AnalyticsRollup`,
// `SubmitTxValidation`) derive `Serialize` + `Deserialize` so the
// future HTTP impl of this trait (the `epsx-payments` binary
// land in the integration gate) can round-trip them as JSON
// without losing the type-level invariants.

/// Filters for `list_admin_subscriptions_with_plan_names`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionFilters {
    pub wallet_address: Option<String>,
    pub plan_id: Option<Uuid>,
    pub status: Option<String>,
}

/// Time window for `get_analytics_rollup`. Maps directly to the
/// `last_30_days` / `last_7_days` / `last_24_hours` aggregations
/// the previous `admin_get_payment_analytics_handler` issued
/// against `payments` / `subscriptions`.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnalyticsWindow {
    #[default]
    Last30Days,
    Last7Days,
    Last24Hours,
    MonthToDate,
}

/// Aggregated analytics for the admin rollup endpoint. The numeric
/// fields are the same ones the previous `PaymentAnalytics` shape
/// returned; the per-day / per-plan / per-currency breakdowns are
/// exposed as parallel `Vec<...>` slices so the future HTTP impl
/// can JSON-serialize them as a flat object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsRollup {
    pub daily_revenue: Vec<DailyRevenueEntry>,
    pub plan_breakdown: Vec<PlanBreakdownEntry>,
    pub payment_methods: Vec<PaymentMethodEntry>,
    pub trends: AnalyticsTrends,
}

/// One row in the `daily_revenue` slice of [`AnalyticsRollup`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRevenueEntry {
    pub date: String, // YYYY-MM-DD
    pub revenue: f64,
    pub payment_count: u32,
}

/// One row in the `plan_breakdown` slice of [`AnalyticsRollup`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanBreakdownEntry {
    pub plan_id: Uuid,
    pub plan_name: String,
    pub total_revenue: f64,
    pub subscription_count: u32,
    pub average_revenue_per_user: f64,
}

/// One row in the `payment_methods` slice of [`AnalyticsRollup`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethodEntry {
    pub currency: String,
    pub payment_count: u32,
    pub total_revenue: f64,
    pub success_rate: f64,
}

/// Aggregate trend metrics returned inside [`AnalyticsRollup`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsTrends {
    pub growth_rate: f64,
    pub churn_rate: f64,
    pub average_subscription_length: f64,
    pub customer_lifetime_value: f64,
}

/// Result of `validate_submit_tx`. Mirrors the
/// `(plan_price, is_active, plan_type, plan_metadata, effective_price)`
/// computation the old `submit_transaction_handler` did inline after
/// the cross-pool `SELECT FROM plans` lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTxValidation {
    pub plan_price: String, // BigDecimal as string for precision
    pub is_active: bool,
    pub plan_type: String,
    pub plan_metadata: serde_json::Value,
    pub effective_price: String, // BigDecimal as string
}

/// Port for payment repository operations
#[async_trait]
pub trait PaymentRepositoryPort: Send + Sync {
    /// Save a payment aggregate
    async fn save(&self, payment: &Payment) -> Result<(), String>;

    /// Find payment by ID
    async fn find_by_id(&self, payment_id: &PaymentId) -> Result<Option<Payment>, String>;

    /// Find payments by user
    async fn find_by_user(&self, wallet_address: &WalletAddress) -> Result<Vec<Payment>, String>;

    /// Find payments by status
    async fn find_by_status(&self, status: PaymentStatus) -> Result<Vec<Payment>, String>;

    /// Find payments by reference
    async fn find_by_reference(&self, reference: &PaymentReference) -> Result<Option<Payment>, String>;

    /// Find payments within date range
    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>
    ) -> Result<Vec<Payment>, String>;

    /// Find pending payments older than threshold
    async fn find_expired_pending(&self, threshold: DateTime<Utc>) -> Result<Vec<Payment>, String>;

    /// Update payment status
    async fn update_status(&self, payment_id: &PaymentId, status: PaymentStatus) -> Result<(), String>;

    /// Delete payment
    async fn delete(&self, payment_id: &PaymentId) -> Result<(), String>;

    /// Get payment statistics for user
    async fn get_user_payment_stats(&self, wallet_address: &WalletAddress) -> Result<PaymentStats, String>;

    // -----------------------------------------------------------------
    // Wave 11 / Track A additions — the 8 cross-pool collapses
    // -----------------------------------------------------------------

    /// Look up a transaction by hash and return the payment along
    /// with its plan name in a SINGLE query. Replaces the
    /// `payments_pool` + `get_diesel_pool()` cross-pool pattern
    /// in `web/payments/get_tx_status_handler.rs:121-137`.
    ///
    /// In the in-process impl the JOIN runs against the
    /// `payments ⋈ plans` tables; today the two pools share a
    /// schema (see audit-payments.md §4) so the JOIN is harmless.
    /// The integration gate replicates `plans` into the payments
    /// schema (wave-11 cutover SQL) and from that point the
    /// HTTP impl of this trait serves the same shape from a
    /// single pool.
    async fn get_tx_status_with_plan_name(
        &self,
        tx_hash: &str,
    ) -> Result<Option<(Payment, Option<String>)>, String>;

    /// Paginated user-payment history with plan names attached.
    ///
    /// Replaces the N+1 `for payment in payments_list { plans::table... }`
    /// loop in `web/payments/user_payment_handlers.rs:144-166`. The
    /// in-process impl runs ONE LEFT JOIN; the regression test in
    /// `payment_repository_adapter::tests::n_plus_one_user_payments`
    /// pins the query count to 1 for a 50-row page.
    async fn list_user_payments_with_plan_names(
        &self,
        wallet_address: &WalletAddress,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<(Payment, Option<String>)>, String>;

    /// Single-payment lookup for the admin details endpoint, with
    /// the plan name attached. Replaces the cross-pool lookup in
    /// `web/payments/admin_handlers/payment_handlers.rs:249-270`.
    async fn get_admin_payment_details_with_plan_name(
        &self,
        payment_id: PaymentId,
    ) -> Result<Option<(Payment, Option<String>)>, String>;

    /// Paginated admin subscription list with plan names attached.
    /// Replaces the two-conn `subscriptions::table` + `plans::table`
    /// block in `web/payments/admin_handlers/subscription_handlers.rs:32-101`.
    async fn list_admin_subscriptions_with_plan_names(
        &self,
        filters: SubscriptionFilters,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<(Subscription, Option<String>)>, String>;

    /// Aggregate analytics rollup. Replaces the 4 sql_query blocks
    /// in `web/payments/admin_handlers/analytics_handlers.rs:39-44`
    /// (daily revenue, plan breakdown, payment methods, trends).
    /// The impl uses ONE `payments` query per slice — there is no
    /// cross-pool `plans::table` lookup because the JOIN against
    /// `plans` is done inline on the payments side.
    async fn get_analytics_rollup(
        &self,
        window: AnalyticsWindow,
    ) -> Result<AnalyticsRollup, String>;

    /// Server-side validation for a payment submission.
    /// Replaces the `get_diesel_pool()` + `SELECT FROM plans`
    /// block at `web/payments/submit_tx_handler.rs:158-184`
    /// and the `wallet_credits.balance` read at `285-294`.
    /// Joins `plans` + reads `wallet_credits` in a single call.
    async fn validate_submit_tx(
        &self,
        plan_id: Uuid,
        wallet_address: &WalletAddress,
    ) -> Result<SubmitTxValidation, String>;

    /// Create a new payment row. Replaces the inline `INSERT INTO
    /// payments` blocks in `submit_tx_handler.rs:339-414`. The impl
    /// runs against the payments pool only.
    async fn create_payment(
        &self,
        cmd: CreatePaymentCommand,
    ) -> Result<Payment, String>;

    /// Update the status of a payment, with optional audit note.
    /// Replaces the inline `UPDATE payments SET status = ...` blocks
    /// in `admin_handlers/payment_handlers.rs:325-427` and others.
    async fn update_payment_status(
        &self,
        payment_id: PaymentId,
        new_status: PaymentStatus,
        audit_note: Option<String>,
    ) -> Result<(), String>;

    /// Activate a subscription. Replaces the inline `INSERT INTO
    /// subscriptions` block in
    /// `validation_handlers.rs::activate_subscription_handler`.
    /// Permission grants remain in the handler (Track C wraps them
    /// in `PermissionAuthorityPort`); the port only handles the
    /// data write.
    async fn grant_subscription(
        &self,
        cmd: ActivateSubscriptionCommand,
    ) -> Result<Subscription, String>;

    /// Revoke a subscription. The `reason` string is appended to
    /// the audit log; the impl cancels the row and writes a
    /// `payment_audit_log` entry in one transaction.
    async fn revoke_subscription(
        &self,
        subscription_id: Uuid,
        reason: Option<String>,
    ) -> Result<(), String>;
}

/// Lightweight subscription row used by
/// `list_admin_subscriptions_with_plan_names`. Mirrors the Diesel
/// `SubscriptionDb` shape but stays in the domain layer (no
/// `diesel` import). The concrete adapter converts to/from the
/// row type at the seam.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub payment_id: Option<Uuid>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub metadata: serde_json::Value,
}

/// Create-payment command DTO used by `create_payment`. Mirrors
/// `application::payment::commands::CreatePaymentCommand` but
/// stays in the domain layer (the `CreatePaymentCommand` from
/// the application layer is wired through this port by
/// handlers — the domain-level shape is what travels over the
/// future HTTP boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentCommand {
    pub payment_reference: String,
    pub wallet_address: String,
    pub amount: String, // BigDecimal as string
    pub currency: String,
    pub method: String,
    pub plan_id: Uuid,
    pub status: String,
    pub transaction_hash: Option<String>,
    pub network: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Activate-subscription command DTO used by `grant_subscription`.
/// Mirrors `application::payment::commands::ActivateSubscriptionCommand`
/// but uses a `Uuid` for `plan_id` (the application-layer
/// command uses `i32` — a legacy field per
/// `commands/activate_subscription_command.rs:11`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateSubscriptionCommand {
    pub payment_id: PaymentId,
    pub wallet_address: WalletAddress,
    pub plan_id: Uuid,
    pub transaction_hash: TransactionHash,
    pub confirmed_at: DateTime<Utc>,
    pub duration_days: u32,
}

/// Port for transaction monitoring and blockchain operations
#[async_trait]
pub trait TransactionRepositoryPort: Send + Sync {
    /// Store transaction hash for monitoring
    async fn store_transaction(&self, payment_id: &PaymentId, tx_hash: &TransactionHash) -> Result<(), String>;

    /// Find transaction by hash
    async fn find_by_hash(&self, tx_hash: &TransactionHash) -> Result<Option<TransactionRecord>, String>;

    /// Update transaction confirmation status
    async fn update_confirmations(&self, tx_hash: &TransactionHash, confirmations: u32) -> Result<(), String>;

    /// Find transactions needing confirmation checks
    async fn find_pending_confirmations(&self) -> Result<Vec<TransactionRecord>, String>;

    /// Get transaction history for payment
    async fn get_transaction_history(&self, payment_id: &PaymentId) -> Result<Vec<TransactionRecord>, String>;
}

/// Port for crypto address management
#[async_trait]
pub trait CryptoAddressRepositoryPort: Send + Sync {
    /// Generate new address for payment
    async fn generate_address(&self, payment_id: &PaymentId, network: &str) -> Result<CryptoAddress, String>;

    /// Find address by payment ID
    async fn find_by_payment(&self, payment_id: &PaymentId) -> Result<Option<CryptoAddress>, String>;

    /// Mark address as used
    async fn mark_address_used(&self, address: &CryptoAddress) -> Result<(), String>;

    /// Get address balance
    async fn get_address_balance(&self, address: &CryptoAddress) -> Result<PaymentAmount, String>;

    /// Find addresses by user for reuse
    async fn find_user_addresses(&self, wallet_address: &WalletAddress, network: &str) -> Result<Vec<CryptoAddress>, String>;
}

/// Port for payment method configuration
#[async_trait]
pub trait PaymentMethodRepositoryPort: Send + Sync {
    /// Get available payment methods
    async fn get_available_methods(&self) -> Result<Vec<super::PaymentMethod>, String>;

    /// Get payment method configuration
    async fn get_method_config(&self, method_type: &str) -> Result<Option<super::PaymentMethodConfig>, String>;

    /// Update payment method availability
    async fn update_method_availability(&self, method_type: &str, available: bool) -> Result<(), String>;

    /// Get exchange rates for currency conversion
    async fn get_exchange_rates(&self, base_currency: &str) -> Result<super::ExchangeRates, String>;
}

/// Payment statistics
#[derive(Debug, Clone)]
pub struct PaymentStats {
    pub total_payments: u32,
    pub completed_payments: u32,
    pub failed_payments: u32,
    pub total_amount: PaymentAmount,
    pub average_amount: PaymentAmount,
    pub last_payment_date: Option<DateTime<Utc>>,
}

/// Transaction record for monitoring
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub payment_id: PaymentId,
    pub tx_hash: TransactionHash,
    pub network: String,
    pub confirmations: u32,
    pub required_confirmations: u32,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub gas_price: Option<u64>,
}

// ============================================================================
// Payment Context Repository Port (V2 Dynamic Payments)
// ============================================================================

use super::aggregates::{PaymentContext, PaymentContextId, PaymentContextType};

/// Port for payment context (dynamic payment link) operations
#[async_trait]
pub trait PaymentContextRepositoryPort: Send + Sync {
    /// Save a payment context aggregate
    async fn save(&self, context: &PaymentContext) -> Result<(), String>;

    /// Find payment context by ID
    async fn find_by_id(&self, id: &PaymentContextId) -> Result<Option<PaymentContext>, String>;

    /// Find payment context by slug
    async fn find_by_slug(&self, slug: &str) -> Result<Option<PaymentContext>, String>;

    /// Find all payment contexts by context type
    async fn find_by_type(&self, context_type: PaymentContextType) -> Result<Vec<PaymentContext>, String>;

    /// Find payment contexts linked to a specific entity (plan, group, etc.)
    async fn find_by_context_id(&self, context_id: &uuid::Uuid) -> Result<Vec<PaymentContext>, String>;

    /// List all active payment contexts with pagination
    async fn list_active(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PaymentContext>, String>;

    /// List all payment contexts with pagination and filters
    async fn list_with_filters(
        &self,
        context_type: Option<PaymentContextType>,
        is_active: Option<bool>,
        created_by: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PaymentContext>, i64), String>;

    /// Update payment context
    async fn update(&self, context: &PaymentContext) -> Result<(), String>;

    /// Delete payment context (soft delete by setting is_active = false)
    async fn soft_delete(&self, id: &PaymentContextId) -> Result<(), String>;

    /// Increment usage count
    async fn increment_usage(&self, id: &PaymentContextId) -> Result<(), String>;

    /// Find expired contexts that need cleanup
    async fn find_expired(&self) -> Result<Vec<PaymentContext>, String>;

    /// Get total count with filters
    async fn count_with_filters(
        &self,
        context_type: Option<PaymentContextType>,
        is_active: Option<bool>,
        created_by: Option<&str>,
    ) -> Result<i64, String>;
}

/// Transaction history information for UI
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransactionHistoryInfo {
    pub tx_hash: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub from_address: String,
    pub to_address: String,
    pub block_number: u64,
    pub plan_name: Option<String>,
}

/// Port for fetching transaction history from blockchain sources (RPC/Scanner)
#[async_trait]
pub trait TransactionHistoryProvider: Send + Sync {
    /// Get paginated transaction history for a wallet
    async fn get_history(
        &self,
        wallet_address: &str,
        page: u32,
        per_page: u32,
    ) -> Result<(Vec<TransactionHistoryInfo>, u64), String>;
}

// ============================================================================
// Wave 11 / Track A — CreditRepositoryPort
// ============================================================================
//
// Mirrors `PaymentRepositoryPort`. The concrete impl is
// `CreditRepositoryAdapter` (in
// `infrastructure::adapters::repositories::credit_repository_adapter`),
// which already implements the 6 operations as plain methods —
// this trait is a thin port wrapper so the future
// `epsx-payments` binary can serve credits over HTTP the same
// way it serves payments.

/// Port for credit wallet operations
#[async_trait]
pub trait CreditRepositoryPort: Send + Sync {
    /// Get credit balance for a wallet
    async fn get_balance(&self, wallet_address: &str) -> Result<Option<CreditBalanceRow>, String>;

    /// Get or create wallet credits record (returns balance)
    async fn get_or_create_balance(&self, wallet_address: &str) -> Result<CreditBalanceRow, String>;

    /// Get credit transactions for a wallet
    async fn get_transactions(
        &self,
        wallet_address: &str,
        filters: Option<CreditTransactionFilters>,
    ) -> Result<Vec<CreditTransactionRow>, String>;

    /// Get all credit transactions (admin)
    async fn get_all_transactions(
        &self,
        filters: Option<CreditTransactionFilters>,
    ) -> Result<Vec<CreditTransactionRow>, String>;

    /// Add credit transaction (uses database function for atomic balance update)
    async fn add_transaction(
        &self,
        wallet_address: &str,
        amount: String, // BigDecimal as string
        tx_type: &str,
        reference_id: Option<Uuid>,
        reference_type: Option<&str>,
        reason: Option<&str>,
        granted_by: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Uuid, String>;

    /// Get credit statistics (admin)
    async fn get_stats(&self) -> Result<CreditStats, String>;
}

/// Per-wallet credit balance row. Mirrors `WalletCreditDb` but
/// stays in the domain layer (no `diesel` import).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditBalanceRow {
    pub wallet_address: String,
    pub balance: String,        // BigDecimal as string
    pub pending_balance: String,
    pub lifetime_earned: String,
    pub lifetime_spent: String,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Per-transaction credit movement row. Mirrors `CreditTransactionDb`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransactionRow {
    pub id: Uuid,
    pub wallet_address: String,
    pub amount: String,
    pub balance_after: String,
    pub tx_type: String,
    pub reference_id: Option<Uuid>,
    pub reference_type: Option<String>,
    pub reason: Option<String>,
    pub granted_by: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Filters for `get_transactions` / `get_all_transactions`. Mirrors
/// the existing `infrastructure::models::credit::CreditTransactionFilters`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreditTransactionFilters {
    pub wallet_address: Option<String>,
    pub tx_type: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Aggregate credit statistics for the admin stats endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditStats {
    pub total_credits_outstanding: String, // BigDecimal as string
    pub total_credits_granted_today: String,
    pub total_credits_used_today: String,
    pub active_users_with_credits: i64,
    pub total_transactions_today: i64,
    pub average_balance: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Object-safety compile-time check: this would fail to
    /// compile if `PaymentRepositoryPort` had a method that
    /// returned an `impl Trait` or had a generic method (e.g.
    /// `async fn foo<T>`), which is not object-safe.
    #[allow(dead_code)]
    fn _assert_payment_repo_object_safe(_: &dyn PaymentRepositoryPort) {}

    /// Object-safety compile-time check for `CreditRepositoryPort`.
    #[allow(dead_code)]
    fn _assert_credit_repo_object_safe(_: &dyn CreditRepositoryPort) {}

    #[test]
    fn analytics_window_default_is_last_30_days() {
        assert_eq!(AnalyticsWindow::default(), AnalyticsWindow::Last30Days);
    }

    #[test]
    fn subscription_filters_default_is_empty() {
        let f = SubscriptionFilters::default();
        assert!(f.wallet_address.is_none());
        assert!(f.plan_id.is_none());
        assert!(f.status.is_none());
    }

    #[test]
    fn create_payment_command_dto_is_serializable() {
        let cmd = CreatePaymentCommand {
            payment_reference: "PAY-test".to_string(),
            wallet_address: "0xabc".to_string(),
            amount: "29.99".to_string(),
            currency: "USDT".to_string(),
            method: "blockchain".to_string(),
            plan_id: Uuid::new_v4(),
            status: "pending".to_string(),
            transaction_hash: Some("0xdeadbeef".to_string()),
            network: Some("bsc-mainnet".to_string()),
            metadata: Some(serde_json::json!({})),
            expires_at: None,
            completed_at: None,
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        assert!(json.contains("\"payment_reference\":\"PAY-test\""));
    }
}
