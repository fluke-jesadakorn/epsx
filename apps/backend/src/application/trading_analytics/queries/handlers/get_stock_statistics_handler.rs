use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::trading_analytics::queries::{
    GetStockStatisticsQuery, GetStockStatisticsResponse
};
use crate::domain::trading_analytics::StockAnalysisRepositoryPort;

/// Query handler for getting overall stock analysis statistics
pub struct GetStockStatisticsQueryHandler {
    stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
}

impl GetStockStatisticsQueryHandler {
    pub fn new(stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>) -> Self {
        Self {
            stock_analysis_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetStockStatisticsQuery> for GetStockStatisticsQueryHandler {
    async fn handle(&self, _query: GetStockStatisticsQuery) -> ApplicationResult<GetStockStatisticsResponse> {
        // 1. Get statistics from repository
        let statistics = self.stock_analysis_repository
            .get_statistics()
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 2. Return response
        Ok(GetStockStatisticsResponse { statistics })
    }
}
