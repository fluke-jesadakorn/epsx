use crate::prelude::*;
use crate::application::shared::Command;

/// Command to add a stock to an existing EPS ranking
///
/// Adds a stock entry to a ranking with automatic position calculation based on score.
/// The ranking will validate that the stock matches any applied filters (sector/country).
#[derive(Debug, Clone)]
pub struct AddStockToRankingCommand {
    /// Ranking ID to add stock to
    pub ranking_id: String,

    /// Stock symbol
    pub symbol: String,

    /// Company name
    pub company_name: String,

    /// EPS value for ranking
    pub eps_value: f64,

    /// Growth factor percentage
    pub growth_factor: f64,

    /// Stock sector (must match ranking filter if present)
    pub sector: String,

    /// Stock country (must match ranking filter if present)
    pub country: String,
}

impl Command for AddStockToRankingCommand {
    type Response = AddStockToRankingResponse;
}

/// Response returned after successfully adding a stock to a ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStockToRankingResponse {
    /// Ranking ID
    pub ranking_id: String,

    /// Stock symbol added
    pub symbol: String,

    /// Position/rank assigned (1 = highest)
    pub rank: u32,

    /// Calculated score
    pub score: f64,

    /// Timestamp of addition
    pub added_at: DateTime<Utc>,
}
