// Disable Wallet Command Handler
// CQRS handler for disabling a wallet

use crate::application::shared::{ApplicationError, ApplicationResult, CommandHandler};
use crate::application::wallet_management::commands::admin_models::{
    DisableWalletCommand, DisableWalletResponse,
};
use async_trait::async_trait;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use diesel::sql_types::Text;
use diesel::prelude::*;
use std::sync::Arc;
use tracing::{error, info};
use chrono::{Utc, Duration};
use serde_json::json;

pub struct DisableWalletCommandHandler {
    db_pool: Arc<&'static Pool<AsyncPgConnection>>,
}

impl DisableWalletCommandHandler {
    pub fn new(db_pool: Arc<&'static Pool<AsyncPgConnection>>) -> Self {
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
            error!("❌ Failed to get connection: {}", e);
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
            error!("❌ Failed to check wallet existence: {}", e);
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
            "disabledBy": "admin" // TODO: Get actual admin ID from context if possible
        });

        // 4. Update wallet
        // We need to merge this into wallet_metadata.
        // Assuming wallet_metadata is a top-level JSONB object.
        
        let update_query = diesel::sql_query(
            "UPDATE wallet_users 
             SET is_active = false, 
                 wallet_metadata = jsonb_set(wallet_metadata, '{disable_info}', $2),
                 updated_at = NOW()
             WHERE wallet_address = $1"
        )
        .bind::<Text, _>(&command.wallet_address)
        .bind::<diesel::sql_types::Jsonb, _>(&disable_info);

        update_query.execute(&mut conn).await.map_err(|e| {
            error!("❌ Failed to disable wallet: {}", e);
            ApplicationError::infrastructure(format!("Failed to disable wallet: {}", e))
        })?;
        
        // 5. If block_login is true, revoke active sessions
        if command.block_login {
            diesel::sql_query(
                "UPDATE sessions SET is_revoked = true WHERE wallet_address = $1 AND is_revoked = false"
            )
            .bind::<Text, _>(&command.wallet_address)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("❌ Failed to revoke sessions: {}", e);
                // Non-critical error, log but continue
                ApplicationError::infrastructure(format!("Failed to revoke sessions: {}", e))
            })?;
        }

        info!(
            "✅ Successfully disabled wallet: {}",
            command.wallet_address
        );

        Ok(DisableWalletResponse {
            success: true,
            message: "Wallet disabled successfully".to_string(),
            disabled_until,
        })
    }
}
