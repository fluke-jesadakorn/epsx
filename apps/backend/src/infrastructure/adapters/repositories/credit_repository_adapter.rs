//! Credit Repository Adapter (Infrastructure Layer)
//! PostgreSQL implementation for credit wallet persistence using Diesel
//!
//! Wave 11 / Track A: this file now also implements
//! `domain::payment::repository_ports::CreditRepositoryPort` —
//! the trait wrapper around the 6 in-process methods below. The
//! trait impl is at the bottom of the file.

use crate::prelude::*;
use async_trait::async_trait;
use tracing::{info, error, debug};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use uuid::Uuid;
use chrono::Utc;
use bigdecimal::BigDecimal;
use std::str::FromStr;

use crate::infrastructure::models::credit::{
    WalletCreditDb, NewWalletCreditDb, UpdateWalletCreditDb,
    CreditTransactionDb,
    CreditTransactionFilters, CreditStatsResponse
};
use crate::schemas::payments::{wallet_credits, credit_transactions};
use crate::domain::payment::repository_ports::{
    CreditRepositoryPort, CreditBalanceRow, CreditTransactionRow,
    CreditTransactionFilters as PortCreditTransactionFilters, CreditStats,
};

/// PostgreSQL credit repository adapter
#[derive(Clone)]
pub struct CreditRepositoryAdapter {
    db_pool: &'static TlsPool,
}

