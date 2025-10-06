// Get Wallet Stats Query Handler
// CQRS handler for retrieving global wallet statistics

use crate::application::shared::{ApplicationError, ApplicationResult, Query, QueryHandler};
use crate::application::wallet_management::queries::admin_models::{
    GetWalletStatsQuery, GetWalletStatsResponse, WalletStatsDto,
};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};

pub struct GetWalletStatsQueryHandler {
    db_pool: Arc<PgPool>,
}

impl GetWalletStatsQueryHandler {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl QueryHandler<GetWalletStatsQuery> for GetWalletStatsQueryHandler {
    async fn handle(
        &self,
        query: GetWalletStatsQuery,
    ) -> ApplicationResult<GetWalletStatsResponse> {
        // 1. Validate query
        query.validate()?;

        // 2. Get wallet statistics
        let stats_result = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_users,
                COUNT(*) FILTER (WHERE is_active = true) as active_users,
                COUNT(*) FILTER (WHERE is_active = false) as inactive_users,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days') as new_users_30_days
            FROM wallet_users
            "#
        )
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("❌ Failed to fetch wallet statistics: {}", e);
            ApplicationError::infrastructure(format!("Failed to fetch stats: {}", e))
        })?;

        // 3. Calculate growth rate
        let total = stats_result.total_users.unwrap_or(0);
        let new_30_days = stats_result.new_users_30_days.unwrap_or(0);
        let growth_rate = if total > 0 {
            (new_30_days as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // 4. Build stats DTO
        let stats = WalletStatsDto {
            total_users: total as i32,
            active_users: stats_result.active_users.unwrap_or(0) as i32,
            inactive_users: stats_result.inactive_users.unwrap_or(0) as i32,
            new_users_30_days: new_30_days as i32,
            active_users_30_days: stats_result.active_users.unwrap_or(0) as i32,
            growth_rate,
        };

        info!(
            "✅ Successfully retrieved wallet statistics: {} total users, {:.2}% growth",
            stats.total_users, growth_rate
        );

        Ok(GetWalletStatsResponse {
            success: true,
            stats,
        })
    }
}
