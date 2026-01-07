use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get a single stock analysis by symbol
#[derive(Debug, Clone)]
pub struct GetStockAnalysisQuery {
    pub symbol: String,
}

impl Query for GetStockAnalysisQuery {
    type Response = GetStockAnalysisResponse;
}

/// Response containing stock analysis details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStockAnalysisResponse {
    pub symbol: String,
    pub company_name: String,
    pub current_eps: f64,
    pub previous_eps: f64,
    pub eps_growth: f64,
    pub growth_classification: String,
    pub sector: String,
    pub country: String,
    pub analysis_score: u8,
    pub investment_recommendation: String,
    pub rankings: Vec<RankingSummary>,
    pub last_updated: DateTime<Utc>,
}

/// Summary of a ranking the stock appears in
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingSummary {
    pub ranking_id: String,
    pub ranking_type: String,
    pub rank: u32,
    pub total_entries: u32,
}
