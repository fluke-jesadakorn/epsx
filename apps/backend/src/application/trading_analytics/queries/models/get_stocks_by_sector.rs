use crate::prelude::*;
use crate::application::shared::Query;
use crate::application::trading_analytics::queries::StockAnalysisSummary;

/// Query to get stocks in a specific sector
#[derive(Debug, Clone)]
pub struct GetStocksBySectorQuery {
    pub sector: String,
}

impl Query for GetStocksBySectorQuery {
    type Response = GetStocksBySectorResponse;
}

/// Response containing stocks in a sector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStocksBySectorResponse {
    pub sector: String,
    pub stocks: Vec<StockAnalysisSummary>,
    pub count: usize,
}
