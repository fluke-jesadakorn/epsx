// Enable Wallet Command Handler
// CQRS handler for re-enabling a wallet

use crate::application::shared::{ApplicationError, ApplicationResult, CommandHandler};
use crate::infrastructure::database::diesel_connection_manager::TlsPool;
use crate::application::wallet_management::commands::admin_models::{
    EnableWalletCommand, EnableWalletResponse,
};
use async_trait::async_trait;
use diesel_async::{RunQueryDsl};
use diesel::sql_types::Text;
use diesel::prelude::*;
use std::sync::Arc;
use tracing::{error, info};
use chrono::Utc;
use serde_json::json;

pub struct EnableWalletCommandHandler {
    db_pool: Arc<&'static TlsPool>,
}

impl EnableWalletCommandHandler {
    pub fn new(db_pool: Arc<&'static TlsPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl CommandHandler<EnableWalletCommand> for EnableWalletCommandHandler {
    async fn handle(
        &self,
        command: EnableWalletCommand,
    ) -> ApplicationResult<EnableWalletResponse> {
        // 1. Validate command
        if command.wallet_address.trim().is_empty() {
            return Err(ApplicationError::validation("wallet_address", "Wallet address cannot be empty"));
        }

        let mut conn = self.db_pool.get().await.map_err(|e| {
            error!("❌ Failed to get connection: {}", e);
            ApplicationError::infrastructure(format!("Failed to get connection: {}", e))
        })?;

        // 2. Check if wallet exists
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

        // 3. Prepare re-enable metadata
        // We'll move the current disable_info to a history array if we want to keep track,
        // or just remove it. For now, let's keep it simple and just remove or nullify it,
        // but maybe adding a "reenabled_info" is better for audit.
        
        let reenable_info = json!({
            "enabledAt": Utc::now(),
            "resolutionNote": command.resolution_note,
            "restoredPermissions": command.restore_permissions,
            "resumedSubscriptions": command.resume_subscriptions,
            "enabledBy": &command.admin_wallet_address
        });

        // 4. Update wallet
        // Remove 'disable_info' and set is_active to true.
        // We can optionally store the last disable info in a history field, but simpler to just remove for now
        // and add re-enable log.
        // `#-` operator removes key from JSONB.
        
        let update_query = diesel::sql_query(
            "UPDATE wallet_users 
             SET is_active = true, 
                 wallet_metadata = (wallet_metadata #- '{disable_info}') || jsonb_build_object('last_reenable_info', $2),
                 updated_at = NOW()
             WHERE wallet_address = $1"
        )
        .bind::<Text, _>(&command.wallet_address)
        .bind::<diesel::sql_types::Jsonb, _>(&reenable_info);

        update_query.execute(&mut conn).await.map_err(|e| {
            error!("❌ Failed to enable wallet: {}", e);
            ApplicationError::infrastructure(format!("Failed to enable wallet: {}", e))
        })?;

        info!(
            "✅ Successfully enabled wallet: {}",
            command.wallet_address
        );

        Ok(EnableWalletResponse {
            success: true,
            message: "Wallet re-enabled successfully".to_string(),
        })
    }
}
