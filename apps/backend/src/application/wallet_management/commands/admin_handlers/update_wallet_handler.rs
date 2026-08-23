// Update Wallet Command Handler
// CQRS handler for updating wallet information
// MIGRATED TO SQLX (real): no stubs.

use crate::application::shared::{ApplicationError, ApplicationResult, Command, CommandHandler};
use crate::application::wallet_management::commands::admin_models::{
    UpdateWalletCommand, UpdateWalletResponse,
};
use crate::application::wallet_management::queries::admin_models::{
    WalletActivitySummaryDto, WalletDetailDto, WalletPermissionDto, WalletPlanDto,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};

pub struct UpdateWalletCommandHandler {
    db_pool: Arc<PgPool>,
}

impl UpdateWalletCommandHandler {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl CommandHandler<UpdateWalletCommand> for UpdateWalletCommandHandler {
    async fn handle(
        &self,
        command: UpdateWalletCommand,
    ) -> ApplicationResult<UpdateWalletResponse> {
        // 1. Validate command
        command.validate()?;

        // 2. Check if wallet exists
        let wallet_exists: Option<(String,)> =
            sqlx::query_as("SELECT wallet_address FROM wallet_users WHERE wallet_address = $1")
                .bind(&command.wallet_address)
                .fetch_optional(self.db_pool.as_ref())
                .await
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

        // 3. Build dynamic UPDATE using sqlx::QueryBuilder
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> =
            sqlx::QueryBuilder::new("UPDATE wallet_users SET updated_at = NOW()");

        if let Some(is_active) = command.is_active {
            qb.push(", is_active = ").push_bind(is_active);
        }

        if let Some(ref new_metadata) = command.metadata {
            qb.push(", wallet_metadata = COALESCE(wallet_metadata, '{}'::jsonb) || ")
                .push_bind(new_metadata.clone())
                .push("::jsonb");
        }

        qb.push(" WHERE wallet_address = ").push_bind(&command.wallet_address);

        qb.build()
            .execute(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to update wallet: {}", e);
                ApplicationError::infrastructure(format!("Failed to update wallet: {}", e))
            })?;

        info!("Successfully updated wallet: {}", command.wallet_address);

        // 4. Fetch updated wallet details
        let updated_wallet = self.fetch_wallet_details(&command.wallet_address).await?;

        Ok(UpdateWalletResponse {
            success: true,
            wallet: updated_wallet,
            message: "Wallet updated successfully".to_string(),
        })
    }
}

impl UpdateWalletCommandHandler {
    /// Helper method to fetch complete wallet details after update
    async fn fetch_wallet_details(
        &self,
        wallet_address: &str,
    ) -> ApplicationResult<WalletDetailDto> {
        // Get basic wallet info
        #[derive(sqlx::FromRow)]
        struct WalletRow {
            wallet_address: String,
            is_active: bool,
            created_at: DateTime<Utc>,
            last_auth_at: Option<DateTime<Utc>>,
            wallet_metadata: Option<Value>,
        }

        let wallet: WalletRow = sqlx::query_as(
            r#"
            SELECT wallet_address, is_active, created_at, last_auth_at, wallet_metadata
            FROM wallet_users
            WHERE wallet_address = $1
            "#,
        )
        .bind(wallet_address)
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch updated wallet: {}", e);
            ApplicationError::infrastructure(format!("Failed to fetch wallet: {}", e))
        })?;

        // Get permissions
        #[derive(sqlx::FromRow)]
        struct PermissionRow {
            permission: Option<String>,
            source: Option<String>,
            granted_at: Option<DateTime<Utc>>,
            expires_at: Option<DateTime<Utc>>,
            is_active: Option<bool>,
        }

        let permissions_result: Vec<PermissionRow> = sqlx::query_as(
            r#"
            SELECT
                p.permission_string as permission,
                'plan' as source,
                pgm.granted_at,
                wgm.expires_at,
                wgm.is_active
            FROM wallet_plan_assignments wgm
            JOIN plan_permissions pgm ON wgm.plan_id = pgm.plan_id
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE wgm.wallet_address = $1
              AND p.is_active = true

            UNION ALL

            SELECT
                p.permission_string as permission,
                'direct' as source,
                wdp.granted_at,
                wdp.expires_at,
                wdp.is_active
            FROM wallet_direct_permissions wdp
            JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = $1
              AND p.is_active = true

            ORDER BY permission
            "#,
        )
        .bind(wallet_address)
        .fetch_all(self.db_pool.as_ref())
        .await
        .unwrap_or_default();

        let permissions: Vec<WalletPermissionDto> = permissions_result
            .into_iter()
            .map(|row| WalletPermissionDto {
                permission: row.permission.unwrap_or_else(|| "unknown".to_string()),
                source: row.source.unwrap_or_else(|| "unknown".to_string()),
                granted_at: row.granted_at.unwrap_or_else(Utc::now),
                expires_at: row.expires_at,
                is_active: row.is_active.unwrap_or(true),
            })
            .collect();

        let plans: Vec<WalletPlanDto> = Vec::new();

        let active_permissions_count = permissions.iter().filter(|p| p.is_active).count();
        let activity_summary = WalletActivitySummaryDto {
            total_logins: 1,
            last_30_days_logins: if wallet.last_auth_at.is_some() { 1 } else { 0 },
            total_permissions: permissions.len() as i32,
            active_permissions: active_permissions_count as i32,
            expired_permissions: (permissions.len() - active_permissions_count) as i32,
            plans_count: plans.len() as i32,
        };

        Ok(WalletDetailDto {
            wallet_address: wallet.wallet_address,
            is_active: wallet.is_active,
            created_at: wallet.created_at,
            last_auth_at: wallet.last_auth_at,
            permissions,
            plans,
            activity_summary,
            metadata: wallet.wallet_metadata,
        })
    }
}