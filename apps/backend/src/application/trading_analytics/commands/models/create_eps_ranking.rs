use crate::prelude::*;
use crate::application::shared::Command;

/// Command to create a new EPS ranking
///
/// Creates a ranking that can track and compare multiple stocks based on EPS metrics.
/// Supports filtering by sector and country to create targeted rankings.
#[derive(Debug, Clone)]
pub struct CreateEPSRankingCommand {
    /// Type of ranking: "growth", "absolute", "quality"
    pub ranking_type: String,

    /// Time period: "quarterly", "annual", "ytd"
    pub time_period: String,

    /// Optional sector filter (e.g., "Technology")
    pub sector_filter: Option<String>,

    /// Optional country filter (e.g., "US")
    pub country_filter: Option<String>,
}

impl Command for CreateEPSRankingCommand {
    type Response = CreateEPSRankingResponse;
}

/// Response returned after successfully creating an EPS ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEPSRankingResponse {
    /// Generated ranking ID
    pub ranking_id: String,

    /// Ranking type
    pub ranking_type: String,

    /// Time period
    pub time_period: String,

    /// Applied filters
    pub filters: RankingFilters,

    /// Timestamp when ranking was created
    pub created_at: DateTime<Utc>,
}

/// Filters applied to a ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingFilters {
    pub sector: Option<String>,
    pub country: Option<String>,
}
