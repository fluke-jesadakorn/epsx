use crate::prelude::TlsPool;
use std::sync::Arc;
use std::str::FromStr;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use chrono::{Utc, Duration};

use uuid::Uuid;
use crate::domain::shared_kernel::app_error::AppError;
use crate::infrastructure::blockchain::{BscEventListener, PaymentEvent};
use crate::domain::wallet_management::{
    aggregates::WalletMetadata,
    value_objects::WalletAddress,
};

/// Blockchain monitoring service that listens for payment events
/// and triggers plan access extension (Direct Payment Model)
pub struct BlockchainMonitor {
    bsc_listener: Arc<RwLock<BscEventListener>>,
    is_running: Arc<RwLock<bool>>,
    db_pool: Arc<&'static TlsPool>,
}

impl BlockchainMonitor {
    /// Create new blockchain monitor
    pub fn new(
        rpc_url: String,
        contract_address: String,
        start_block: u64,
        poll_interval_secs: u64,
        supported_tokens: Vec<String>,
        db_pool: Arc<&'static TlsPool>,
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
            return Err(AppError::infrastructure_error("Monitor already running"));
        }
        *is_running = true;
        drop(is_running);

        info!("Starting blockchain monitor (Direct Payment Model)...");

        let listener = self.bsc_listener.clone();
        let is_running_flag = self.is_running.clone();
        let db_pool = self.db_pool.clone();

        tokio::spawn(async move {
            let mut listener = listener.write().await;

            let result = listener.start_listening(|event| {
                let pool = db_pool.clone();
                Box::pin(async move {
                    Self::process_payment_event(event, pool).await
                })
            }).await;

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
    async fn process_payment_event(event: PaymentEvent, pool: Arc<&'static TlsPool>) -> Result<(), AppError> {
        info!("Processing payment event: {}", event.unique_id());
        info!("   User: {}", event.user_address);
        info!("   Context: type={}, id={}", event.context_type, event.context_id);
        info!("   Amount: ${}", event.amount);
        info!("   TX: {}", event.transaction_hash);

        // Only process PLAN payments (context_type == 0) for plan activation
        if event.context_type != 0 {
            info!("Skipping non-plan payment (context_type={})", event.context_type);
            return Ok(());
        }

        // Step 1: Check if event already processed (prevent duplicates)
        let mut conn = pool.get().await
            .map_err(|e| AppError::database_error(format!("Failed to get connection: {}", e)))?;

        #[derive(QueryableByName)]
        struct EventIdRow {
            #[allow(dead_code)]
            #[diesel(sql_type = diesel::sql_types::Integer)]
            id: i32,
        }

        let existing_event = diesel::sql_query(
            "SELECT id FROM processed_blockchain_events WHERE transaction_hash = $1 AND log_index = $2"
        )
        .bind::<diesel::sql_types::Text, _>(&event.transaction_hash)
        .bind::<diesel::sql_types::Integer, _>(event.log_index as i32)
        .get_result::<EventIdRow>(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::database_error(format!("Failed to check event: {}", e)))?;

        if existing_event.is_some() {
            warn!("Event already processed: {}", event.unique_id());
            return Ok(());
        }

        // Step 2: Insert event as processing
        let amount_bd = bigdecimal::BigDecimal::from_str(&event.amount.to_string())
            .map_err(|e| AppError::infrastructure_error(format!("Failed to convert amount: {}", e)))?;

        diesel::sql_query(
            r#"
            INSERT INTO processed_blockchain_events (
                transaction_hash, log_index, event_type, block_number,
                contract_address, user_address, plan_id, token_address,
                amount, payment_id, event_timestamp, processing_status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&event.transaction_hash)
        .bind::<diesel::sql_types::Integer, _>(event.log_index as i32)
        .bind::<diesel::sql_types::Text, _>("PaymentWithContext")
        .bind::<diesel::sql_types::BigInt, _>(event.block_number as i64)
        .bind::<diesel::sql_types::Text, _>(&event.token_address)
        .bind::<diesel::sql_types::Text, _>(&event.user_address)
        .bind::<diesel::sql_types::Integer, _>(event.context_id as i32)
        .bind::<diesel::sql_types::Text, _>(&event.token_address)
        .bind::<diesel::sql_types::Numeric, _>(&amount_bd)
        .bind::<diesel::sql_types::BigInt, _>(event.payment_id as i64)
        .bind::<diesel::sql_types::Timestamp, _>(event.timestamp.naive_utc())
        .bind::<diesel::sql_types::Text, _>("processing")
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to insert event: {}", e);
            AppError::database_error(format!("Failed to insert event: {}", e))
        })?;

        // Step 3: Resolve wallet address and plan UUID
        let wallet_addr = WalletAddress::new(event.user_address.clone())
            .map_err(|e| AppError::validation_error("wallet_address", format!("Invalid wallet address: {}", e)))?;

        #[derive(QueryableByName)]
        struct IdResult {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        // Map contract context_id (tier_level) to database plan UUID
        // For PLAN payments (context_type=0), context_id maps to tier_level
        let plan_uuid: Uuid = diesel::sql_query(
            "SELECT id FROM plans WHERE tier_level = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Integer, _>(event.context_id as i32)
        .get_result::<IdResult>(&mut conn)
        .await
        .map(|r| r.id)
        .map_err(|_| AppError::entity_not_found("Subscription plan", event.context_id.to_string()))?;

        let now = Utc::now();
        let standard_duration_days: i64 = 30;

        // Step 4: Ensure wallet_users entry exists (required for FK constraint)
        let metadata = WalletMetadata::default();
        let metadata_json = serde_json::to_value(&metadata)
            .map_err(|e| AppError::infrastructure_error(format!("Failed to serialize metadata: {}", e)))?;

        diesel::sql_query(
            r#"
            INSERT INTO wallet_users (wallet_address, is_active, wallet_metadata, created_at, updated_at)
            VALUES ($1, true, $2, $3, $4)
            ON CONFLICT (wallet_address) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str().to_lowercase())
        .bind::<diesel::sql_types::Jsonb, _>(&metadata_json)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to ensure wallet user exists: {}", e);
            AppError::database_error(format!("Failed to ensure wallet user: {}", e))
        })?;

        // Step 5: Check for existing assignment (active OR inactive) for this plan
        #[derive(QueryableByName)]
        struct ExistingAssignment {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            expires_at: chrono::DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
        }

        let existing_assignment: Option<ExistingAssignment> = diesel::sql_query(
            "SELECT id, expires_at, is_active FROM wallet_plan_assignments WHERE LOWER(wallet_address) = LOWER($1) AND plan_id = $2 ORDER BY is_active DESC, expires_at DESC LIMIT 1"
        )
        .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str())
        .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
        .get_result(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::database_error(format!("Failed to check existing assignment: {}", e)))?;

        let payment_reference = format!("BC-{}", &event.transaction_hash[..10.min(event.transaction_hash.len())]);

        if let Some(existing) = existing_assignment {
            // REACTIVATE/EXTEND existing assignment
            let base_time = if existing.is_active && existing.expires_at > now { existing.expires_at } else { now };
            let new_expiry = base_time + Duration::days(standard_duration_days);

            info!("{} plan {} for wallet {}. Old expiry: {}, New expiry: {}",
                if existing.is_active { "Extending" } else { "Reactivating" },
                plan_uuid, wallet_addr.as_str(), existing.expires_at, new_expiry);

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
            .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str())
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .execute(&mut conn)
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
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(format!("Failed to extend plan: {}", e)))?;

