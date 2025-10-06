// Get Portfolio Rankings Query
// Portfolio view with positive-growth stocks only

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;
use crate::application::trading_analytics::dtos::{CardDashboardResponse};

#[derive(Debug, Clone)]
pub struct GetPortfolioRankingsQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: Option<String>,
    pub min_growth: Option<f64>,
}

impl Query for GetPortfolioRankingsQuery {
    type Response = GetPortfolioRankingsResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPortfolioRankingsResponse {
    pub rankings: CardDashboardResponse,
}
