use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::trading_analytics::queries::{
    GetStocksBySectorQuery, GetStocksBySectorResponse, StockAnalysisSummary
};
use crate::domain::trading_analytics::{StockAnalysisRepositoryPort, MarketSector};

/// Query handler for getting stocks in a specific sector
pub struct GetStocksBySectorQueryHandler {
    stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
}

impl GetStocksBySectorQueryHandler {
    pub fn new(stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>) -> Self {
        Self {
            stock_analysis_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetStocksBySectorQuery> for GetStocksBySectorQueryHandler {
    async fn handle(&self, query: GetStocksBySectorQuery) -> ApplicationResult<GetStocksBySectorResponse> {
        // 1. Validate and parse sector
        let sector = MarketSector::new(query.sector.clone())
            .map_err(|e| ApplicationError::validation("sector", e.to_string()))?;

        // 2. Get stocks from repository
        let analyses = self.stock_analysis_repository
            .find_by_sector(&sector)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Map to summaries
        let stocks: Vec<StockAnalysisSummary> = analyses.into_iter().map(|analysis| {
            StockAnalysisSummary {
                symbol: analysis.symbol().to_string(),
                company_name: analysis.company_name().to_string(),
                current_eps: analysis.current_eps().value(),
                eps_growth: analysis.eps_growth().value(),
                growth_classification: analysis.eps_growth().classify().as_str().to_string(),
                sector: analysis.sector().name().to_string(),
                country: analysis.country().name().to_string(),
                analysis_score: analysis.analysis_score().overall_score,
                last_updated: analysis.last_updated(),
            }
        }).collect();

        let count = stocks.len();

        // 4. Return response
        Ok(GetStocksBySectorResponse {
            sector: query.sector,
            stocks,
            count,
        })
    }
}
