use async_trait::async_trait;
use epsx_contracts::errors::AppError;

use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;

/// Provider-neutral request for a page of market rankings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketRankingsRequest {
    pub skip: i32,
    pub limit: i32,
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: Option<String>,
}

/// Provider-neutral market rankings page.
#[derive(Debug, Clone)]
pub struct MarketRankingsPage {
    pub items: Vec<StockScreeningResult>,
    pub total: i32,
}

/// Domain boundary for retrieving market rankings from an external provider.
#[async_trait]
pub trait MarketRankingsProviderPort: Send + Sync {
    async fn fetch_rankings(
        &self,
        request: MarketRankingsRequest,
    ) -> Result<MarketRankingsPage, AppError>;
}
