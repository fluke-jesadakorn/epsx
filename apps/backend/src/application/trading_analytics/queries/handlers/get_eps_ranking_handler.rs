use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::trading_analytics::queries::{
    GetEPSRankingQuery, GetEPSRankingResponse, RankingEntryDTO, RankingStatisticsDTO
};
use crate::domain::trading_analytics::EPSRankingRepositoryPort;

/// Query handler for getting a single EPS ranking
pub struct GetEPSRankingQueryHandler {
    ranking_repository: Arc<dyn EPSRankingRepositoryPort>,
}

impl GetEPSRankingQueryHandler {
    pub fn new(ranking_repository: Arc<dyn EPSRankingRepositoryPort>) -> Self {
        Self {
            ranking_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetEPSRankingQuery> for GetEPSRankingQueryHandler {
    async fn handle(&self, query: GetEPSRankingQuery) -> ApplicationResult<GetEPSRankingResponse> {
        // 1. Find ranking
        let ranking = self.ranking_repository.find_by_id(&query.ranking_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("ranking_id", "Ranking not found"))?;

        // 2. Map entries to DTOs
        let entries: Vec<RankingEntryDTO> = ranking.entries()
            .iter()
            .map(|(rank, entry)| RankingEntryDTO {
                rank: *rank,
                symbol: entry.symbol.to_string(),
                company_name: entry.company_name.clone(),
                eps_value: entry.eps_value.value(),
                growth_factor: entry.growth_factor.value(),
                sector: entry.sector.name().to_string(),
                country: entry.country.name().to_string(),
                score: entry.score,
            })
            .collect();

        // 3. Map statistics to DTO
        let stats = ranking.statistics();
        let statistics = RankingStatisticsDTO {
            average_eps: stats.avg_eps,
            average_growth: stats.avg_growth,
            highest_eps: stats.top_score,
            lowest_eps: stats.bottom_score,
        };

        // 4. Build response
        Ok(GetEPSRankingResponse {
            ranking_id: query.ranking_id,
            ranking_type: ranking.ranking_type().to_string(),
            time_period: ranking.time_period().to_string(),
            sector_filter: ranking.sector_filter().map(|s| s.to_string()),
            country_filter: ranking.country_filter().map(|c| c.name().to_string()),
            entries,
            total_entries: ranking.total_entries(),
            statistics,
            last_updated: ranking.last_updated(),
        })
    }
}
