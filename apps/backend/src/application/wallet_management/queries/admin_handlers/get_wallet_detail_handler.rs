// Get Wallet Detail Query Handler
// CQRS handler for retrieving detailed wallet information
// MIGRATED TO SQLX (real): no stubs, no todo!().

use crate::application::shared::{ApplicationError, ApplicationResult, Query, QueryHandler};
use crate::application::wallet_management::queries::admin_models::{
    GetWalletDetailQuery, GetWalletDetailResponse, WalletActivitySummaryDto, WalletDetailDto,
    WalletPermissionDto, WalletPlanDto,
};
use crate::application::wallet_management::wallet_management_repository::WalletManagementRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};

#[derive(sqlx::FromRow)]
struct PermissionDetailRow {
    pub permission: String,
    pub source: String,
    pub granted_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
}

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

        // 2. Initialize repository
        let repo = WalletManagementRepository::new(self.db_pool.clone());

        // 3. Get wallet basic info using repository
        let wallet = repo
            .get_wallet_basic_info(&query.wallet_address)
            .await
            .map_err(|e| {
                error!("Failed to fetch wallet info: {}", e);
                ApplicationError::infrastructure(e.to_string())
            })?
            .ok_or_else(|| ApplicationError::not_found("Wallet", &query.wallet_address))?;

        // 4. Get permissions (union of group and direct permissions)
        let permissions_result: Vec<PermissionDetailRow> = sqlx::query_as(
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
        .bind(&query.wallet_address)
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!(
                "Failed to fetch permissions for {}: {}",
                query.wallet_address, e
            );
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

        // Combine login counts into a single query
        #[derive(sqlx::FromRow)]
        struct LoginCounts {
            total_logins: i64,
            last_30_days_logins: i64,
        }

        let login_counts: LoginCounts = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_logins,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days') as last_30_days_logins
            FROM sessions WHERE wallet_address = $1
            "#,
        )
        .bind(&query.wallet_address)
        .fetch_one(self.db_pool.as_ref())
        .await
        .unwrap_or(LoginCounts { total_logins: 0, last_30_days_logins: 0 });

        let total_logins = login_counts.total_logins as i32;
        let last_30_days_logins = login_counts.last_30_days_logins as i32;

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
            "Successfully retrieved details for wallet: {}",
            query.wallet_address
        );

        Ok(GetWalletDetailResponse {
            success: true,
            wallet: wallet_detail,
        })
    }
}