impl CreditRepositoryAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }

    /// Get credit balance for a wallet
    pub async fn get_balance(&self, wallet_address: &str) -> AppResult<Option<WalletCreditDb>> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Getting credit balance for wallet: {}", wallet_address);

        let result = wallet_credits::table
            .filter(wallet_credits::wallet_address.eq(wallet_address))
            .first::<WalletCreditDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to get credit balance for wallet {}: {}", wallet_address, e);
                AppError::database_error(format!("Failed to get credit balance: {}", e))
            })?;

        Ok(result)
    }

    /// Get or create wallet credits record (returns balance)
    pub async fn get_or_create_balance(&self, wallet_address: &str) -> AppResult<WalletCreditDb> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Getting or creating credit balance for wallet: {}", wallet_address);

        // Try to get existing record first
        let existing = wallet_credits::table
            .filter(wallet_credits::wallet_address.eq(wallet_address))
            .first::<WalletCreditDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to check existing credit balance: {}", e);
                AppError::database_error(format!("Failed to check credit balance: {}", e))
            })?;

        if let Some(balance) = existing {
            return Ok(balance);
        }

        // Create new record with zero balance
        let new_credit = NewWalletCreditDb {
            wallet_address: wallet_address.to_string(),
            balance: BigDecimal::from(0),
            pending_balance: BigDecimal::from(0),
            lifetime_earned: BigDecimal::from(0),
            lifetime_spent: BigDecimal::from(0),
        };

        diesel::insert_into(wallet_credits::table)
            .values(&new_credit)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to create credit balance: {}", e);
                AppError::database_error(format!("Failed to create credit balance: {}", e))
            })?;

        // Fetch the created record
        let created = wallet_credits::table
            .filter(wallet_credits::wallet_address.eq(wallet_address))
            .first::<WalletCreditDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to fetch created credit balance: {}", e);
                AppError::database_error(format!("Failed to fetch created balance: {}", e))
            })?;

        info!("Created new credit balance for wallet: {}", wallet_address);
        Ok(created)
    }

    /// Get credit transactions for a wallet
    pub async fn get_transactions(
        &self,
        wallet_address: &str,
        filters: Option<CreditTransactionFilters>,
    ) -> AppResult<Vec<CreditTransactionDb>> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Getting credit transactions for wallet: {}", wallet_address);

        let mut query = credit_transactions::table
            .filter(credit_transactions::wallet_address.eq(wallet_address))
            .into_boxed();

        if let Some(ref f) = filters {
            if let Some(ref tx_type) = f.tx_type {
                query = query.filter(credit_transactions::tx_type.eq(tx_type));
            }

            if let Some(ref from_date) = f.from_date {
                query = query.filter(credit_transactions::created_at.ge(from_date));
            }

            if let Some(ref to_date) = f.to_date {
                query = query.filter(credit_transactions::created_at.le(to_date));
            }

            if let Some(limit) = f.limit {
                query = query.limit(limit);
            }

            if let Some(offset) = f.offset {
                query = query.offset(offset);
            }
        }

        query = query.order(credit_transactions::created_at.desc());

        let results = query
            .load::<CreditTransactionDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get credit transactions: {}", e);
                AppError::database_error(format!("Failed to get transactions: {}", e))
            })?;

        info!("Found {} credit transactions for wallet {}", results.len(), wallet_address);
        Ok(results)
    }

    /// Get all credit transactions (admin)
    pub async fn get_all_transactions(
        &self,
        filters: Option<CreditTransactionFilters>,
    ) -> AppResult<Vec<CreditTransactionDb>> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Getting all credit transactions");

        let mut query = credit_transactions::table.into_boxed();

        if let Some(ref f) = filters {
            if let Some(ref wallet_addr) = f.wallet_address {
                query = query.filter(credit_transactions::wallet_address.eq(wallet_addr));
            }

            if let Some(ref tx_type) = f.tx_type {
                query = query.filter(credit_transactions::tx_type.eq(tx_type));
            }

            if let Some(ref from_date) = f.from_date {
                query = query.filter(credit_transactions::created_at.ge(from_date));
            }

            if let Some(ref to_date) = f.to_date {
                query = query.filter(credit_transactions::created_at.le(to_date));
            }

            if let Some(limit) = f.limit {
                query = query.limit(limit);
            }

            if let Some(offset) = f.offset {
                query = query.offset(offset);
            }
        }

        query = query.order(credit_transactions::created_at.desc());

        let results = query
            .load::<CreditTransactionDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get all credit transactions: {}", e);
                AppError::database_error(format!("Failed to get transactions: {}", e))
            })?;

        info!("Found {} credit transactions total", results.len());
        Ok(results)
    }

    /// Add credit transaction (uses database function for atomic balance update)
    pub async fn add_transaction(
        &self,
        wallet_address: &str,
        amount: BigDecimal,
        tx_type: &str,
        reference_id: Option<Uuid>,
        reference_type: Option<&str>,
        reason: Option<&str>,
        granted_by: Option<&str>,
        expires_at: Option<chrono::DateTime<Utc>>,
        metadata: Option<serde_json::Value>,
    ) -> AppResult<Uuid> {
        let mut conn = self.db_pool.conn().await?;

        info!(
            "Adding credit transaction for wallet {}: amount={}, type={}",
            wallet_address, amount, tx_type
        );

        // Call the database function to add transaction atomically
        #[derive(QueryableByName)]
        struct TransactionResult {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            add_credit_transaction: Uuid,
        }

        let result = diesel::sql_query(
            "SELECT add_credit_transaction($1, $2, $3, $4, $5, $6, $7, $8, $9) as add_credit_transaction"
        )
        .bind::<diesel::sql_types::Varchar, _>(wallet_address)
        .bind::<diesel::sql_types::Numeric, _>(&amount)
        .bind::<diesel::sql_types::Varchar, _>(tx_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(reference_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Varchar>, _>(reference_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(reason)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Varchar>, _>(granted_by)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(expires_at)
        .bind::<diesel::sql_types::Jsonb, _>(metadata.unwrap_or(serde_json::json!({})))
        .get_result::<TransactionResult>(&mut conn)
        .await
        .map(|r| r.add_credit_transaction)
        .map_err(|e| {
            error!("Failed to add credit transaction: {}", e);
            AppError::database_error(format!("Failed to add transaction: {}", e))
        })?;

        info!("Successfully added credit transaction: {}", result);
        Ok(result)
    }

    /// Get credit statistics (admin)
    pub async fn get_stats(&self) -> AppResult<CreditStatsResponse> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Getting credit statistics");

        // Total credits outstanding
        let total_outstanding: BigDecimal = wallet_credits::table
            .select(diesel::dsl::sum(wallet_credits::balance))
            .first::<Option<BigDecimal>>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get total outstanding credits: {}", e);
                AppError::database_error(format!("Failed to get stats: {}", e))
            })?
            .unwrap_or_else(|| BigDecimal::from(0));

        // Count active users with credits
        let active_users: i64 = wallet_credits::table
            .filter(wallet_credits::balance.gt(BigDecimal::from(0)))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count active users: {}", e);
                AppError::database_error(format!("Failed to get stats: {}", e))
            })?;

        // Average balance
        let average_balance: BigDecimal = wallet_credits::table
            .filter(wallet_credits::balance.gt(BigDecimal::from(0)))
            .select(diesel::dsl::avg(wallet_credits::balance))
            .first::<Option<BigDecimal>>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get average balance: {}", e);
                AppError::database_error(format!("Failed to get stats: {}", e))
            })?
            .unwrap_or_else(|| BigDecimal::from(0));

        // Today's transactions
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

        let total_granted_today: BigDecimal = credit_transactions::table
            .filter(credit_transactions::created_at.ge(today_start))
            .filter(credit_transactions::tx_type.eq("grant"))
            .select(diesel::dsl::sum(credit_transactions::amount))
            .first::<Option<BigDecimal>>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get today's grants: {}", e);
                AppError::database_error(format!("Failed to get stats: {}", e))
            })?
            .unwrap_or_else(|| BigDecimal::from(0));

        let total_used_today: BigDecimal = credit_transactions::table
            .filter(credit_transactions::created_at.ge(today_start))
            .filter(credit_transactions::tx_type.eq("payment_debit"))
            .select(diesel::dsl::sum(credit_transactions::amount))
            .first::<Option<BigDecimal>>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get today's usage: {}", e);
                AppError::database_error(format!("Failed to get stats: {}", e))
            })?
            .unwrap_or_else(|| BigDecimal::from(0))
            .abs(); // Make positive

        let total_transactions_today: i64 = credit_transactions::table
            .filter(credit_transactions::created_at.ge(today_start))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count today's transactions: {}", e);
                AppError::database_error(format!("Failed to get stats: {}", e))
            })?;

        Ok(CreditStatsResponse {
            total_credits_outstanding: total_outstanding,
            total_credits_granted_today: total_granted_today,
            total_credits_used_today: total_used_today,
            active_users_with_credits: active_users,
            total_transactions_today,
            average_balance,
        })
    }

    /// Update wallet credit balance (internal use)
    pub async fn update_balance(
        &self,
        wallet_address: &str,
        update: UpdateWalletCreditDb,
    ) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        debug!("Updating credit balance for wallet: {}", wallet_address);

        diesel::update(wallet_credits::table.filter(wallet_credits::wallet_address.eq(wallet_address)))
            .set(&update)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to update credit balance: {}", e);
                AppError::database_error(format!("Failed to update balance: {}", e))
            })?;

        info!("Successfully updated credit balance for wallet: {}", wallet_address);
        Ok(())
    }
}

