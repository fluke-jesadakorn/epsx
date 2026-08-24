use crate::prelude::TlsPool;
use chrono::{Duration, Utc};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::domain::wallet_management::{aggregates::WalletMetadata, value_objects::WalletAddress};
use crate::infrastructure::blockchain::{BscEventListener, PaymentEvent};
use epsx_contracts::errors::AppError;
use uuid::Uuid;

/// Blockchain monitoring service that listens for payment events
/// and triggers plan access extension (Direct Payment Model)
pub struct BlockchainMonitor {
    bsc_listener: Arc<RwLock<BscEventListener>>,
    is_running: Arc<RwLock<bool>>,
    db_pool: Arc<TlsPool>,
}

impl BlockchainMonitor {
    /// Create new blockchain monitor
    pub fn new(
        rpc_url: String,
        contract_address: String,
        start_block: u64,
        poll_interval_secs: u64,
        supported_tokens: Vec<String>,
        db_pool: Arc<TlsPool>,
    ) -> Result<Self, AppError> {
        let bsc_listener = BscEventListener::new(
            rpc_url,
            contract_address,
            start_block,
            poll_interval_secs,
            supported_tokens,
        )?;

        Ok(Self {
            bsc_listener: Arc::new(RwLock::new(bsc_listener)),
            is_running: Arc::new(RwLock::new(false)),
            db_pool,
        })
    }

