// Disable Wallet Command Handler
// CQRS handler for disabling a wallet
// MIGRATED TO SQLX (real): no stubs.

use crate::application::shared::{ApplicationError, ApplicationResult, CommandHandler};
use crate::application::wallet_management::commands::admin_models::{
    DisableWalletCommand, DisableWalletResponse,
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};

pub struct DisableWalletCommandHandler {
    db_pool: Arc<PgPool>,
}

impl DisableWalletCommandHandler {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl CommandHandler<DisableWalletCommand> for DisableWalletCommandHandler {
    async fn handle(
        &self,
        command: DisableWalletCommand,
    ) -> ApplicationResult<DisableWalletResponse> {
        // Verify wallet exists
        let exists: Option<(String,)> = sqlx::query_as("SELECT wallet_address FROM wallet_users WHERE wallet_address = $1")
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

        // Mark wallet as inactive and append audit note to wallet_metadata JSON
        let now = Utc::now();
        let disabled_note = json!({
            "disabled_at": now.to_rfc3339(),
            "disabled_by": "admin",
            "reason": "admin_disable",
        });

        sqlx::query(
            r#"
            UPDATE wallet_users
            SET is_active = false,
                wallet_metadata = COALESCE(wallet_metadata, '{}'::jsonb) || $2::jsonb,
                updated_at = $3
            WHERE wallet_address = $1
            "#,
        )
        .bind(&command.wallet_address)
        .bind(disabled_note)
        .bind(now)
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to disable wallet {}: {}", command.wallet_address, e);
            ApplicationError::infrastructure(format!("Failed to disable wallet: {}", e))
        })?;

        info!("Disabled wallet {}", command.wallet_address);

        Ok(DisableWalletResponse {
            success: true,
            message: format!("Wallet {} disabled at {}", command.wallet_address, now.to_rfc3339()),
            disabled_until: Some(now + Duration::days(30)),
        })
    }
}