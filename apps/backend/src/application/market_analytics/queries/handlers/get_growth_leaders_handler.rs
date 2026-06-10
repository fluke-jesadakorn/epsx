use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::market_analytics::queries::{
    GetGrowthLeadersQuery, GetGrowthLeadersResponse, StockAnalysisSummary
};
use crate::domain::market_analytics::StockAnalysisRepositoryPort;

/// Query handler for getting stocks with highest growth rates
pub struct GetGrowthLeadersQueryHandler {
    stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
}

impl GetGrowthLeadersQueryHandler {
    pub fn new(stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>) -> Self {
        Self {
            stock_analysis_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetGrowthLeadersQuery> for GetGrowthLeadersQueryHandler {
    async fn handle(&self, query: GetGrowthLeadersQuery) -> ApplicationResult<GetGrowthLeadersResponse> {
        // 1. Get growth leaders from repository
        let analyses = self.stock_analysis_repository
            .find_growth_leaders(query.min_growth, query.limit)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 2. Map to summaries
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

        // 3. Return response
        Ok(GetGrowthLeadersResponse {
            stocks,
            min_growth: query.min_growth,
            count,
        })
    }
}
