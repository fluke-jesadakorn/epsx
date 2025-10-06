use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::trading_analytics::queries::{
    GetStockAnalysisQuery, GetStockAnalysisResponse, RankingSummary
};
use crate::domain::trading_analytics::{StockAnalysisRepositoryPort, StockSymbol};

/// Query handler for getting a single stock analysis
pub struct GetStockAnalysisQueryHandler {
    stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
}

impl GetStockAnalysisQueryHandler {
    pub fn new(stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>) -> Self {
        Self {
            stock_analysis_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetStockAnalysisQuery> for GetStockAnalysisQueryHandler {
    async fn handle(&self, query: GetStockAnalysisQuery) -> ApplicationResult<GetStockAnalysisResponse> {
        // 1. Validate symbol
        let symbol = StockSymbol::new(query.symbol.clone())
            .map_err(|e| ApplicationError::validation("symbol", e.to_string()))?;

        // 2. Find stock analysis
        let stock_analysis = self.stock_analysis_repository.find_by_symbol(&symbol).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("symbol", "Stock analysis not found"))?;

        // 3. Map rankings to summary
        let rankings: Vec<RankingSummary> = stock_analysis.rankings()
            .iter()
            .map(|(category, ranking)| RankingSummary {
                ranking_id: format!("{:?}", category),
                ranking_type: category.to_string(),
                rank: ranking.rank,
                total_entries: ranking.total_stocks,
            })
            .collect();

        // 4. Build response
        Ok(GetStockAnalysisResponse {
            symbol: query.symbol,
            company_name: stock_analysis.company_name().to_string(),
            current_eps: stock_analysis.current_eps().value(),
            previous_eps: stock_analysis.previous_eps().value(),
            eps_growth: stock_analysis.eps_growth().value(),
            growth_classification: stock_analysis.eps_growth().classify().as_str().to_string(),
            sector: stock_analysis.sector().name().to_string(),
            country: stock_analysis.country().name().to_string(),
            analysis_score: stock_analysis.analysis_score().overall_score,
            investment_recommendation: stock_analysis.investment_recommendation().as_str().to_string(),
            rankings,
            last_updated: stock_analysis.last_updated(),
        })
    }
}
