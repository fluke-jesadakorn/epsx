//! Credit Repository Adapter (Infrastructure Layer)
//! PostgreSQL implementation for credit wallet persistence using Diesel

use crate::prelude::*;
use tracing::{info, error, debug};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use uuid::Uuid;
use chrono::Utc;
use bigdecimal::BigDecimal;

use crate::infrastructure::models::credit::{
    WalletCreditDb, NewWalletCreditDb, UpdateWalletCreditDb,
    CreditTransactionDb, NewCreditTransactionDb,
    CreditTransactionFilters, CreditStatsResponse
};
use crate::schemas::payments::{wallet_credits, credit_transactions};

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
        let result: Uuid = diesel::sql_query(
            "SELECT add_credit_transaction($1, $2, $3, $4, $5, $6, $7, $8, $9)"
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
        .get_result::<(Uuid,)>(&mut conn)
        .await
        .map(|row| row.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are placeholder tests
    // In a real implementation, you would use a test database

    #[test]
    fn test_repository_creation() {
        // This is just a placeholder to show the structure
        // Real tests would require database setup
        assert!(true);
    }
}
