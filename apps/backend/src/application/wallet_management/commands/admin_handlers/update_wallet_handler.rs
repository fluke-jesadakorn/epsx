// Update Wallet Command Handler
// CQRS handler for updating wallet information

use crate::application::shared::{ApplicationError, ApplicationResult, Command, CommandHandler};
use crate::application::wallet_management::commands::admin_models::{
    UpdateWalletCommand, UpdateWalletResponse,
};
use crate::application::wallet_management::queries::admin_models::{
    WalletActivitySummaryDto, WalletDetailDto, WalletGroupDto, WalletPermissionDto,
};
use async_trait::async_trait;
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
        let wallet_exists = sqlx::query!(
            "SELECT wallet_address FROM wallet_users WHERE wallet_address = $1",
            command.wallet_address
        )
        .fetch_optional(self.db_pool.as_ref())
        .await
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

        // 3. Build dynamic update query
        let mut updates = Vec::new();
        let mut update_count = 1;

        if command.is_active.is_some() {
            updates.push(format!("is_active = ${}", update_count));
            update_count += 1;
        }

        // Always update updated_at
        updates.push("updated_at = NOW()".to_string());

        if updates.is_empty() {
            return Err(ApplicationError::validation(
                "update_fields",
                "No fields to update",
            ));
        }

        // 4. Execute update
        let update_query = format!(
            "UPDATE wallet_users SET {} WHERE wallet_address = ${}",
            updates.join(", "),
            update_count
        );

        let mut query = sqlx::query(&update_query);

        // Bind parameters in order
        if let Some(is_active) = command.is_active {
            query = query.bind(is_active);
        }
        query = query.bind(&command.wallet_address);

        query
            .execute(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("❌ Failed to update wallet: {}", e);
                ApplicationError::infrastructure(format!("Failed to update wallet: {}", e))
            })?;

        info!(
            "✅ Successfully updated wallet: {}",
            command.wallet_address
        );

        // 5. Fetch updated wallet details
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
        let wallet = sqlx::query!(
            r#"
            SELECT wallet_address, is_active, created_at, last_auth_at
            FROM wallet_users
            WHERE wallet_address = $1
            "#,
            wallet_address
        )
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("❌ Failed to fetch updated wallet: {}", e);
            ApplicationError::infrastructure(format!("Failed to fetch wallet: {}", e))
        })?;

        // Get permissions
        let permissions_result = sqlx::query!(
            r#"
            -- Permissions from groups
            SELECT
                p.permission_string as permission,
                'group' as source,
                pgm.granted_at,
                wga.expires_at,
                wga.is_active
            FROM wallet_group_assignments wga
            JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE wga.wallet_address = $1
              AND p.is_active = true

            UNION ALL

            -- Direct permissions
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
            wallet_address
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .unwrap_or_default();

        let permissions: Vec<WalletPermissionDto> = permissions_result
            .into_iter()
            .map(|row| WalletPermissionDto {
                permission: row.permission.unwrap_or_else(|| "unknown".to_string()),
                source: row.source.unwrap_or_else(|| "unknown".to_string()),
                granted_at: row.granted_at.unwrap_or_else(chrono::Utc::now),
                expires_at: row.expires_at,
                is_active: row.is_active.unwrap_or(true),
            })
            .collect();

        // Groups placeholder
        let groups: Vec<WalletGroupDto> = Vec::new();

        // Activity summary
        let active_permissions_count = permissions.iter().filter(|p| p.is_active).count();
        let activity_summary = WalletActivitySummaryDto {
            total_logins: 1,
            last_30_days_logins: if wallet.last_auth_at.is_some() { 1 } else { 0 },
            total_permissions: permissions.len() as i32,
            active_permissions: active_permissions_count as i32,
            expired_permissions: (permissions.len() - active_permissions_count) as i32,
            groups_count: groups.len() as i32,
        };

        Ok(WalletDetailDto {
            wallet_address: wallet.wallet_address,
            is_active: wallet.is_active,
            created_at: wallet.created_at,
            last_auth_at: wallet.last_auth_at,
            permissions,
            groups,
            activity_summary,
        })
    }
}
