use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::market_analytics::queries::{
    ListStockAnalysesQuery, ListStockAnalysesResponse, StockAnalysisSummary
};
use crate::domain::market_analytics::{
    StockAnalysisRepositoryPort, StockAnalysisSearchCriteria, MarketSector, Country
};
use crate::web::pagination::Pagination;

/// Query handler for listing stock analyses
pub struct ListStockAnalysesQueryHandler {
    stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>,
}

impl ListStockAnalysesQueryHandler {
    pub fn new(stock_analysis_repository: Arc<dyn StockAnalysisRepositoryPort>) -> Self {
        Self {
            stock_analysis_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<ListStockAnalysesQuery> for ListStockAnalysesQueryHandler {
    async fn handle(&self, query: ListStockAnalysesQuery) -> ApplicationResult<ListStockAnalysesResponse> {
        // 1. Parse filters
        let sector = if let Some(sector_str) = query.sector {
            Some(MarketSector::new(sector_str)
                .map_err(|e| ApplicationError::validation("sector", e.to_string()))?)
        } else {
            None
        };

        let country = if let Some(country_str) = query.country {
            Some(Country::new(country_str)
                .map_err(|e| ApplicationError::validation("country", e.to_string()))?)
        } else {
            None
        };

        // 2. Build search criteria with pagination
        let pg = Pagination::standard(query.page, query.limit);

        let criteria = StockAnalysisSearchCriteria {
            sector,
            country,
            min_score: query.min_score,
            min_growth: query.min_growth,
            limit: Some(pg.limit as i64),
            offset: Some(pg.offset),
        };

        // 3. Get analyses and total count
        let analyses = self.stock_analysis_repository.find_all(criteria.clone()).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        let total = self.stock_analysis_repository.count(criteria).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Map to summaries
        let summaries: Vec<StockAnalysisSummary> = analyses.into_iter().map(|analysis| {
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

        // 5. Return response
        Ok(ListStockAnalysesResponse {
            analyses: summaries,
            total,
            page: pg.page,
            limit: pg.limit,
        })
    }
}
