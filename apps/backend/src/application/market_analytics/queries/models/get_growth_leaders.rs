use crate::prelude::*;
use crate::application::shared::Query;
use crate::application::market_analytics::queries::StockAnalysisSummary;

/// Query to get stocks with highest growth rates
#[derive(Debug, Clone)]
pub struct GetGrowthLeadersQuery {
    pub min_growth: f64,
    pub limit: u32,
}

impl Query for GetGrowthLeadersQuery {
    type Response = GetGrowthLeadersResponse;
}

/// Response containing growth leader stocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetGrowthLeadersResponse {
    pub stocks: Vec<StockAnalysisSummary>,
    pub min_growth: f64,
    pub count: usize,
}