            info!("{} user {} plan access until {}",
                if existing.is_active { "Extended" } else { "Reactivated" },
                wallet_addr.as_str(), new_expiry);
        } else {
            // NEW assignment: no prior record for this wallet+plan
            let new_expiry = now + Duration::days(standard_duration_days);

            diesel::sql_query(
                r#"
                UPDATE wallet_plan_assignments
                SET is_active = false, updated_at = NOW()
                WHERE LOWER(wallet_address) = LOWER($1)
                  AND is_active = true
                  AND plan_id IN (SELECT id FROM plans WHERE plan_type = 'subscription')
                "#
            )
            .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str())
            .execute(&mut conn)
            .await
            .ok();

            diesel::sql_query(
                r#"
                INSERT INTO wallet_plan_assignments (
                    wallet_address, plan_id, assigned_at, expires_at, is_active,
                    assignment_source, assignment_reason, payment_reference,
                    auto_renew, assignment_metadata
                )
                VALUES ($1, $2, NOW(), $3, true, 'blockchain', 'Plan purchase via blockchain event', $4, false, '{}')
                "#
            )
            .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str().to_lowercase())
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .bind::<diesel::sql_types::Timestamptz, _>(new_expiry)
            .bind::<diesel::sql_types::Text, _>(&payment_reference)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to create plan assignment: {}", e);
                AppError::database_error(format!("Failed to create plan assignment: {}", e))
            })?;

            info!("Created new plan assignment for user {} → plan {} (expires: {})", wallet_addr.as_str(), plan_uuid, new_expiry);
        }

        // Fix 2: Sync payments.status so frontend polling resolves correctly
        diesel::sql_query(
            "UPDATE payments SET status = 'confirmed', completed_at = NOW() WHERE transaction_hash = $1 AND status != 'confirmed'"
        )
        .bind::<diesel::sql_types::Text, _>(&event.transaction_hash)
        .execute(&mut conn)
        .await
        .ok();

        // Step 6: Update event status to completed
        diesel::sql_query(
            r#"
            UPDATE processed_blockchain_events
            SET processing_status = $1, processed_at = $2
            WHERE transaction_hash = $3 AND log_index = $4
            "#
        )
        .bind::<diesel::sql_types::Text, _>("completed")
        .bind::<diesel::sql_types::Timestamp, _>(Utc::now().naive_utc())
        .bind::<diesel::sql_types::Text, _>(&event.transaction_hash)
        .bind::<diesel::sql_types::Integer, _>(event.log_index as i32)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to update event status: {}", e)))?;

        info!("Payment event processed successfully");
        info!("   User: {} now has access to plan {}", event.user_address, event.context_id);

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
