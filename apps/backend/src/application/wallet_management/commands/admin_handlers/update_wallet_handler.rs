// Update Wallet Command Handler
// CQRS handler for updating wallet information

use crate::application::shared::{ApplicationError, ApplicationResult, Command, CommandHandler};
use crate::infrastructure::database::diesel_connection_manager::TlsPool;
use crate::application::wallet_management::commands::admin_models::{
    UpdateWalletCommand, UpdateWalletResponse,
};
use crate::application::wallet_management::queries::admin_models::{
    WalletActivitySummaryDto, WalletDetailDto, WalletPlanDto, WalletPermissionDto,
};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use std::sync::Arc;
use tracing::{error, info};

pub struct UpdateWalletCommandHandler {
    db_pool: Arc<&'static TlsPool>,
}

impl UpdateWalletCommandHandler {
    pub fn new(db_pool: Arc<&'static TlsPool>) -> Self {
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

        let mut conn = self.db_pool.get().await.map_err(|e| {
            error!("Failed to get connection: {}", e);
            ApplicationError::infrastructure(format!("Failed to get connection: {}", e))
        })?;

        // 2. Check if wallet exists
        #[derive(QueryableByName)]
        struct WalletExistsRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            #[allow(dead_code)]
            wallet_address: String,
        }

        let wallet_exists = diesel::sql_query(
            "SELECT wallet_address FROM wallet_users WHERE wallet_address = $1"
        )
        .bind::<diesel::sql_types::Text, _>(&command.wallet_address)
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

        // 3. Build dynamic update query
        let mut updates = Vec::new();

        if command.is_active.is_some() {
            updates.push(format!("is_active = {}", command.is_active.unwrap()));
        }

        // Handle metadata update - merge with existing wallet_metadata
        if let Some(ref new_metadata) = command.metadata {
            // Use JSONB concatenation to merge new values with existing metadata
            // This preserves existing fields while updating/adding new ones
            let metadata_json = serde_json::to_string(new_metadata).unwrap_or_else(|_| "{}".to_string());
            updates.push(format!(
                "wallet_metadata = COALESCE(wallet_metadata, '{{}}') || '{}'::jsonb",
                metadata_json.replace("'", "''")
            ));
        }

        // Always update updated_at
        updates.push("updated_at = NOW()".to_string());

        if updates.len() == 1 {
            // Only updated_at was added
            return Err(ApplicationError::validation(
                "update_fields",
                "No fields to update",
            ));
        }

        // 4. Execute update
        let update_query = format!(
            "UPDATE wallet_users SET {} WHERE wallet_address = '{}'",
            updates.join(", "),
            command.wallet_address.replace("'", "''")
        );

        diesel::sql_query(&update_query)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to update wallet: {}", e);
                ApplicationError::infrastructure(format!("Failed to update wallet: {}", e))
            })?;

        info!(
            "Successfully updated wallet: {}",
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
        let mut conn = self.db_pool.get().await.map_err(|e| {
            error!("Failed to get connection: {}", e);
            ApplicationError::infrastructure(format!("Failed to get connection: {}", e))
        })?;

        // Get basic wallet info
        #[derive(QueryableByName)]
        struct WalletRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            wallet_metadata: Option<serde_json::Value>,
        }

        let wallet = diesel::sql_query(
            r#"
            SELECT wallet_address, is_active, created_at, last_auth_at, wallet_metadata
            FROM wallet_users
            WHERE wallet_address = $1
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .get_result::<WalletRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to fetch updated wallet: {}", e);
            ApplicationError::infrastructure(format!("Failed to fetch wallet: {}", e))
        })?;

        // ... (permissions query skipped for brevity as it is unchanged)

        // Get permissions
        #[derive(QueryableByName)]
        struct PermissionRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            permission: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            source: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            granted_at: Option<chrono::DateTime<chrono::Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            expires_at: Option<chrono::DateTime<chrono::Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
            is_active: Option<bool>,
        }

        let permissions_result = diesel::sql_query(
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
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .load::<PermissionRow>(&mut conn)
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
        let plans: Vec<WalletPlanDto> = Vec::new();

        // Activity summary
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
