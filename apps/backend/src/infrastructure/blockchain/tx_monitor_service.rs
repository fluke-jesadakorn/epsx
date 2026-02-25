//! Transaction Monitor Service
//!
//! Background service that monitors pending transactions on the blockchain.
//! Polls RPC for transaction status and updates database when confirmed.
//! Verifies ERC20 Transfer events to ensure correct recipient, token, and amount.

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use ethers::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::{
    infrastructure::database::{get_diesel_pool, get_payments_pool},
    schemas::payments::payments,
};

/// ERC20 Transfer(address,address,uint256) event topic
/// keccak256("Transfer(address,address,uint256)")
const ERC20_TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

/// Transaction monitor service configuration
pub struct TransactionMonitorConfig {
    /// RPC URL for blockchain queries
    pub rpc_url: String,
    /// Payment receiver addresses (where tokens should be sent)
    pub receiver_addresses: HashSet<H160>,
    /// Minimum confirmations required (15 for BSC mainnet)
    pub min_confirmations: u64,
    /// Polling interval in seconds
    pub poll_interval_secs: u64,
    /// Supported ERC20 token contract addresses (lowercase)
    pub supported_tokens: HashSet<String>,
    /// Max age in hours for pending payments before expiry
    pub pending_ttl_hours: i64,
}

impl Default for TransactionMonitorConfig {
    fn default() -> Self {
        let mut receiver_addresses = HashSet::new();

        if let Ok(addr_str) = std::env::var("PAYMENT_RECEIVER_ADDRESS") {
            for s in addr_str.split(',') {
                if let Ok(addr) = s.trim().parse::<H160>() {
                    receiver_addresses.insert(addr);
                }
            }
        }
        
        if let Ok(addr_str) = std::env::var("PAYMENT_ESCROW_ADDRESS") {
            for s in addr_str.split(',') {
                if let Ok(addr) = s.trim().parse::<H160>() {
                    receiver_addresses.insert(addr);
                }
            }
        }

        if receiver_addresses.is_empty() {
            error!("SECURITY: No PAYMENT_RECEIVER_ADDRESS or PAYMENT_ESCROW_ADDRESS configured. Transaction monitor will reject all payments.");
        }

        let tokens_str = std::env::var("SUPPORTED_PAYMENT_TOKENS").unwrap_or_else(|_| {
            "0x55d398326f99059fF775485246999027B3197955,0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d,0x337610d27c682E347C9cD60BD4b3b107C9d34dDD,0x64544969ed7EBf5f083679233325356EbE738930".to_string()
        });

        let supported_tokens: HashSet<String> = tokens_str
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        let is_mainnet = std::env::var("NEXT_PUBLIC_BLOCKCHAIN_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string())
            == "mainnet";

        Self {
            rpc_url: std::env::var("BSC_RPC_URL")
                .unwrap_or_else(|_| "https://bsc-dataseed.binance.org".to_string()),
            receiver_addresses,
            min_confirmations: if is_mainnet { 50 } else { 3 },
            poll_interval_secs: 5,
            supported_tokens,
            pending_ttl_hours: 24,
        }
    }
}

/// Parsed ERC20 Transfer event from tx receipt logs
#[derive(Debug)]
struct Erc20Transfer {
    from: H160,
    token: H160,
    to: H160,
    amount: U256,
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
            "Transaction monitor started: confirmations={}, receivers={:?}, tokens={}, ttl={}h",
            self.config.min_confirmations,
            self.config.receiver_addresses,
            self.config.supported_tokens.len(),
            self.config.pending_ttl_hours,
        );

