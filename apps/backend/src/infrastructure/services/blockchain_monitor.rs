use std::sync::Arc;
use std::str::FromStr;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use chrono::{Utc, Duration};

use crate::domain::shared_kernel::app_error::AppError;
use crate::infrastructure::blockchain::{BscEventListener, PaymentEvent};
use crate::domain::wallet_management::{
    aggregates::WalletMetadata,
    value_objects::WalletAddress,
};

/// Blockchain monitoring service that listens for payment events
/// and triggers subscription activation
pub struct BlockchainMonitor {
    bsc_listener: Arc<RwLock<BscEventListener>>,
    is_running: Arc<RwLock<bool>>,
    db_pool: Arc<&'static Pool<AsyncPgConnection>>,
}

impl BlockchainMonitor {
    /// Create new blockchain monitor
    pub fn new(
        rpc_url: String,
        contract_address: String,
        start_block: u64,
        poll_interval_secs: u64,
        db_pool: Arc<&'static Pool<AsyncPgConnection>>,
    ) -> Result<Self, AppError> {
        let bsc_listener = BscEventListener::new(
            rpc_url,
            contract_address,
            start_block,
            poll_interval_secs,
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

        info!("🚀 Starting blockchain monitor...");

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

    /// Process a payment event
    async fn process_payment_event(event: PaymentEvent, pool: Arc<&'static Pool<AsyncPgConnection>>) -> Result<(), AppError> {
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
        // Convert Decimal to BigDecimal for Diesel compatibility
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
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
        }

        let user_row = diesel::sql_query(
            "SELECT wallet_address, is_active FROM wallet_users WHERE LOWER(wallet_address) = LOWER($1)"
        )
        .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str())
        .get_result::<UserRow>(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::database_error(format!("Failed to query user: {}", e)))?;

        if user_row.is_none() {
            // Create new wallet user
            info!("Creating new wallet user: {}", wallet_addr.as_str());
            let metadata = WalletMetadata::default();
            let metadata_json = serde_json::to_value(&metadata)
                .map_err(|e| AppError::infrastructure_error(format!("Failed to serialize metadata: {}", e)))?;

            diesel::sql_query(
                r#"
                INSERT INTO wallet_users (wallet_address, is_active, wallet_metadata, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                "#
            )
            .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str().to_lowercase())
            .bind::<diesel::sql_types::Bool, _>(true)
            .bind::<diesel::sql_types::Jsonb, _>(&metadata_json)
            .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to create user: {}", e);
                AppError::database_error(format!("Failed to create user: {}", e))
            })?;
        }

        // Step 4: Get plan details to determine subscription duration
        #[derive(QueryableByName)]
        struct PlanRow {
            #[diesel(sql_type = diesel::sql_types::Integer)]
            id: i32,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            plan_type: Option<String>,
        }

        let plan_row = diesel::sql_query(
            "SELECT id, name, plan_type FROM pricing_plans WHERE id = $1"
        )
        .bind::<diesel::sql_types::Integer, _>(event.plan_id as i32)
        .get_result::<PlanRow>(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::database_error(format!("Failed to fetch plan: {}", e)))?;

        // Default to monthly billing since pricing_plans doesn't have billing_cycle
        let billing_cycle = "monthly";

        // Calculate subscription expiry based on billing cycle
        let expires_at = match billing_cycle {
            "monthly" => Some(Utc::now() + Duration::days(30)),
            "yearly" => Some(Utc::now() + Duration::days(365)),
            "lifetime" => None, // No expiry for lifetime plans
            _ => Some(Utc::now() + Duration::days(30)), // Default to monthly
        };

        // Step 5: Create subscription
        info!("Creating subscription for plan_id: {}", event.plan_id);

        // Generate subscription ID from transaction hash and plan ID
        let subscription_id_str = format!("blockchain-{}-plan{}", &event.transaction_hash[2..10], event.plan_id);

        // Prepare billing cycle JSONB
        let billing_cycle_json = serde_json::json!({
            "type": billing_cycle,
            "amount": event.amount.to_string(),
            "currency": "USDT"
        });

        // Prepare payment method JSONB
        let payment_method_json = serde_json::json!({
            "type": "blockchain",
            "token_address": event.token_address,
            "transaction_hash": event.transaction_hash,
            "block_number": event.block_number
        });

        // Prepare features and limits based on plan (basic defaults)
        let features_json = serde_json::json!({"plan_id": event.plan_id});
        let limits_json = serde_json::json!({"plan_id": event.plan_id});

        // Get plan tier name
        let tier = plan_row.as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("basic");

        // Calculate next payment date
        let next_payment = match billing_cycle {
            "monthly" => Utc::now() + Duration::days(30),
            "yearly" => Utc::now() + Duration::days(365),
            _ => Utc::now() + Duration::days(365), // Lifetime treated as far future
        };

        #[derive(QueryableByName)]
        struct SubscriptionIdRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: uuid::Uuid,
        }

        let subscription_result = diesel::sql_query(
            r#"
            INSERT INTO active_subscriptions (
                wallet_address, subscription_id, tier, billing_cycle,
                start_date, next_payment_date, auto_renewal, payment_method,
                features_included, usage_limits, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_addr.as_str().to_lowercase())
        .bind::<diesel::sql_types::Text, _>(&subscription_id_str)
        .bind::<diesel::sql_types::Text, _>(tier)
        .bind::<diesel::sql_types::Jsonb, _>(&billing_cycle_json)
        .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
        .bind::<diesel::sql_types::Timestamptz, _>(next_payment)
        .bind::<diesel::sql_types::Bool, _>(false)
        .bind::<diesel::sql_types::Jsonb, _>(&payment_method_json)
        .bind::<diesel::sql_types::Jsonb, _>(&features_json)
        .bind::<diesel::sql_types::Jsonb, _>(&limits_json)
        .bind::<diesel::sql_types::Text, _>("active")
        .get_result::<SubscriptionIdRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to create subscription: {}", e);
            AppError::database_error(format!("Failed to create subscription: {}", e))
        })?;

        let subscription_uuid = subscription_result.id;
        info!("✅ Created subscription id: {}", subscription_uuid);

        // Step 6: Update event status to completed
        // Note: subscription_id field in processed_blockchain_events is INTEGER, but we have UUID
        // Store the plan_id instead for now as a reference
        diesel::sql_query(
            r#"
            UPDATE processed_blockchain_events
            SET processing_status = $1, subscription_id = $2, processed_at = $3
            WHERE transaction_hash = $4 AND log_index = $5
            "#
        )
        .bind::<diesel::sql_types::Text, _>("completed")
        .bind::<diesel::sql_types::Integer, _>(event.plan_id as i32)
        .bind::<diesel::sql_types::Timestamp, _>(Utc::now().naive_utc())
        .bind::<diesel::sql_types::Text, _>(&event.transaction_hash)
        .bind::<diesel::sql_types::Integer, _>(event.log_index as i32)
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::database_error(format!("Failed to update event status: {}", e)))?;

        info!("✅ Payment event processed successfully: subscription_uuid={}", subscription_uuid);
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
        // This test requires a real database pool
        // Use test database URL from environment
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());

        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
        let pool = Pool::builder(config).build().expect("Failed to create test pool");
        let pool_static: &'static Pool<AsyncPgConnection> = Box::leak(Box::new(pool));
        let pool_arc = Arc::new(pool_static);

        let monitor = BlockchainMonitor::new(
            "https://data-seed-prebsc-1-s1.binance.org:8545/".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            0,
            3,
            pool_arc,
        );

        assert!(monitor.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires database connection
    async fn test_monitor_state() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());

        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
        let pool = Pool::builder(config).build().expect("Failed to create test pool");
        let pool_static: &'static Pool<AsyncPgConnection> = Box::leak(Box::new(pool));
        let pool_arc = Arc::new(pool_static);

        let monitor = BlockchainMonitor::new(
            "https://data-seed-prebsc-1-s1.binance.org:8545/".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            0,
            3,
            pool_arc,
        ).unwrap();

        assert!(!monitor.is_running().await);
    }
}
