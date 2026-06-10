// Diesel Models for Credit Wallet System
//
// Database models for wallet_credits and credit_transactions tables using Diesel ORM

use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable, AsChangeset};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

/// Diesel Queryable model for wallet_credits table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schemas::payments::wallet_credits)]
pub struct WalletCreditDb {
    pub wallet_address: String,
    pub balance: BigDecimal,
    pub pending_balance: BigDecimal,
    pub lifetime_earned: BigDecimal,
    pub lifetime_spent: BigDecimal,
    pub last_transaction_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Diesel Insertable model for creating new wallet_credits record
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schemas::payments::wallet_credits)]
pub struct NewWalletCreditDb {
    pub wallet_address: String,
    pub balance: BigDecimal,
    pub pending_balance: BigDecimal,
    pub lifetime_earned: BigDecimal,
    pub lifetime_spent: BigDecimal,
}

/// Diesel AsChangeset model for updating wallet_credits
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schemas::payments::wallet_credits)]
pub struct UpdateWalletCreditDb {
    pub balance: Option<BigDecimal>,
    pub pending_balance: Option<BigDecimal>,
    pub lifetime_earned: Option<BigDecimal>,
    pub lifetime_spent: Option<BigDecimal>,
    pub last_transaction_at: Option<DateTime<Utc>>,
}

/// Diesel Queryable model for credit_transactions table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schemas::payments::credit_transactions)]
pub struct CreditTransactionDb {
    pub id: Uuid,
    pub wallet_address: String,
    pub amount: BigDecimal,
    pub balance_after: BigDecimal,
    pub tx_type: String,
    pub reference_id: Option<Uuid>,
    pub reference_type: Option<String>,
    pub reason: Option<String>,
    pub granted_by: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Diesel Insertable model for creating new credit transactions
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schemas::payments::credit_transactions)]
pub struct NewCreditTransactionDb {
    pub wallet_address: String,
    pub amount: BigDecimal,
    pub balance_after: BigDecimal,
    pub tx_type: String,
    pub reference_id: Option<Uuid>,
    pub reference_type: Option<String>,
    pub reason: Option<String>,
    pub granted_by: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Form data for granting credits (admin API)
#[derive(Debug, Deserialize, Serialize)]
pub struct GrantCreditsRequest {
    pub wallet_address: String,
    pub amount: BigDecimal,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by: String,
}

/// Form data for revoking credits (admin API)
#[derive(Debug, Deserialize, Serialize)]
pub struct RevokeCreditsRequest {
    pub wallet_address: String,
    pub amount: BigDecimal,
    pub reason: Option<String>,
    pub granted_by: String,
}

/// Response model for credit balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditBalanceResponse {
    pub wallet_address: String,
    pub balance: BigDecimal,
    pub pending_balance: BigDecimal,
    pub available_balance: BigDecimal,
    pub lifetime_earned: BigDecimal,
    pub lifetime_spent: BigDecimal,
    pub last_transaction_at: Option<DateTime<Utc>>,
}

/// Response model for credit transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransactionResponse {
    pub id: Uuid,
    pub wallet_address: String,
    pub amount: BigDecimal,
    pub balance_after: BigDecimal,
    pub tx_type: String,
    pub reference_id: Option<Uuid>,
    pub reference_type: Option<String>,
    pub reason: Option<String>,
    pub granted_by: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Credit statistics for admin dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditStatsResponse {
    pub total_credits_outstanding: BigDecimal,
    pub total_credits_granted_today: BigDecimal,
    pub total_credits_used_today: BigDecimal,
    pub active_users_with_credits: i64,
    pub total_transactions_today: i64,
    pub average_balance: BigDecimal,
}

/// Transaction filters for querying
#[derive(Debug, Clone, Deserialize)]
pub struct CreditTransactionFilters {
    pub wallet_address: Option<String>,
    pub tx_type: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<CreditTransactionDb> for CreditTransactionResponse {
    fn from(db: CreditTransactionDb) -> Self {
        Self {
            id: db.id,
            wallet_address: db.wallet_address,
            amount: db.amount,
            balance_after: db.balance_after,
            tx_type: db.tx_type,
            reference_id: db.reference_id,
            reference_type: db.reference_type,
            reason: db.reason,
            granted_by: db.granted_by,
            expires_at: db.expires_at,
            created_at: db.created_at,
        }
    }
}

impl From<WalletCreditDb> for CreditBalanceResponse {
    fn from(db: WalletCreditDb) -> Self {
        // Available balance = balance - pending_balance
        let available = &db.balance - &db.pending_balance;
        Self {
            wallet_address: db.wallet_address,
            balance: db.balance.clone(),
            pending_balance: db.pending_balance,
            available_balance: available,
            lifetime_earned: db.lifetime_earned,
            lifetime_spent: db.lifetime_spent,
            last_transaction_at: db.last_transaction_at,
        }
    }
}
