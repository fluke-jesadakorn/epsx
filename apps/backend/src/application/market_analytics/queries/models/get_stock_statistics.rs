use crate::prelude::*;
use crate::application::shared::Query;
use crate::domain::market_analytics::repository_ports::StockAnalysisStatistics;

/// Query to get overall stock analysis statistics
#[derive(Debug, Clone)]
pub struct GetStockStatisticsQuery;

impl Query for GetStockStatisticsQuery {
    type Response = GetStockStatisticsResponse;
}

/// Response containing stock analysis statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStockStatisticsResponse {
    pub statistics: StockAnalysisStatistics,
}
