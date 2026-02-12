//! Transaction Monitor Service
//!
//! Background service that monitors pending transactions on the blockchain.
//! Polls RPC for transaction status and updates database when confirmed.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use ethers::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::{
    infrastructure::database::{get_diesel_pool, get_payments_pool},
    schemas::payments::payments,
};

/// Transaction monitor service configuration
pub struct TransactionMonitorConfig {
    /// RPC URL for blockchain queries
    pub rpc_url: String,
    /// Payment contract address
    pub contract_address: H160,
    /// Minimum confirmations required
    pub min_confirmations: u64,
    /// Polling interval in seconds
    pub poll_interval_secs: u64,
}

impl Default for TransactionMonitorConfig {
    fn default() -> Self {
        Self {
            rpc_url: std::env::var("BSC_RPC_URL")
                .unwrap_or_else(|_| "https://data-seed-prebsc-1-s1.binance.org:8545".to_string()),
            contract_address: std::env::var("PAYMENT_ESCROW_ADDRESS")
                .ok()
                .and_then(|addr| addr.parse::<H160>().ok())
                .unwrap_or_else(H160::zero),
            min_confirmations: 1,
            poll_interval_secs: 5,
        }
    }
}

/// Background service for monitoring pending transactions
pub struct TransactionMonitorService {
    config: TransactionMonitorConfig,
    provider: Arc<Provider<Http>>,
}

impl TransactionMonitorService {
    /// Create new transaction monitor service
    pub fn new(config: TransactionMonitorConfig) -> Result<Self, String> {
        let provider = Provider::<Http>::try_from(&config.rpc_url)
            .map_err(|e| format!("Failed to create RPC provider: {}", e))?;

        Ok(Self {
            config,
            provider: Arc::new(provider),
        })
    }

    /// Start the background monitoring loop
    pub async fn start(&self) {
        info!(
            "🚀 Starting transaction monitor service (poll interval: {}s)",
            self.config.poll_interval_secs
        );

        let mut error_count = 0;
        // Use a simple loop with sleep instead of interval for dynamic backoff
        loop {
            if let Err(e) = self.check_pending_transactions().await {
                error_count += 1;
                let backoff_secs = (self.config.poll_interval_secs * error_count.min(12) as u64).max(self.config.poll_interval_secs);
                
                error!("Error checking pending transactions (attempt {}): {}", error_count, e);
                if e.contains("relation") && e.contains("does not exist") {
                    error!("💡 HINT: The 'payments' table might be missing. Ensure migrations are run: `diesel migration run --config-file apps/backend/diesel_payments.toml` and PAYMENTS_DATABASE_URL is set correctly.");
                }

                // Sleep with backoff
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
            } else {
                // Reset error count on success
                if error_count > 0 {
                    info!("✅ Transaction monitor service checks recovered");
                    error_count = 0;
                }
                
                // Normal sleep
                tokio::time::sleep(Duration::from_secs(self.config.poll_interval_secs)).await;
            }
        }
    }

    /// Check all pending transactions
    async fn check_pending_transactions(&self) -> Result<(), String> {
        // Get payments database connection
        let payments_pool = get_payments_pool()
            .await
            .map_err(|e| format!("Failed to get payments pool: {}", e))?;
        let mut conn = payments_pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        // Get current block number
        let current_block = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| format!("Failed to get block number: {}", e))?;

        // Query pending transactions
        let pending_payments: Vec<(String, Option<i64>)> = payments::table
            .filter(payments::status.eq("pending"))
            .select((payments::transaction_hash, payments::block_number))
            .load::<(Option<String>, Option<i64>)>(&mut conn)
            .await
            .map_err(|e| format!("Failed to query pending payments: {}", e))?
            .into_iter()
            .filter_map(|(tx_hash, block_num)| tx_hash.map(|h| (h, block_num)))
            .collect();

        if pending_payments.is_empty() {
            trace!("No pending transactions to check");
            return Ok(());
        }

        info!("📋 Checking {} pending transactions", pending_payments.len());

