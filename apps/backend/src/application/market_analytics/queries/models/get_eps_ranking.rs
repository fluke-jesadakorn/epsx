use crate::prelude::*;
use crate::application::shared::Query;

/// Query to get a single EPS ranking by ID
#[derive(Debug, Clone)]
pub struct GetEPSRankingQuery {
    pub ranking_id: String,
}

impl Query for GetEPSRankingQuery {
    type Response = GetEPSRankingResponse;
}

/// Response containing EPS ranking details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEPSRankingResponse {
    pub ranking_id: String,
    pub ranking_type: String,
    pub time_period: String,
    pub sector_filter: Option<String>,
    pub country_filter: Option<String>,
    pub entries: Vec<RankingEntryDTO>,
    pub total_entries: u32,
    pub statistics: RankingStatisticsDTO,
    pub last_updated: DateTime<Utc>,
}

/// DTO for a ranking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingEntryDTO {
    pub rank: u32,
    pub symbol: String,
    pub company_name: String,
    pub eps_value: f64,
    pub growth_factor: f64,
    pub sector: String,
    pub country: String,
    pub score: f64,
}

/// DTO for ranking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingStatisticsDTO {
    pub average_eps: f64,
    pub average_growth: f64,
    pub highest_eps: f64,
    pub lowest_eps: f64,
}
