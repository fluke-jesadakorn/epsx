// Payment Bounded Context Repository Ports
// These define the interfaces for data persistence in the Payment bounded context

pub mod subscription_port;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::{
    Payment, PaymentId, PaymentStatus, PaymentAmount,
    TransactionHash, CryptoAddress, PaymentReference
};
use crate::domain::wallet_management::value_objects::WalletAddress;

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

// wave11(track-b): the pre-wave-11 `PaymentContextRepositoryPort`
// declared here used a domain-aggregate surface (`PaymentContext`,
// `PaymentContextId`, `PaymentContextType` with `Result<_, String>`)
// that no live caller exercised. Track B replaces it with a
// wider surface that mirrors the concrete
// `PaymentContextRepositoryAdapter` 1:1 — the port uses the
// Diesel DTOs (`PaymentContextDb`, `NewPaymentContextDb`,
// `UpdatePaymentContextDb`) directly so the admin
// `payment_link_handlers` (and its future
// `web/payments/payment_link_handlers.rs` replacement) can plug
// in via `Arc<dyn PaymentContextRepositoryPort>` without a
// domain↔DB shim layer.
//
// The port trait itself lives in
// `repository_ports/payment_context_port.rs`; the impl is on
// `infrastructure::adapters::repositories::payment_context_repository_adapter::PaymentContextRepositoryAdapter`.
// See `docs/wave8-service-boundary/ROADMAP.md` §4 wave-11
// preconditions item 3.
pub mod payment_context_port;

pub use payment_context_port::PaymentContextRepositoryPort;

// wave11(track-b) re-export: the new
// `SubscriptionRepositoryPort`. See
// `repository_ports/subscription_port.rs` for the full
// docstring and the audit references.
pub use subscription_port::SubscriptionRepositoryPort;

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