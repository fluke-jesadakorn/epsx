use crate::prelude::*;
use crate::application::shared::Command;

/// Command to delete a stock analysis
///
/// Removes a stock analysis from the system. This operation:
/// - Deletes the stock analysis aggregate
/// - Removes the stock from all rankings it appears in
/// - Publishes domain events for audit trail
#[derive(Debug, Clone)]
pub struct DeleteStockAnalysisCommand {
    /// Stock symbol to delete
    pub symbol: String,
}

impl Command for DeleteStockAnalysisCommand {
    type Response = DeleteStockAnalysisResponse;
}

/// Response returned after successfully deleting a stock analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteStockAnalysisResponse {
    /// Symbol that was deleted
    pub symbol: String,

    /// Timestamp of deletion
    pub deleted_at: DateTime<Utc>,
}