        for (tx_hash, _block_num) in pending_payments {
            if let Err(e) = self
                .check_single_transaction(&tx_hash, current_block.as_u64())
                .await
            {
                error!("Error checking transaction {}: {}", tx_hash, e);
            }
        }

        Ok(())
    }

    /// Check a single transaction and update status
    async fn check_single_transaction(
        &self,
        tx_hash: &str,
        current_block: u64,
    ) -> Result<(), String> {
        let hash = tx_hash
            .parse::<H256>()
            .map_err(|e| format!("Invalid tx hash: {}", e))?;

        // Get transaction receipt
        let receipt = self
            .provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| format!("Failed to get receipt: {}", e))?;

        match receipt {
            Some(receipt) => {
                // Transaction was mined
                let block_number = receipt.block_number.map(|b| b.as_u64()).unwrap_or(0);
                let confirmations = current_block.saturating_sub(block_number) as i32;
                let tx_status = receipt.status.map(|s| s.as_u64()).unwrap_or(0);

                if tx_status == 1 {
                    // Transaction succeeded
                    if confirmations >= self.config.min_confirmations as i32 {
                        // Enough confirmations - mark as confirmed
                        self.finalize_payment(tx_hash, &receipt).await?;
                    } else {
                        // Update confirmations count
                        self.update_confirmations(tx_hash, confirmations, block_number as i64)
                            .await?;
                    }
                } else {
                    // Transaction failed on-chain
                    self.mark_as_failed(tx_hash, "Transaction reverted on-chain")
                        .await?;
                }
            }
            None => {
                // Transaction not yet mined - update last_checked_at
                self.update_last_checked(tx_hash).await?;
            }
        }

        Ok(())
    }

    /// Update confirmation count for a transaction
    async fn update_confirmations(
        &self,
        tx_hash: &str,
        confirmations: i32,
        block_number: i64,
    ) -> Result<(), String> {
        let payments_pool = get_payments_pool()
            .await
            .map_err(|e| format!("Failed to get payments pool: {}", e))?;
        let mut conn = payments_pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        diesel::sql_query(
            r#"
            UPDATE payments 
            SET confirmations = $1, 
                block_number = $2,
                status = 'confirming',
                last_checked_at = NOW()
            WHERE transaction_hash = $3
            "#,
        )
        .bind::<diesel::sql_types::Integer, _>(confirmations)
        .bind::<diesel::sql_types::BigInt, _>(block_number)
        .bind::<diesel::sql_types::Text, _>(tx_hash)
        .execute(&mut conn)
        .await
        .map_err(|e| format!("Failed to update confirmations: {}", e))?;

        debug!(
            "Updated confirmations for {}: {} (block {})",
            tx_hash, confirmations, block_number
        );

        Ok(())
    }

    /// Update last_checked_at timestamp
    async fn update_last_checked(&self, tx_hash: &str) -> Result<(), String> {
        let payments_pool = get_payments_pool()
            .await
            .map_err(|e| format!("Failed to get payments pool: {}", e))?;
        let mut conn = payments_pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        diesel::sql_query(
            r#"
            UPDATE payments 
            SET last_checked_at = NOW()
            WHERE transaction_hash = $1
            "#,
        )
        .bind::<diesel::sql_types::Text, _>(tx_hash)
        .execute(&mut conn)
        .await
        .map_err(|e| format!("Failed to update last_checked_at: {}", e))?;

        Ok(())
    }

    /// Mark transaction as failed
    async fn mark_as_failed(&self, tx_hash: &str, error_message: &str) -> Result<(), String> {
        let payments_pool = get_payments_pool()
            .await
            .map_err(|e| format!("Failed to get payments pool: {}", e))?;
        let mut conn = payments_pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        diesel::sql_query(
            r#"
            UPDATE payments 
            SET status = 'failed',
                error_message = $1,
                last_checked_at = NOW()
            WHERE transaction_hash = $2
            "#,
        )
        .bind::<diesel::sql_types::Text, _>(error_message)
        .bind::<diesel::sql_types::Text, _>(tx_hash)
        .execute(&mut conn)
        .await
        .map_err(|e| format!("Failed to mark as failed: {}", e))?;

        warn!("❌ Transaction {} marked as failed: {}", tx_hash, error_message);

        Ok(())
    }

    /// Finalize a confirmed payment - parse events, assign group, update status
    async fn finalize_payment(
        &self,
        tx_hash: &str,
        receipt: &TransactionReceipt,
    ) -> Result<(), String> {
        info!("✅ Finalizing payment for transaction: {}", tx_hash);

        let payments_pool = get_payments_pool()
            .await
            .map_err(|e| format!("Failed to get payments pool: {}", e))?;
        let mut payments_conn = payments_pool
            .get()
            .await
            .map_err(|e| format!("Failed to get payments connection: {}", e))?;

        let primary_pool = get_diesel_pool()
            .await
            .map_err(|e| format!("Failed to get primary pool: {}", e))?;
        let mut primary_conn = primary_pool
            .get()
            .await
            .map_err(|e| format!("Failed to get primary connection: {}", e))?;

        // Get payment details from database
        let payment: Option<(Uuid, String)> = payments::table
            .filter(payments::transaction_hash.eq(tx_hash))
            .select((payments::plan_id, payments::wallet_address))
            .first::<(Uuid, String)>(&mut payments_conn)
            .await
            .ok();

        let (plan_uuid, wallet_address) = match payment {
            Some(p) => p,
            None => {
                return Err(format!(
                    "Payment not found in database for tx: {}",
                    tx_hash
                ));
            }
        };

        let block_number = receipt.block_number.map(|b| b.as_u64() as i64);
        let expires_at = Utc::now() + chrono::Duration::days(30);

        // 1. Update payment status to confirmed
        diesel::sql_query(
            r#"
            UPDATE payments 
            SET status = 'confirmed',
                block_number = $1,
                confirmations = $2,
                completed_at = NOW(),
                last_checked_at = NOW()
            WHERE transaction_hash = $3
            "#,
        )
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(block_number)
        .bind::<diesel::sql_types::Integer, _>(self.config.min_confirmations as i32)
        .bind::<diesel::sql_types::Text, _>(tx_hash)
        .execute(&mut payments_conn)
        .await
        .map_err(|e| format!("Failed to update payment status: {}", e))?;

        // 2. Ensure wallet_users entry exists
        diesel::sql_query(
            r#"
            INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata)
            VALUES ($1, true, 'Bronze', '{}')
            ON CONFLICT (wallet_address) DO NOTHING
            "#,
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .execute(&mut primary_conn)
        .await
        .ok(); // Non-critical if this fails

        // 2.5 Verify group/plan exists before assignment
        #[derive(diesel::QueryableByName)]
        #[allow(dead_code)]
        struct GroupCheck {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
        }

        let group_check: Option<GroupCheck> = diesel::sql_query(
            "SELECT id, name FROM plans WHERE id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
        .get_result(&mut primary_conn)
        .await
        .ok();

        let plan_name = match group_check {
            Some(g) => g.name,
            None => {
                error!(
                    "❌ Group/Plan {} not found or inactive for wallet {}",
                    plan_uuid, wallet_address
                );
                return Err(format!(
                    "Plan group {} not found or inactive",
                    plan_uuid
                ));
            }
        };

        info!(
            "📦 Assigning group '{}' ({}) to wallet {}",
            plan_name, plan_uuid, wallet_address
        );

        // 2.6 Check for existing assignment (active OR inactive) for this plan
        #[derive(diesel::QueryableByName)]
        struct ExistingAssignment {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            expires_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
        }

        let existing_assignment: Option<ExistingAssignment> = diesel::sql_query(
            "SELECT id, expires_at, is_active FROM wallet_plan_assignments WHERE LOWER(wallet_address) = LOWER($1) AND plan_id = $2 ORDER BY is_active DESC, expires_at DESC LIMIT 1"
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
        .get_result(&mut primary_conn)
        .await
        .optional()
        .ok()
        .flatten();

        let payment_reference = format!("PAY-{}", Uuid::new_v4());

        if let Some(existing) = existing_assignment {
            // CASE 1: REACTIVATE/EXTEND existing assignment
            let base_time = if existing.is_active && existing.expires_at > Utc::now() { existing.expires_at } else { Utc::now() };
            let new_expiry = base_time + chrono::Duration::days(30);

            info!("🔄 {} plan {} for wallet {}. Old expiry: {}, New expiry: {}",
                if existing.is_active { "Extending" } else { "Reactivating" },
                plan_uuid, wallet_address, existing.expires_at, new_expiry);

            // Deactivate other subscription plans first
            diesel::sql_query(
                r#"
                UPDATE wallet_plan_assignments
                SET is_active = false, updated_at = NOW()
                WHERE LOWER(wallet_address) = LOWER($1)
                  AND is_active = true
                  AND plan_id != $2
                  AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_address)
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .execute(&mut primary_conn)
            .await
            .ok();

            diesel::sql_query(
                r#"
                UPDATE wallet_plan_assignments
                SET expires_at = $1, payment_reference = $2, updated_at = NOW(), is_active = true
                WHERE id = $3
                "#
            )
            .bind::<diesel::sql_types::Timestamptz, _>(new_expiry)
            .bind::<diesel::sql_types::Text, _>(&payment_reference)
            .bind::<diesel::sql_types::Uuid, _>(existing.id)
            .execute(&mut primary_conn)
            .await
            .map_err(|e| {
                error!("❌ Failed to extend plan: {}", e);
                format!("Failed to extend plan: {}", e)
            })?;

        } else {
            // CASE 2: NEW ASSIGNMENT (no prior record for this wallet+plan)
            diesel::sql_query(
                r#"
                UPDATE wallet_plan_assignments
                SET is_active = false, updated_at = NOW()
                WHERE LOWER(wallet_address) = LOWER($1)
                  AND is_active = true
                  AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_address)
            .execute(&mut primary_conn)
            .await
            .ok();

            diesel::sql_query(
                r#"
                INSERT INTO wallet_plan_assignments (
                    wallet_address, plan_id, assigned_at, expires_at, is_active,
                    assignment_source, assignment_reason, payment_reference,
                    auto_renew, assignment_metadata
                )
                VALUES ($1, $2, NOW(), $3, true, 'payment', 'Plan purchase via blockchain payment', $4, false, '{}')
                "#
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_address)
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .bind::<diesel::sql_types::Timestamptz, _>(expires_at)
            .bind::<diesel::sql_types::Text, _>(&payment_reference)
            .execute(&mut primary_conn)
            .await
            .map_err(|e| {
                error!(
                    "❌ Failed to assign plan {} to wallet {}: {}",
                    plan_uuid, wallet_address, e
                );
                format!("Failed to assign plan: {}", e)
            })?;
        }

        info!(
            "✅ Group assignment successful: wallet={}, group={}",
            wallet_address, plan_uuid
        );

        // 4. Update wallet tier
        diesel::sql_query(
            r#"
            UPDATE wallet_users 
            SET tier_level = 'Gold', updated_at = NOW()
            WHERE wallet_address = $1
            "#,
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .execute(&mut primary_conn)
        .await
        .ok(); // Non-critical

        info!(
            "✅ Payment finalized: wallet={}, plan='{}' ({}), expires={}, ref={}",
            wallet_address, plan_name, plan_uuid, expires_at, payment_reference
        );

        Ok(())
    }
}

/// Start the transaction monitor as a background task
pub fn spawn_transaction_monitor() {
    tokio::spawn(async {
        let config = TransactionMonitorConfig::default();

        if config.contract_address == H160::zero() {
            warn!("⚠️ PAYMENT_ESCROW_ADDRESS not set, transaction monitor disabled");
            return;
        }

        match TransactionMonitorService::new(config) {
            Ok(service) => {
                service.start().await;
            }
            Err(e) => {
                error!("Failed to start transaction monitor: {}", e);
            }
        }
    });
}
