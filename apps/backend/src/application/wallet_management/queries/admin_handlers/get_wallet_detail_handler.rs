// Get Wallet Detail Query Handler
// CQRS handler for retrieving detailed wallet information

use crate::application::shared::{ApplicationError, ApplicationResult, Query, QueryHandler};
use crate::application::wallet_management::queries::admin_models::{
    GetWalletDetailQuery, GetWalletDetailResponse, WalletActivitySummaryDto, WalletDetailDto,
    WalletGroupDto, WalletPermissionDto,
};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};

pub struct GetWalletDetailQueryHandler {
    db_pool: Arc<PgPool>,
}

impl GetWalletDetailQueryHandler {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl QueryHandler<GetWalletDetailQuery> for GetWalletDetailQueryHandler {
    async fn handle(
        &self,
        query: GetWalletDetailQuery,
    ) -> ApplicationResult<GetWalletDetailResponse> {
        // 1. Validate query
        query.validate()?;

        // 2. Get wallet basic info
        let wallet_result = sqlx::query!(
            r#"
            SELECT wallet_address, is_active, created_at, last_auth_at
            FROM wallet_users
            WHERE wallet_address = $1
            "#,
            query.wallet_address
        )
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("❌ Failed to fetch wallet info: {}", e);
            ApplicationError::infrastructure(format!("Failed to fetch wallet: {}", e))
        })?;

        // Check if wallet exists
        let wallet = wallet_result.ok_or_else(|| {
            ApplicationError::not_found("Wallet", &query.wallet_address)
        })?;

        // 3. Get wallet permissions (from groups + direct)
        let permissions_result = sqlx::query!(
            r#"
            -- Permissions from groups
            SELECT
                p.permission_string as permission,
                'group' as source,
                pgm.granted_at,
                wgm.expires_at,
                wgm.is_active
            FROM wallet_group_memberships wgm
            JOIN permission_group_memberships pgm ON wgm.group_id = pgm.group_id
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE wgm.wallet_address = $1
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
            query.wallet_address
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("❌ Failed to fetch permissions for {}: {}", query.wallet_address, e);
            ApplicationError::infrastructure(format!("Failed to fetch permissions: {}", e))
        })?;

        // Convert permissions to DTOs
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

        // 4. Get wallet groups (placeholder - can be implemented later)
        let groups: Vec<WalletGroupDto> = Vec::new();

        // 5. Calculate activity summary
        let active_permissions_count = permissions.iter().filter(|p| p.is_active).count();
        let activity_summary = WalletActivitySummaryDto {
            total_logins: 1, // TODO: Implement proper login tracking
            last_30_days_logins: if wallet.last_auth_at.is_some() { 1 } else { 0 },
            total_permissions: permissions.len() as i32,
            active_permissions: active_permissions_count as i32,
            expired_permissions: (permissions.len() - active_permissions_count) as i32,
            groups_count: groups.len() as i32,
        };

        // 6. Build wallet detail DTO
        let wallet_detail = WalletDetailDto {
            wallet_address: wallet.wallet_address,
            is_active: wallet.is_active,
            created_at: wallet.created_at,
            last_auth_at: wallet.last_auth_at,
            permissions,
            groups,
            activity_summary,
        };

        info!(
            "✅ Successfully retrieved details for wallet: {}",
            query.wallet_address
        );

        Ok(GetWalletDetailResponse {
            success: true,
            wallet: wallet_detail,
        })
    }
}
