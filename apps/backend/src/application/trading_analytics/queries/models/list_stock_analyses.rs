use crate::prelude::*;
use crate::application::shared::Query;

/// Query to list stock analyses with optional filtering
#[derive(Debug, Clone)]
pub struct ListStockAnalysesQuery {
    pub sector: Option<String>,
    pub country: Option<String>,
    pub min_growth: Option<f64>,
    pub min_score: Option<u8>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Query for ListStockAnalysesQuery {
    type Response = ListStockAnalysesResponse;
}

/// Response containing paginated list of stock analyses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStockAnalysesResponse {
    pub analyses: Vec<StockAnalysisSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

/// Summary view of a stock analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAnalysisSummary {
    pub symbol: String,
    pub company_name: String,
    pub current_eps: f64,
    pub eps_growth: f64,
    pub growth_classification: String,
    pub sector: String,
    pub country: String,
    pub analysis_score: u8,
    pub last_updated: DateTime<Utc>,
}
