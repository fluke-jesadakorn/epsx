// Get Wallet Detail Query Handler
// CQRS handler for retrieving detailed wallet information

use crate::application::shared::{ApplicationError, ApplicationResult, Query, QueryHandler};
use crate::application::wallet_management::queries::admin_models::{
    GetWalletDetailQuery, GetWalletDetailResponse, WalletActivitySummaryDto, WalletDetailDto,
    WalletPlanDto, WalletPermissionDto,
};
use crate::application::wallet_management::wallet_management_repository::WalletManagementRepository;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use std::sync::Arc;
use tracing::{error, info};

#[derive(diesel::QueryableByName)]
struct PermissionDetailRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub permission: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub source: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub granted_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_active: bool,
}

pub struct GetWalletDetailQueryHandler {
    db_pool: Arc<&'static Pool<AsyncPgConnection>>,
}

impl GetWalletDetailQueryHandler {
    pub fn new(db_pool: Arc<&'static Pool<AsyncPgConnection>>) -> Self {
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

        // 2. Initialize repository
        let repo = WalletManagementRepository::new(self.db_pool.clone());

        // 3. Get wallet basic info using repository
        let wallet = repo
            .get_wallet_basic_info(&query.wallet_address)
            .await
            .map_err(|e| {
                error!("❌ Failed to fetch wallet info: {}", e);
                ApplicationError::infrastructure(e.to_string())
            })?
            .ok_or_else(|| ApplicationError::not_found("Wallet", &query.wallet_address))?;

        let mut conn = self.db_pool.get().await
            .map_err(|e| ApplicationError::infrastructure(format!("Failed to get database connection: {}", e)))?;

        // 4. Get permissions (union of group and direct permissions)
        let permissions_result = diesel::sql_query(r#"
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
        "#)
        .bind::<diesel::sql_types::Text, _>(&query.wallet_address)
        .load::<PermissionDetailRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("❌ Failed to fetch permissions for {}: {}", query.wallet_address, e);
            ApplicationError::infrastructure(format!("Failed to fetch permissions: {}", e))
        })?;

        // Convert permissions to DTOs
        let permissions: Vec<WalletPermissionDto> = permissions_result
            .into_iter()
            .map(|p| WalletPermissionDto {
                permission: p.permission,
                source: p.source,
                granted_at: p.granted_at,
                expires_at: p.expires_at,
                is_active: p.is_active,
            })
            .collect();

        // 5. Get wallet plans (placeholder - can be implemented later)
        let plans: Vec<WalletPlanDto> = Vec::new();

        // 6. Calculate activity summary with actual login tracking
        let active_permissions_count = permissions.iter().filter(|p| p.is_active).count();

        let mut conn = self.db_pool.get().await.map_err(|e| {
            error!("❌ Failed to get connection: {}", e);
            ApplicationError::infrastructure(format!("Failed to get connection: {}", e))
        })?;

        // Get total login count from sessions table
        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let total_logins: i32 = diesel::sql_query(
            "SELECT COUNT(*) as count FROM sessions WHERE wallet_address = $1"
        )
        .bind::<diesel::sql_types::Text, _>(&query.wallet_address)
        .get_result::<CountRow>(&mut conn)
        .await
        .map(|r| r.count as i32)
        .unwrap_or(0);

        // Get login count in last 30 days from sessions table
        let last_30_days_logins: i32 = diesel::sql_query(
            "SELECT COUNT(*) as count FROM sessions
             WHERE wallet_address = $1
             AND created_at >= NOW() - INTERVAL '30 days'"
        )
        .bind::<diesel::sql_types::Text, _>(&query.wallet_address)
        .get_result::<CountRow>(&mut conn)
        .await
        .map(|r| r.count as i32)
        .unwrap_or(0);

        let activity_summary = WalletActivitySummaryDto {
            total_logins,
            last_30_days_logins,
            total_permissions: permissions.len() as i32,
            active_permissions: active_permissions_count as i32,
            expired_permissions: (permissions.len() - active_permissions_count) as i32,
            plans_count: plans.len() as i32,
        };

        // 7. Build wallet detail DTO
        let wallet_detail = WalletDetailDto {
            wallet_address: wallet.wallet_address,
            is_active: wallet.is_active,
            created_at: wallet.created_at,
            last_auth_at: wallet.last_auth_at,
            permissions,
            plans,
            activity_summary,
            metadata: wallet.wallet_metadata,
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
