// Admin Get Wallet Stats Query
// Query to retrieve global wallet statistics for admin dashboard

use crate::application::shared::{ApplicationResult, Query};
use serde::{Deserialize, Serialize};

// ============================================================================
// QUERY
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct GetWalletStatsQuery {
    // No parameters - returns global stats
}

impl Query for GetWalletStatsQuery {
    type Response = GetWalletStatsResponse;

    fn validate(&self) -> ApplicationResult<()> {
        // No validation needed
        Ok(())
    }
}

// ============================================================================
// RESPONSE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWalletStatsResponse {
    pub success: bool,
    pub stats: WalletStatsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStatsDto {
    pub total_users: i32,
    pub active_users: i32,
    pub inactive_users: i32,
    pub new_users_30_days: i32,
    pub active_users_30_days: i32,
    pub growth_rate: f64,
}
