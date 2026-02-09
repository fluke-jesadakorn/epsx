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

        info!("🚀 Starting blockchain monitor (Direct Payment Model)...");

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
                error!("❌ Blockchain listener error: {}", e);
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
        info!("🛑 Blockchain monitor stopped");
    }

    /// Check if monitor is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Process a payment event - Direct Payment Model
    /// Updates wallet_users.plan_expires_at instead of creating subscription records
    async fn process_payment_event(event: PaymentEvent, pool: Arc<&'static TlsPool>) -> Result<(), AppError> {
        info!("💳 Processing payment event: {}", event.unique_id());
        info!("   User: {}", event.user_address);
        info!("   Plan: {}", event.plan_id);
        info!("   Amount: ${}", event.amount);
        info!("   TX: {}", event.transaction_hash);

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
            warn!("⚠️ Event already processed: {}", event.unique_id());
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
        .bind::<diesel::sql_types::Text, _>("PaymentReceived")
        .bind::<diesel::sql_types::BigInt, _>(event.block_number as i64)
        .bind::<diesel::sql_types::Text, _>(&event.token_address)
        .bind::<diesel::sql_types::Text, _>(&event.user_address)
        .bind::<diesel::sql_types::Integer, _>(event.plan_id as i32)
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

        // Step 3: Find or create wallet user
        let wallet_addr = WalletAddress::new(event.user_address.clone())
            .map_err(|e| AppError::validation_error("wallet_address", format!("Invalid wallet address: {}", e)))?;

        #[derive(QueryableByName)]
        struct UserRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            plan_expires_at: Option<chrono::DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            current_plan_id: Option<Uuid>,
        }

        #[derive(QueryableByName)]
        struct IdResult {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        // Map contract plan_id (tier_level) to database group ID
        let plan_uuid: Uuid = diesel::sql_query(
            "SELECT id FROM plans WHERE tier_level = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Integer, _>(event.plan_id as i32)
        .get_result::<IdResult>(&mut conn)
        .await
        .map(|r| r.id)
        .map_err(|_| AppError::entity_not_found("Subscription plan", event.plan_id.to_string()))?;

        let user_row = diesel::sql_query(
            "SELECT wallet_address, plan_expires_at, current_plan_id FROM wallet_users WHERE LOWER(wallet_address) = LOWER($1)"
        )
        .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str())
        .get_result::<UserRow>(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::database_error(format!("Failed to query user: {}", e)))?;

        let now = Utc::now();
        let standard_duration_days: i64 = 30;

        if let Some(user) = user_row {
            // Update existing user's plan access
            let current_expiry = user.plan_expires_at.unwrap_or(now);
            
            // Calculate new expiry: extend from current expiry if still active, otherwise from now
            let base_time = if current_expiry > now { current_expiry } else { now };
            let new_expiry = base_time + Duration::days(standard_duration_days);

            // Check for plan change (upgrade/switch)
            let is_plan_change = user.current_plan_id.map(|id| id != plan_uuid).unwrap_or(true);
            
            if is_plan_change {
                info!("🔄 Plan change detected: {:?} → {}", user.current_plan_id, plan_uuid);
                // For plan changes, start fresh from now (no carry-over)
                let new_expiry = now + Duration::days(standard_duration_days);
                
                diesel::sql_query(
                    r#"
                    UPDATE wallet_users
                    SET plan_expires_at = $1, current_plan_id = $2, updated_at = $3
                    WHERE LOWER(wallet_address) = LOWER($4)
                    "#
                )
                .bind::<diesel::sql_types::Timestamptz, _>(new_expiry)
                .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
                .bind::<diesel::sql_types::Timestamptz, _>(now)
                .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str())
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to update user plan: {}", e)))?;

                info!("✅ Updated user {} to plan {} (expires: {})", user.wallet_address, plan_uuid, new_expiry);
            } else {
                // Same plan - extend access
                diesel::sql_query(
                    r#"
                    UPDATE wallet_users
                    SET plan_expires_at = $1, updated_at = $2
                    WHERE LOWER(wallet_address) = LOWER($3)
                    "#
                )
                .bind::<diesel::sql_types::Timestamptz, _>(new_expiry)
                .bind::<diesel::sql_types::Timestamptz, _>(now)
                .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str())
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(format!("Failed to extend user access: {}", e)))?;

                info!("✅ Extended user {} access until {}", user.wallet_address, new_expiry);
            }
        } else {
            // Create new wallet user with plan access
            info!("Creating new wallet user: {}", wallet_addr.as_str());
            let metadata = WalletMetadata::default();
            let metadata_json = serde_json::to_value(&metadata)
                .map_err(|e| AppError::infrastructure_error(format!("Failed to serialize metadata: {}", e)))?;

            let new_expiry = now + Duration::days(standard_duration_days);

            diesel::sql_query(
                r#"
                INSERT INTO wallet_users (wallet_address, is_active, wallet_metadata, plan_expires_at, current_plan_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#
            )
            .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str().to_lowercase())
            .bind::<diesel::sql_types::Bool, _>(true)
            .bind::<diesel::sql_types::Jsonb, _>(&metadata_json)
            .bind::<diesel::sql_types::Timestamptz, _>(new_expiry)
            .bind::<diesel::sql_types::Uuid, _>(plan_uuid)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to create user: {}", e);
                AppError::database_error(format!("Failed to create user: {}", e))
            })?;

            info!("✅ Created new user {} with plan {} (expires: {})", wallet_addr.as_str(), plan_uuid, new_expiry);
        }

        // Step 4: Update event status to completed
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

        info!("✅ Payment event processed successfully");
        info!("   User: {} now has access to plan {}", event.user_address, event.plan_id);

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
    use super::*;
    use diesel_async::pooled_connection::AsyncDieselConnectionManager;

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