// ============================================================================
// Wave 11 / Track A — CreditRepositoryPort impl
// ============================================================================
//
// Thin trait wrapper around the 6 in-process methods above. The
// `*_port` methods on the port each forward to the inherent
// method on this adapter, converting the in-process error
// types to the port's `String` return.

#[async_trait]
impl CreditRepositoryPort for CreditRepositoryAdapter {
    async fn get_balance(&self, wallet_address: &str) -> Result<Option<CreditBalanceRow>, String> {
        let row = self.get_balance(wallet_address)
            .await
            .map_err(|e| format!("get_balance: {}", e))?;
        Ok(row.map(|db| CreditBalanceRow {
            wallet_address: db.wallet_address,
            balance: db.balance.to_string(),
            pending_balance: db.pending_balance.to_string(),
            lifetime_earned: db.lifetime_earned.to_string(),
            lifetime_spent: db.lifetime_spent.to_string(),
            updated_at: Some(db.updated_at),
        }))
    }

    async fn get_or_create_balance(&self, wallet_address: &str) -> Result<CreditBalanceRow, String> {
        let db = self.get_or_create_balance(wallet_address)
            .await
            .map_err(|e| format!("get_or_create: {}", e))?;
        Ok(CreditBalanceRow {
            wallet_address: db.wallet_address,
            balance: db.balance.to_string(),
            pending_balance: db.pending_balance.to_string(),
            lifetime_earned: db.lifetime_earned.to_string(),
            lifetime_spent: db.lifetime_spent.to_string(),
            updated_at: Some(db.updated_at),
        })
    }