    /// Start monitoring blockchain events
    pub async fn start_monitoring(&self) -> Result<(), AppError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(AppError::internal_server_error("Monitor already running"));
        }
        *is_running = true;
        drop(is_running);

        info!("Starting blockchain monitor (Direct Payment Model)...");

        let listener = self.bsc_listener.clone();
        let is_running_flag = self.is_running.clone();
        let db_pool = self.db_pool.clone();

        tokio::spawn(async move {
            let mut listener = listener.write().await;

            let result = listener
                .start_listening(|event| {
                    let pool = db_pool.clone();
                    Box::pin(async move { Self::process_payment_event(event, pool).await })
                })
                .await;

            if let Err(e) = result {
                error!("Blockchain listener error: {}", e);
            }

            let mut is_running = is_running_flag.write().await;
            *is_running = false;
        });

        Ok(())
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Blockchain monitor stopped");
    }

    /// Check if monitor is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Process a payment event - Direct Payment Model (V2: PaymentWithContext)
    /// Creates/extends wallet_plan_assignments for proper plan activation
    async fn process_payment_event(
        event: PaymentEvent,
        pool: Arc<TlsPool>,
    ) -> Result<(), AppError> {
        info!("Processing payment event: {}", event.unique_id());
        info!("   User: {}", event.user_address);
        info!(
            "   Context: type={}, id={}",
            event.context_type, event.context_id
        );
        info!("   Amount: ${}", event.amount);
        info!("   TX: {}", event.transaction_hash);

        // Only process PLAN payments (context_type == 0) for plan activation
        if event.context_type != 0 {
            info!(
                "Skipping non-plan payment (context_type={})",
                event.context_type
            );
            return Ok(());
        }

        // Step 1: Check if event already processed (prevent duplicates)
        let existing_event: (i32,) = sqlx::query_as(
            "SELECT id FROM processed_blockchain_events WHERE transaction_hash = $1 AND log_index = $2",
        )
        .bind(&event.transaction_hash)
        .bind(event.log_index as i32)
        .fetch_optional(pool.as_ref())
        .await
        .ok()
        .flatten()
        .unwrap_or((0,));

        if existing_event.0 > 0 {
            warn!("Event already processed: {}", event.unique_id());
            return Ok(());
        }

        // Step 2: Insert event as processing
        let amount_bd =
            bigdecimal::BigDecimal::from_str(&event.amount.to_string()).map_err(|e| {
                AppError::internal_server_error(format!("Failed to convert amount: {}", e))
            })?;

        sqlx::query(
            r#"
            INSERT INTO processed_blockchain_events (
                transaction_hash, log_index, event_type, block_number,
                contract_address, user_address, plan_id, token_address,
                amount, payment_id, event_timestamp, processing_status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&event.transaction_hash)
        .bind(event.log_index as i32)
        .bind("PaymentWithContext")
        .bind(event.block_number as i64)
        .bind(&event.token_address)
        .bind(&event.user_address)
        .bind(event.context_id as i32)
        .bind(&event.token_address)
        .bind(&amount_bd)
        .bind(event.payment_id as i64)
        .bind(event.timestamp.naive_utc())
        .bind("processing")
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to insert event: {}", e);
            AppError::database_error(format!("Failed to insert event: {}", e))
        })?;

        // Step 3: Resolve wallet address and plan UUID
        let wallet_addr = WalletAddress::new(event.user_address.clone())
            .map_err(|e| AppError::validation_error(format!("Invalid wallet_address: {}", e)))?;

        // Map contract context_id (tier_level) to database plan UUID
        let plan_uuid: Uuid = sqlx::query_scalar(
        "SELECT id FROM plans WHERE tier_level = $1 LIMIT 1",
    )
    .bind(event.context_id as i32)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|_| AppError::entity_not_found(format!("Subscription plan {}", event.context_id)))?
    .ok_or_else(|| AppError::entity_not_found(format!("Subscription plan {}", event.context_id)))?;

        let now = Utc::now();
        let standard_duration_days: i64 = 30;

        // Step 4: Ensure wallet_users entry exists (required for FK constraint)
        let metadata = WalletMetadata::default();
        let metadata_json = serde_json::to_value(&metadata).map_err(|e| {
            AppError::internal_server_error(format!("Failed to serialize metadata: {}", e))
        })?;

        sqlx::query(
            r#"
            INSERT INTO wallet_users (wallet_address, is_active, wallet_metadata, created_at, updated_at)
            VALUES ($1, true, $2, $3, $4)
            ON CONFLICT (wallet_address) DO NOTHING
            "#,
        )
        .bind(wallet_addr.as_str().to_lowercase())
        .bind(&metadata_json)
        .bind(now)
        .bind(now)
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to ensure wallet user exists: {}", e);
            AppError::database_error(format!("Failed to ensure wallet user: {}", e))
        })?;

        // Step 5: Check for existing assignment (active OR inactive) for this plan
        #[derive(sqlx::FromRow)]
        struct ExistingAssignment {
            id: Uuid,
            expires_at: chrono::DateTime<Utc>,
            is_active: bool,
        }

        let existing_assignment: Option<ExistingAssignment> = sqlx::query_as(
            "SELECT id, expires_at, is_active FROM wallet_plan_assignments WHERE LOWER(wallet_address) = LOWER($1) AND plan_id = $2 ORDER BY is_active DESC, expires_at DESC LIMIT 1"
        )
        .bind(wallet_addr.as_str())
        .bind(plan_uuid)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("Failed to check existing assignment: {}", e)))?;

        let payment_reference = format!(
            "BC-{}",
            &event.transaction_hash[..10.min(event.transaction_hash.len())]
        );

        if let Some(existing) = existing_assignment {
            // REACTIVATE/EXTEND existing assignment
            let base_time = if existing.is_active && existing.expires_at > now {
                existing.expires_at
            } else {
                now
            };
            let new_expiry = base_time + Duration::days(standard_duration_days);

            info!(
                "{} plan {} for wallet {}. Old expiry: {}, New expiry: {}",
                if existing.is_active {
                    "Extending"
                } else {
                    "Reactivating"
                },
                plan_uuid,
                wallet_addr.as_str(),
                existing.expires_at,
                new_expiry
            );

            // Deactivate other subscription plans first
            sqlx::query(
                r#"
                UPDATE wallet_plan_assignments
                SET is_active = false, updated_at = NOW()
                WHERE LOWER(wallet_address) = LOWER($1)
                  AND is_active = true
                  AND plan_id != $2
                  AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
                "#,
            )
            .bind(wallet_addr.as_str())
            .bind(plan_uuid)
            .execute(pool.as_ref())
            .await
            .ok();

            sqlx::query(
                r#"
                UPDATE wallet_plan_assignments
                SET expires_at = $1, payment_reference = $2, updated_at = NOW(), is_active = true
                WHERE id = $3
                "#,
            )
            .bind(new_expiry)
            .bind(&payment_reference)
            .bind(existing.id)
            .execute(pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("Failed to extend plan: {}", e)))?;

            info!(
                "{} user {} plan access until {}",
                if existing.is_active {
                    "Extended"
                } else {
                    "Reactivated"
                },
                wallet_addr.as_str(),
                new_expiry
            );
        } else {
            // NEW assignment: no prior record for this wallet+plan
            let new_expiry = now + Duration::days(standard_duration_days);

            sqlx::query(
                r#"
                UPDATE wallet_plan_assignments
                SET is_active = false, updated_at = NOW()
                WHERE LOWER(wallet_address) = LOWER($1)
                  AND is_active = true
                  AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
                "#,
            )
            .bind(wallet_addr.as_str())
            .execute(pool.as_ref())
            .await
            .ok();

            sqlx::query(
                r#"
                INSERT INTO wallet_plan_assignments (
                    wallet_address, plan_id, assigned_at, expires_at, is_active,
                    assignment_source, assignment_reason, payment_reference,
                    auto_renew, assignment_metadata
                )
                VALUES ($1, $2, NOW(), $3, true, 'blockchain', 'Plan purchase via blockchain event', $4, false, '{}')
                "#,
            )
            .bind(wallet_addr.as_str().to_lowercase())
            .bind(plan_uuid)
            .bind(new_expiry)
            .bind(&payment_reference)
            .execute(pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to create plan assignment: {}", e);
                AppError::database_error(format!("Failed to create plan assignment: {}", e))
            })?;

            info!(
                "Created new plan assignment for user {} → plan {} (expires: {})",
                wallet_addr.as_str(),
                plan_uuid,
                new_expiry
            );
        }

        // Fix 2: Sync payments.status so frontend polling resolves correctly
        sqlx::query(
            "UPDATE payments SET status = 'confirmed', completed_at = NOW() WHERE transaction_hash = $1 AND status != 'confirmed'",
        )
        .bind(&event.transaction_hash)
        .execute(pool.as_ref())
        .await
        .ok();

        // Step 6: Update event status to completed
        sqlx::query(
            r#"
            UPDATE processed_blockchain_events
            SET processing_status = $1, processed_at = $2
            WHERE transaction_hash = $3 AND log_index = $4
            "#,
        )
        .bind("completed")
        .bind(Utc::now().naive_utc())
        .bind(&event.transaction_hash)
        .bind(event.log_index as i32)
        .execute(pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("Failed to update event status: {}", e)))?;

        info!("Payment event processed successfully");
        info!(
            "   User: {} now has access to plan {}",
            event.user_address, event.context_id
        );

        Ok(())
    }

    /// Get current blockchain height
    pub async fn get_current_block(&self) -> Result<u64, AppError> {
        let listener = self.bsc_listener.read().await;
        listener.get_current_block().await
    }

    /// Update starting block for listener
    pub async fn set_start_block(&self, block: u64) {
        let mut listener = self.bsc_listener.write().await;
        listener.set_last_checked_block(block);
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_blockchain_monitor_creation() {
        // Test requires proper TLS pool initialization and is ignored
        // Run manually with proper database setup
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_monitor_state() {
        // Test requires proper TLS pool initialization and is ignored
        // Run manually with proper database setup
    }
}
