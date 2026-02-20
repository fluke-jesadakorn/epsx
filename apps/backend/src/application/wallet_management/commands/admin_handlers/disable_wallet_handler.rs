// Disable Wallet Command Handler
// CQRS handler for disabling a wallet

use crate::application::shared::{ApplicationError, ApplicationResult, CommandHandler};
use crate::infrastructure::database::diesel_connection_manager::TlsPool;
use crate::application::wallet_management::commands::admin_models::{
    DisableWalletCommand, DisableWalletResponse,
};
use async_trait::async_trait;
use diesel_async::{RunQueryDsl};
use diesel::sql_types::Text;
use diesel::prelude::*;
use std::sync::Arc;
use tracing::{error, info};
use chrono::{Utc, Duration};
use serde_json::json;

pub struct DisableWalletCommandHandler {
    db_pool: Arc<&'static TlsPool>,
}

impl DisableWalletCommandHandler {
    pub fn new(db_pool: Arc<&'static TlsPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl CommandHandler<DisableWalletCommand> for DisableWalletCommandHandler {
    async fn handle(
        &self,
        command: DisableWalletCommand,
    ) -> ApplicationResult<DisableWalletResponse> {
        // 1. Validate command (basic validation done by types, but we can add more)
        if command.wallet_address.trim().is_empty() {
            return Err(ApplicationError::validation("wallet_address", "Wallet address cannot be empty"));
        }

        let mut conn = self.db_pool.get().await.map_err(|e| {
            error!("Failed to get connection: {}", e);
            ApplicationError::infrastructure(format!("Failed to get connection: {}", e))
        })?;

        // 2. Check if wallet exists
        // Using sql_query for flexibility as schema might not be fully generated/available in this context
        #[derive(QueryableByName)]
        struct WalletExistsRow {
            #[diesel(sql_type = Text)]
            #[allow(dead_code)]
            wallet_address: String,
        }

        let wallet_exists = diesel::sql_query(
            "SELECT wallet_address FROM wallet_users WHERE wallet_address = $1"
        )
        .bind::<Text, _>(&command.wallet_address)
        .get_result::<WalletExistsRow>(&mut conn)
        .await
        .optional()
        .map_err(|e| {
            error!("Failed to check wallet existence: {}", e);
            ApplicationError::infrastructure(format!("Failed to check wallet: {}", e))
        })?;

        if wallet_exists.is_none() {
            return Err(ApplicationError::not_found(
                "Wallet",
                &command.wallet_address,
            ));
        }

        // 3. Prepare additional metadata
        let disabled_at = Utc::now();
        let disabled_until = command.duration_days.map(|days| disabled_at + Duration::days(days as i64));

        let disable_info = json!({
            "reasonCategory": command.reason_category,
            "reasonDetails": command.reason_details,
            "affectedPlatforms": command.affected_platforms,
            "blockLogin": command.block_login,
            "pauseSubscriptions": command.pause_subscriptions,
            "notifyUser": command.notify_user,
            "disabledAt": disabled_at,
            "expiresAt": disabled_until,
            "disabledBy": &command.admin_wallet_address
        });

        // 4. Update wallet - use dedicated disable_info column
        let update_query = diesel::sql_query(
            "UPDATE wallet_users
             SET is_active = false,
                 disable_info = $2,
                 updated_at = NOW()
             WHERE wallet_address = $1"
        )
        .bind::<Text, _>(&command.wallet_address)
        .bind::<diesel::sql_types::Jsonb, _>(&disable_info);

        update_query.execute(&mut conn).await.map_err(|e| {
            error!("Failed to disable wallet: {}", e);
            ApplicationError::infrastructure(format!("Failed to disable wallet: {}", e))
        })?;
        
        info!(
            "Successfully disabled wallet: {}",
            command.wallet_address
        );

        Ok(DisableWalletResponse {
            success: true,
            message: "Wallet disabled successfully".to_string(),
            disabled_until,
        })
    }
}
