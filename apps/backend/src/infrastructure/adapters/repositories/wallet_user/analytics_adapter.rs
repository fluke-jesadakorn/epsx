use crate::prelude::*;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
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
        let mut conn = self.db_pool.conn().await?;

        #[derive(diesel::QueryableByName)]
        struct StatsResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_users: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            active_users: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            recent_auth_24h: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            new_wallets_24h: i64,
        }

        let query = r#"
            SELECT
                COUNT(*) as total_users,
                COUNT(*) FILTER (WHERE is_active = true) as active_users,
                COUNT(*) FILTER (WHERE last_auth_at > NOW() - INTERVAL '24 hours') as recent_auth_24h,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') as new_wallets_24h
            FROM wallet_users
        "#;

        let results = diesel::sql_query(query)
            .load::<StatsResult>(&mut conn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get wallet user statistics: {}", e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_analytics_adapter")
                    .with_operation("get_statistics")
            })?;

        let stats = results.first().ok_or_else(|| {
            AppError::database_error("No statistics returned".to_string())
                .with_component("wallet_user_analytics_adapter")
        })?;

        Ok(WalletUserStatistics {
            total_users: stats.total_users as u64,
            active_users: stats.active_users as u64,
            recent_auth_24h: stats.recent_auth_24h as u64,
            new_wallets_24h: stats.new_wallets_24h as u64,
        })
    }
}

