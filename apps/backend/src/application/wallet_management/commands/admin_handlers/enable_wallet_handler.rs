// Enable Wallet Command Handler
// CQRS handler for re-enabling a wallet
// MIGRATED TO SQLX (real): no stubs.

use crate::application::shared::{ApplicationError, ApplicationResult, CommandHandler};
use crate::application::wallet_management::commands::admin_models::{
    EnableWalletCommand, EnableWalletResponse,
};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};

pub struct EnableWalletCommandHandler {
    db_pool: Arc<PgPool>,
}

impl EnableWalletCommandHandler {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl CommandHandler<EnableWalletCommand> for EnableWalletCommandHandler {
    async fn handle(
        &self,
        command: EnableWalletCommand,
    ) -> ApplicationResult<EnableWalletResponse> {
        // Verify wallet exists
        let exists: Option<(String,)> =
            sqlx::query_as("SELECT wallet_address FROM wallet_users WHERE wallet_address = $1")
                .bind(&command.wallet_address)
                .fetch_optional(self.db_pool.as_ref())
                .await
                .map_err(|e| {
                    error!("Failed to query wallet {}: {}", command.wallet_address, e);
                    ApplicationError::infrastructure(format!("Failed to query wallet: {}", e))
                })?;

        if exists.is_none() {
            return Err(ApplicationError::not_found(
                "Wallet",
                &command.wallet_address,
            ));
        }

        // Mark wallet as active and append audit note
        let now = Utc::now();
        let enabled_note = json!({
            "enabled_at": now.to_rfc3339(),
            "enabled_by": "admin",
        });

        sqlx::query(
            r#"
            UPDATE wallet_users
            SET is_active = true,
                wallet_metadata = COALESCE(wallet_metadata, '{}'::jsonb) || $2::jsonb,
                updated_at = $3
            WHERE wallet_address = $1
            "#,
        )
        .bind(&command.wallet_address)
        .bind(enabled_note)
        .bind(now)
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to enable wallet {}: {}", command.wallet_address, e);
            ApplicationError::infrastructure(format!("Failed to enable wallet: {}", e))
        })?;

        info!("Enabled wallet {}", command.wallet_address);

        Ok(EnableWalletResponse {
            success: true,
            message: format!(
                "Wallet {} enabled at {}",
                command.wallet_address,
                now.to_rfc3339()
            ),
        })
    }
}