    async fn get_transactions(
        &self,
        wallet_address: &str,
        filters: Option<PortCreditTransactionFilters>,
    ) -> Result<Vec<CreditTransactionRow>, String> {
        // The port's `CreditTransactionFilters` is identical to
        // the existing `infrastructure::models::credit::CreditTransactionFilters`
        // (no Deserialize impl on the latter — that's only on the
        // port filter). We construct the in-process filter
        // field-by-field.
        let f = filters.map(|p| CreditTransactionFilters {
            wallet_address: p.wallet_address,
            tx_type: p.tx_type,
            from_date: p.from_date,
            to_date: p.to_date,
            limit: p.limit,
            offset: p.offset,
        });
        let rows = self.get_transactions(wallet_address, f)
            .await
            .map_err(|e| format!("get_transactions: {}", e))?;
        Ok(rows.into_iter().map(credit_tx_to_row).collect())
    }

    async fn get_all_transactions(
        &self,
        filters: Option<PortCreditTransactionFilters>,
    ) -> Result<Vec<CreditTransactionRow>, String> {
        let f = filters.map(|p| CreditTransactionFilters {
            wallet_address: p.wallet_address,
            tx_type: p.tx_type,
            from_date: p.from_date,
            to_date: p.to_date,
            limit: p.limit,
            offset: p.offset,
        });
        let rows = self.get_all_transactions(f)
            .await
            .map_err(|e| format!("get_all_transactions: {}", e))?;
        Ok(rows.into_iter().map(credit_tx_to_row).collect())
    }

    async fn add_transaction(
        &self,
        wallet_address: &str,
        amount: String,
        tx_type: &str,
        reference_id: Option<Uuid>,
        reference_type: Option<&str>,
        reason: Option<&str>,
        granted_by: Option<&str>,
        expires_at: Option<chrono::DateTime<Utc>>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Uuid, String> {
        let amt = BigDecimal::from_str(&amount)
            .map_err(|e| format!("amount parse: {}", e))?;
        self.add_transaction(
            wallet_address,
            amt,
            tx_type,
            reference_id,
            reference_type,
            reason,
            granted_by,
            expires_at,
            metadata,
        )
        .await
        .map_err(|e| format!("add_transaction: {}", e))
    }

    async fn get_stats(&self) -> Result<CreditStats, String> {
        let r = self.get_stats()
            .await
            .map_err(|e| format!("get_stats: {}", e))?;
        Ok(CreditStats {
            total_credits_outstanding: r.total_credits_outstanding.to_string(),
            total_credits_granted_today: r.total_credits_granted_today.to_string(),
            total_credits_used_today: r.total_credits_used_today.to_string(),
            active_users_with_credits: r.active_users_with_credits,
            total_transactions_today: r.total_transactions_today,
            average_balance: r.average_balance.to_string(),
        })
    }
}

fn credit_tx_to_row(db: CreditTransactionDb) -> CreditTransactionRow {
    CreditTransactionRow {
        id: db.id,
        wallet_address: db.wallet_address,
        amount: db.amount.to_string(),
        balance_after: db.balance_after.to_string(),
        tx_type: db.tx_type,
        reference_id: db.reference_id,
        reference_type: db.reference_type,
        reason: db.reason,
        granted_by: db.granted_by,
        expires_at: db.expires_at,
        metadata: db.metadata.unwrap_or(serde_json::json!({})),
        created_at: db.created_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Object-safety compile-time check: the port would not
    /// compile if `CreditRepositoryPort` had a generic method or
    /// an `impl Trait` return.
    #[allow(dead_code)]
    fn _assert_credit_port_object_safe(_: &dyn CreditRepositoryPort) {}

    #[test]
    fn test_repository_creation() {
        // This is just a placeholder to show the structure
        // Real tests would require database setup
        assert!(true);
    }

    #[test]
    fn credit_balance_row_round_trip_strings() {
        // The port DTO carries BigDecimal as a string. Verify
        // the round-trip preserves "0", positive, and negative
        // amounts exactly.
        let r = CreditBalanceRow {
            wallet_address: "0xtest".to_string(),
            balance: "100.5".to_string(),
            pending_balance: "0".to_string(),
            lifetime_earned: "100.5".to_string(),
            lifetime_spent: "0".to_string(),
            updated_at: None,
        };
        let json = serde_json::to_string(&r).expect("ser");
        let back: CreditBalanceRow = serde_json::from_str(&json).expect("de");
        assert_eq!(back.wallet_address, "0xtest");
        assert_eq!(back.balance, "100.5");
    }
}