        let mut error_count: u64 = 0;
        loop {
            // Expire stale pending payments
            if let Err(e) = self.expire_stale_payments().await {
                warn!("Failed to expire stale payments: {}", e);
            }

            if let Err(e) = self.check_pending_transactions().await {
                error_count += 1;
                let backoff = self.config.poll_interval_secs * error_count.min(12);
                let backoff = backoff.max(self.config.poll_interval_secs);

                error!(
                    "Error checking pending transactions (attempt {}): {}",
                    error_count, e
                );
                if e.contains("relation") && e.contains("does not exist") {
                    error!("HINT: The 'payments' table might be missing. Run migrations.");
                }

                tokio::time::sleep(Duration::from_secs(backoff)).await;
            } else {
                if error_count > 0 {
                    info!("Transaction monitor recovered");
                    error_count = 0;
                }
                tokio::time::sleep(Duration::from_secs(self.config.poll_interval_secs)).await;
            }
        }
    }

    /// Parse ERC20 Transfer events from transaction receipt logs
    fn parse_erc20_transfers(&self, receipt: &TransactionReceipt) -> Vec<Erc20Transfer> {
        let transfer_topic = ERC20_TRANSFER_TOPIC
            .parse::<H256>()
            .expect("Invalid transfer topic constant");

        receipt
            .logs
            .iter()
            .filter_map(|log| {
                // ERC20 Transfer has 3 topics: [Transfer sig, from, to]
                if log.topics.len() < 3 || log.topics[0] != transfer_topic {
                    return None;
                }
                // data contains the uint256 amount
                if log.data.len() < 32 {
                    return None;
                }

                let from = H160::from(log.topics[1]);
                let to = H160::from(log.topics[2]);
                let amount = U256::from_big_endian(&log.data[..32]);
                let token = log.address;

                Some(Erc20Transfer { from, token, to, amount })
            })
            .collect()
    }

    /// Verify that tx receipt contains a valid ERC20 Transfer to our receiver
    /// Returns (verified_amount_raw, token_address) or error
    fn verify_transfer_logs(
        &self,
        receipt: &TransactionReceipt,
        expected_amount: &bigdecimal::BigDecimal,
        expected_currency: &str,
        expected_sender_wallet: &str,
    ) -> Result<(U256, H160), String> {
        let transfers = self.parse_erc20_transfers(receipt);

        if transfers.is_empty() {
            return Err("No ERC20 Transfer events in transaction".to_string());
        }

        // Parse the expected sender wallet string to H160 to compare securely
        let expected_from = expected_sender_wallet
            .parse::<H160>()
            .map_err(|e| format!("Invalid wallet_address format: {}", e))?;

        // Find a Transfer that sends to any of our allowed receivers with a supported token
        let matching: Vec<&Erc20Transfer> = transfers
            .iter()
            .filter(|t| {
                t.from == expected_from
                    && self.config.receiver_addresses.contains(&t.to)
                    && self
                        .config
                        .supported_tokens
                        .contains(&format!("{:#x}", t.token)) // ensuring it formats as 0x... lowercase
            })
            .collect();

        if matching.is_empty() {
            return Err(format!(
                "No Transfer to valid receivers {:?} from {:?} with supported token. Found {} transfers.",
                self.config.receiver_addresses,
                expected_from,
                transfers.len()
            ));
        }

        // Use the largest matching transfer (in case of multiple)
        let best = matching
            .iter()
            .max_by_key(|t| t.amount)
            .expect("matching is non-empty");

        // Verify amount: convert on-chain raw amount to decimal
        // BSC stablecoins use 18 decimals
        let decimals = match expected_currency.to_uppercase().as_str() {
            "USDT" | "USDC" | "DAI" | "BUSD" => 18u32,
            _ => 18u32,
        };

        let divisor = U256::from(10u64).pow(U256::from(decimals));
        let on_chain_whole = best.amount / divisor;
        let on_chain_frac = best.amount % divisor;

        // Build decimal string for comparison
        let on_chain_str = if on_chain_frac.is_zero() {
            format!("{}", on_chain_whole)
        } else {
            let frac_str = format!("{:0>width$}", on_chain_frac, width = decimals as usize);
            let trimmed = frac_str.trim_end_matches('0');
            format!("{}.{}", on_chain_whole, trimmed)
        };

        let on_chain_decimal = on_chain_str
            .parse::<bigdecimal::BigDecimal>()
            .map_err(|e| format!("Failed to parse on-chain amount: {}", e))?;

        // Allow 0.1% tolerance for rounding
        use bigdecimal::ToPrimitive;
        let expected_f64 = expected_amount.to_f64().unwrap_or(0.0);
        let actual_f64 = on_chain_decimal.to_f64().unwrap_or(0.0);

        if expected_f64 > 0.0 && actual_f64 < expected_f64 * 0.999 {
            return Err(format!(
                "Amount mismatch: expected ${}, on-chain ${}",
                expected_amount, on_chain_decimal
            ));
        }

        info!(
            "Transfer verified: token={:?}, to={:?}, amount={}",
            best.token, best.to, on_chain_decimal
        );

        Ok((best.amount, best.token))
    }

    /// Check all pending transactions
    async fn check_pending_transactions(&self) -> Result<(), String> {
        let payments_pool = get_payments_pool()
            .await
            .map_err(|e| format!("Failed to get payments pool: {}", e))?;
        let mut conn = payments_pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        let current_block = self
            .provider
            .get_block_number()
            .await
            .map_err(|e| format!("Failed to get block number: {}", e))?;

        // Query pending/confirming transactions (not expired, not credit-only)
        let pending_payments: Vec<(String, Option<i64>)> = payments::table
            .filter(
                payments::status
                    .eq("pending")
                    .or(payments::status.eq("confirming")),
            )
            .filter(payments::transaction_hash.is_not_null())
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

        info!("Checking {} pending transactions", pending_payments.len());

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

        let receipt = self
            .provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| format!("Failed to get receipt: {}", e))?;

        match receipt {
            Some(receipt) => {
                let block_number = receipt.block_number.map(|b| b.as_u64()).unwrap_or(0);
                let confirmations = current_block.saturating_sub(block_number) as i32;
                let tx_status = receipt.status.map(|s| s.as_u64()).unwrap_or(0);

                if tx_status == 1 {
                    if confirmations >= self.config.min_confirmations as i32 {
                        self.finalize_payment(tx_hash, &receipt).await?;
                    } else {
                        self.update_confirmations(tx_hash, confirmations, block_number as i64)
                            .await?;
                    }
                } else {
                    self.mark_as_failed(tx_hash, "Transaction reverted on-chain")
                        .await?;
                }
            }
            None => {
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

        warn!("Transaction {} marked as failed: {}", tx_hash, error_message);

        Ok(())
    }

    /// Expire pending payments older than TTL
    async fn expire_stale_payments(&self) -> Result<(), String> {
        let payments_pool = get_payments_pool()
            .await
            .map_err(|e| format!("Failed to get payments pool: {}", e))?;
        let mut conn = payments_pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        let ttl_hours = self.config.pending_ttl_hours;
        let expired_count = diesel::sql_query(
            r#"
            UPDATE payments
            SET status = 'expired',
                error_message = 'Payment expired: pending too long',
                last_checked_at = NOW()
            WHERE status = 'pending'
              AND created_at < NOW() - ($1 || ' hours')::INTERVAL
            "#,
        )
        .bind::<diesel::sql_types::BigInt, _>(ttl_hours)
        .execute(&mut conn)
        .await
        .map_err(|e| format!("Failed to expire stale payments: {}", e))?;

        if expired_count > 0 {
            warn!("Expired {} stale pending payments (>{}h old)", expired_count, ttl_hours);
        }

        Ok(())
    }

    /// Finalize a confirmed payment - verify on-chain, assign plan, update status
    async fn finalize_payment(
        &self,
        tx_hash: &str,
        receipt: &TransactionReceipt,
    ) -> Result<(), String> {
        info!("Finalizing payment for transaction: {}", tx_hash);

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

        // Get payment details from database (amount + currency for verification)
        #[derive(diesel::QueryableByName)]
        struct PaymentRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            plan_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            amount: bigdecimal::BigDecimal,
            #[diesel(sql_type = diesel::sql_types::Text)]
            currency: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            metadata: Option<serde_json::Value>,
        }

        let payment: Option<PaymentRow> = diesel::sql_query(
            "SELECT plan_id, wallet_address, amount, currency, metadata FROM payments WHERE transaction_hash = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Text, _>(tx_hash)
        .get_result(&mut payments_conn)
        .await
        .ok();

        let payment = match payment {
            Some(p) => p,
            None => {
                return Err(format!("Payment not found in database for tx: {}", tx_hash));
            }
        };

        // M5: Check payment expiry — reject if created_at + TTL has passed
        #[derive(diesel::QueryableByName)]
        struct PaymentTimestamp {
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<Utc>,
        }
        let ts: Option<PaymentTimestamp> = diesel::sql_query(
            "SELECT created_at FROM payments WHERE transaction_hash = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Text, _>(tx_hash)
        .get_result(&mut payments_conn)
        .await
        .ok();

        if let Some(ts) = ts {
            let age_hours = (Utc::now() - ts.created_at).num_hours();
            if age_hours > self.config.pending_ttl_hours {
                self.mark_as_failed(tx_hash, "Payment expired before blockchain confirmation").await?;
                return Err(format!("Payment expired ({}h old, limit {}h)", age_hours, self.config.pending_ttl_hours));
            }
        }

        let plan_uuid = payment.plan_id;
        let wallet_address = payment.wallet_address;

        // Determine the blockchain amount to verify
        // If credits were used, verify only the blockchain portion
        let blockchain_amount = payment
            .metadata
            .as_ref()
            .and_then(|m| m.get("blockchain_amount"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<bigdecimal::BigDecimal>().ok())
            .unwrap_or_else(|| payment.amount.clone());

        // Verify ERC20 Transfer events in the receipt
        let (verified_amount, verified_token) =
            self.verify_transfer_logs(receipt, &blockchain_amount, &payment.currency, &wallet_address)?;

        info!(
            "On-chain verification passed: tx={}, amount_raw={}, token={:?}",
            tx_hash, verified_amount, verified_token
        );

        let block_number = receipt.block_number.map(|b| b.as_u64() as i64);
        let expires_at = Utc::now() + chrono::Duration::days(30);
        let token_addr = format!("{:?}", verified_token);

        // 1. Update payment status to confirmed with verified on-chain data
        diesel::sql_query(
            r#"
            UPDATE payments
            SET status = 'confirmed',
                block_number = $1,
                confirmations = $2,
                token_address = $3,
                completed_at = NOW(),
                last_checked_at = NOW()
            WHERE transaction_hash = $4
            "#,
        )
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(block_number)
        .bind::<diesel::sql_types::Integer, _>(self.config.min_confirmations as i32)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(&token_addr))
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
        .ok();

        // 3. Verify plan exists
        #[derive(diesel::QueryableByName)]
        #[allow(dead_code)]
        struct GroupCheck {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            tier_level: i32,
        }

        let group_check: Option<GroupCheck> = diesel::sql_query(
            "SELECT id, name, tier_level FROM plans WHERE id = $1 AND is_active = true",
        )
        .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
        .get_result(&mut primary_conn)
        .await
        .ok();

        let (plan_name, tier_level) = match group_check {
            Some(g) => (g.name, g.tier_level),
            None => {
                error!(
                    "Plan {} not found or inactive for wallet {}",
                    plan_uuid, wallet_address
                );
                return Err(format!("Plan {} not found or inactive", plan_uuid));
            }
        };

        info!(
            "Assigning plan '{}' ({}) to wallet {}",
            plan_name, plan_uuid, wallet_address
        );

        // 4. Check for existing assignment
        #[derive(diesel::QueryableByName)]
        struct ExistingAssignment {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            expires_at: chrono::DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
        }

        let existing: Option<ExistingAssignment> = diesel::sql_query(
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

        if let Some(existing) = existing {
            let base_time = if existing.is_active && existing.expires_at > Utc::now() {
                existing.expires_at
            } else {
                Utc::now()
            };
            let new_expiry = base_time + chrono::Duration::days(30);

            info!(
                "{} plan {} for wallet {}. Old: {}, New: {}",
                if existing.is_active {
                    "Extending"
                } else {
                    "Reactivating"
                },
                plan_uuid,
                wallet_address,
                existing.expires_at,
                new_expiry
            );

            // Deactivate other subscription plans
            diesel::sql_query(
                r#"
                UPDATE wallet_plan_assignments
                SET is_active = false, updated_at = NOW()
                WHERE LOWER(wallet_address) = LOWER($1)
                  AND is_active = true
                  AND plan_id != $2
                  AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
                "#,
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
                "#,
            )
            .bind::<diesel::sql_types::Timestamptz, _>(new_expiry)
            .bind::<diesel::sql_types::Text, _>(&payment_reference)
            .bind::<diesel::sql_types::Uuid, _>(existing.id)
            .execute(&mut primary_conn)
            .await
            .map_err(|e| format!("Failed to extend plan: {}", e))?;
        } else {
            // Deactivate other subscription plans
            diesel::sql_query(
                r#"
                UPDATE wallet_plan_assignments
                SET is_active = false, updated_at = NOW()
                WHERE LOWER(wallet_address) = LOWER($1)
                  AND is_active = true
                  AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
                "#,
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
                "#,
            )
            .bind::<diesel::sql_types::Text, _>(&wallet_address)
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .bind::<diesel::sql_types::Timestamptz, _>(expires_at)
            .bind::<diesel::sql_types::Text, _>(&payment_reference)
            .execute(&mut primary_conn)
            .await
            .map_err(|e| format!("Failed to assign plan: {}", e))?;
        }

        // 5. Update wallet tier based on plan tier_level
        let tier_name = match tier_level {
            0..=2 => "Bronze",
            3..=5 => "Silver",
            6..=8 => "Gold",
            _ => "Platinum",
        };

        diesel::sql_query(
            r#"
            UPDATE wallet_users
            SET tier_level = $1, updated_at = NOW()
            WHERE wallet_address = $2
            "#,
        )
        .bind::<diesel::sql_types::Text, _>(tier_name)
        .bind::<diesel::sql_types::Text, _>(&wallet_address)
        .execute(&mut primary_conn)
        .await
        .ok();

        info!(
            "Payment finalized: wallet={}, plan='{}' ({}), tier={}, expires={}, ref={}",
            wallet_address, plan_name, plan_uuid, tier_name, expires_at, payment_reference
        );

        Ok(())
    }
}

/// Start the transaction monitor as a background task
pub fn spawn_transaction_monitor() {
    tokio::spawn(async {
        let config = TransactionMonitorConfig::default();

        if config.receiver_addresses.is_empty() {
            warn!(
                "No PAYMENT_RECEIVES_ADDRESS or PAYMENT_ESCROW_ADDRESS found, transaction monitor running with defaults"
            );
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
