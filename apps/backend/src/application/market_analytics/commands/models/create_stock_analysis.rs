use crate::prelude::*;
use crate::application::shared::Command;

/// Command to create a new stock analysis
///
/// This command initiates the creation of a comprehensive stock analysis including:
/// - EPS tracking (current and historical)
/// - Growth factor calculation
/// - Sector and country classification
/// - Automatic analysis scoring
#[derive(Debug, Clone)]
pub struct CreateStockAnalysisCommand {
    /// Stock ticker symbol (e.g., "AAPL", "GOOGL")
    pub symbol: String,

    /// Full company name (e.g., "Apple Inc.")
    pub company_name: String,

    /// Current quarter/annual EPS value
    pub current_eps: f64,

    /// Previous quarter/annual EPS value for growth calculation
    pub previous_eps: f64,

    /// Market sector (e.g., "Technology", "Finance")
    pub sector: String,

    /// Country code (e.g., "US", "UK", "JP")
    pub country: String,
}

impl Command for CreateStockAnalysisCommand {
    type Response = CreateStockAnalysisResponse;
}

/// Response returned after successfully creating a stock analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStockAnalysisResponse {
    /// Stock symbol
    pub symbol: String,

    /// Company name
    pub company_name: String,

    /// Calculated analysis score (0-100)
    pub analysis_score: u8,

    /// Calculated EPS growth percentage
    pub eps_growth: f64,

    /// Growth classification
    pub growth_classification: String,

    /// Timestamp when analysis was created
    pub created_at: DateTime<Utc>,
}
