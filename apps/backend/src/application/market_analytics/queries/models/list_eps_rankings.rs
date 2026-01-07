use crate::prelude::*;
use crate::application::shared::Query;

/// Query to list EPS rankings with optional filtering
#[derive(Debug, Clone)]
pub struct ListEPSRankingsQuery {
    pub ranking_type: Option<String>,
    pub time_period: Option<String>,
    pub sector: Option<String>,
    pub country: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Query for ListEPSRankingsQuery {
    type Response = ListEPSRankingsResponse;
}

/// Response containing paginated list of EPS rankings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEPSRankingsResponse {
    pub rankings: Vec<EPSRankingSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

/// Summary view of an EPS ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPSRankingSummary {
    pub ranking_id: String,
    pub ranking_type: String,
    pub time_period: String,
    pub sector_filter: Option<String>,
    pub country_filter: Option<String>,
    pub total_entries: u32,
    pub last_updated: DateTime<Utc>,
}
