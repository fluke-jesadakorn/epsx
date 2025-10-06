use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::trading_analytics::queries::{
    GetTopPerformersQuery, GetTopPerformersResponse, StockAnalysisSummary
};
use crate::domain::trading_analytics::StockAnalysisRepositoryPort;

/// Query handler for getting top performing stocks
pub struct GetTopPerformersQueryHandler {
    stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
}

impl GetTopPerformersQueryHandler {
    pub fn new(stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>) -> Self {
        Self {
            stock_analysis_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetTopPerformersQuery> for GetTopPerformersQueryHandler {
    async fn handle(&self, query: GetTopPerformersQuery) -> ApplicationResult<GetTopPerformersResponse> {
        // 1. Get top performers from repository
        let analyses = self.stock_analysis_repository
            .find_top_performers(query.limit)
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
        Ok(GetTopPerformersResponse { stocks, count })
    }
}
