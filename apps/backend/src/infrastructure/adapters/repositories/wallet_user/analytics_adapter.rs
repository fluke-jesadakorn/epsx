use crate::prelude::*;
use crate::domain::wallet_management::repository_ports::{WalletUserAnalyticsPort, WalletUserStatistics};

pub struct PostgresWalletUserAnalyticsAdapter {
    db_pool: &'static TlsPool,
}

impl PostgresWalletUserAnalyticsAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl WalletUserAnalyticsPort for PostgresWalletUserAnalyticsAdapter {
    async fn get_statistics(&self) -> AppResult<WalletUserStatistics> {
        let row: (i64, i64, i64, i64) = sqlx::query_as(
            "SELECT
                COUNT(*) as total_users,
                COUNT(*) FILTER (WHERE is_active = TRUE) as active_users,
                COUNT(*) FILTER (WHERE last_auth_at > NOW() - INTERVAL '24 hours') as recent_auth_24h,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') as new_wallets_24h
             FROM wallet_users",
        )
        .fetch_one(self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get wallet user statistics: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_analytics_adapter")
                .with_operation("get_statistics")
        })?;

        Ok(WalletUserStatistics {
            total_users: row.0 as u64,
            active_users: row.1 as u64,
            recent_auth_24h: row.2 as u64,
            new_wallets_24h: row.3 as u64,
        })
    }
}
