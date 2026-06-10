// Get Stock Ranking Assignments Query
// Query user stock ranking package assignments

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;

#[derive(Debug, Clone)]
pub struct GetStockRankingAssignmentsQuery {
    pub wallet_address: Option<String>,
    pub package_id: Option<String>,
    pub active_only: Option<bool>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

impl Query for GetStockRankingAssignmentsQuery {
    type Response = GetStockRankingAssignmentsResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStockRankingAssignmentsResponse {
    pub success: bool,
    pub assignments: Vec<StockRankingAssignment>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockRankingAssignment {
    pub assignment_id: String,
    pub wallet_address: String,
    pub package_id: String,
    pub package_name: String,
    pub rank_access_level: i32,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
    pub assignment_source: String,
    pub auto_renew: bool,
    pub days_remaining: Option<i32>,
}
