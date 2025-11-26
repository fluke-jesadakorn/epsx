use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::trading_analytics::queries::{
    ListEPSRankingsQuery, ListEPSRankingsResponse, EPSRankingSummary
};
use crate::domain::trading_analytics::{
    EPSRankingRepositoryPort, EPSRankingSearchCriteria, RankingType, RankingPeriod, SectorCategory, Country
};

/// Query handler for listing EPS rankings
pub struct ListEPSRankingsQueryHandler {
    ranking_repository: Arc<dyn EPSRankingRepositoryPort>,
}

impl ListEPSRankingsQueryHandler {
    pub fn new(ranking_repository: Arc<dyn EPSRankingRepositoryPort>) -> Self {
        Self {
            ranking_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<ListEPSRankingsQuery> for ListEPSRankingsQueryHandler {
    async fn handle(&self, query: ListEPSRankingsQuery) -> ApplicationResult<ListEPSRankingsResponse> {
        // 1. Parse filters
        let ranking_type = if let Some(type_str) = query.ranking_type {
            Some(RankingType::from_str(&type_str)
                .map_err(|e| ApplicationError::validation("ranking_type", e.to_string()))?)
        } else {
            None
        };

        let time_period = if let Some(period_str) = query.time_period {
            Some(RankingPeriod::from_str(&period_str)
                .map_err(|e| ApplicationError::validation("time_period", e.to_string()))?)
        } else {
            None
        };

        let sector_filter = if let Some(sector_str) = query.sector {
            Some(SectorCategory::from_str(&sector_str)
                .map_err(|e| ApplicationError::validation("sector", e.to_string()))?)
        } else {
            None
        };

        let country_filter = if let Some(country_str) = query.country {
            Some(Country::new(country_str)
                .map_err(|e| ApplicationError::validation("country", e.to_string()))?)
        } else {
            None
        };

        // 2. Build search criteria
        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(20).min(100) as i64;
        let offset = ((page - 1) * (limit as u32)) as i64;

        let criteria = EPSRankingSearchCriteria {
            ranking_type,
            time_period,
            sector_filter,
            country_filter,
            limit: Some(limit),
            offset: Some(offset),
        };

        // 3. Get rankings and total count
        let rankings = self.ranking_repository.find_all(criteria.clone()).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        let total = self.ranking_repository.count(criteria).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Map to summaries
        let summaries: Vec<EPSRankingSummary> = rankings.into_iter().map(|ranking| {
            EPSRankingSummary {
                ranking_id: ranking.ranking_id().to_string(),
                ranking_type: ranking.ranking_type().to_string(),
                time_period: ranking.time_period().to_string(),
                sector_filter: ranking.sector_filter().map(|s| s.to_string()),
                country_filter: ranking.country_filter().map(|c| c.name().to_string()),
                total_entries: ranking.total_entries(),
                last_updated: ranking.last_updated(),
            }
        }).collect();

        // 5. Return response
        Ok(ListEPSRankingsResponse {
            rankings: summaries,
            total,
            page,
            limit: limit as u32,
        })
    }
}
