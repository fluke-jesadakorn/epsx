use crate::prelude::*;
use crate::application::shared::Command;

/// Command to update an existing stock analysis
///
/// Allows partial updates to stock analysis data. Any field that is None will not be updated.
/// When EPS values are updated, growth factors and scores are automatically recalculated.
#[derive(Debug, Clone)]
pub struct UpdateStockAnalysisCommand {
    /// Stock symbol to update
    pub symbol: String,

    /// New current EPS value (optional)
    pub current_eps: Option<f64>,

    /// New previous EPS value (optional)
    pub previous_eps: Option<f64>,

    /// New sector classification (optional)
    pub sector: Option<String>,

    /// New country (optional)
    pub country: Option<String>,
}

impl Command for UpdateStockAnalysisCommand {
    type Response = UpdateStockAnalysisResponse;
}

/// Response returned after successfully updating a stock analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStockAnalysisResponse {
    /// Stock symbol
    pub symbol: String,

    /// New analysis score after update
    pub analysis_score: u8,

    /// New EPS growth after update
    pub eps_growth: f64,

    /// Timestamp of update
    pub updated_at: DateTime<Utc>,
}
