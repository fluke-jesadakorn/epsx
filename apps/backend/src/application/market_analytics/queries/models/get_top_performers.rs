use crate::application::market_analytics::queries::StockAnalysisSummary;
use crate::application::shared::Query;
use crate::prelude::*;

/// Query to get top performing stocks
#[derive(Debug, Clone)]
pub struct GetTopPerformersQuery {
    pub limit: u32,
}

impl Query for GetTopPerformersQuery {
    type Response = GetTopPerformersResponse;
}

/// Response containing top performing stocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTopPerformersResponse {
    pub stocks: Vec<StockAnalysisSummary>,
    pub count: usize,
}
