use crate::prelude::*;
use crate::domain::market_analytics::{StockAnalysis, StockSymbol, MarketSector, Country};

/// Search criteria for stock analyses
#[derive(Debug, Clone, Default)]
pub struct StockAnalysisSearchCriteria {
    pub sector: Option<MarketSector>,
    pub country: Option<Country>,
    pub min_score: Option<u8>,
    pub min_growth: Option<f64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Stock analysis statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAnalysisStatistics {
    pub total_analyses: i64,
    pub average_score: f64,
    pub average_growth: f64,
    pub high_growth_count: i64, // growth > 20%
}

/// Repository port for stock analysis operations
#[async_trait]
pub trait StockAnalysisRepositoryPort: Send + Sync {
    /// Find stock analysis by symbol
    async fn find_by_symbol(&self, symbol: &StockSymbol) -> AppResult<Option<StockAnalysis>>;

    /// List stock analyses with optional filtering
    async fn find_all(&self, criteria: StockAnalysisSearchCriteria) -> AppResult<Vec<StockAnalysis>>;

    /// Save (create or update) a stock analysis
    async fn save(&self, analysis: &StockAnalysis) -> AppResult<()>;

    /// Delete a stock analysis
    async fn delete(&self, symbol: &StockSymbol) -> AppResult<()>;

    /// Count analyses matching criteria
    async fn count(&self, criteria: StockAnalysisSearchCriteria) -> AppResult<i64>;

    /// Get statistics
    async fn get_statistics(&self) -> AppResult<StockAnalysisStatistics>;

    /// Check if symbol exists
    async fn symbol_exists(&self, symbol: &StockSymbol) -> AppResult<bool>;

    /// Get top performers by score
    async fn find_top_performers(&self, limit: u32) -> AppResult<Vec<StockAnalysis>>;

    /// Get stocks by sector
    async fn find_by_sector(&self, sector: &MarketSector) -> AppResult<Vec<StockAnalysis>>;

    /// Get growth leaders (growth > threshold)
    async fn find_growth_leaders(&self, min_growth: f64, limit: u32) -> AppResult<Vec<StockAnalysis>>;
}
