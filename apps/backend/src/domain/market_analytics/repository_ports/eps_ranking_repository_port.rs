use crate::prelude::*;
use crate::domain::market_analytics::{EPSRanking, RankingType, RankingPeriod, Country, SectorCategory};

/// Search criteria for EPS rankings
#[derive(Debug, Clone, Default)]
pub struct EPSRankingSearchCriteria {
    pub ranking_type: Option<RankingType>,
    pub time_period: Option<RankingPeriod>,
    pub sector_filter: Option<SectorCategory>,
    pub country_filter: Option<Country>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Repository port for EPS ranking operations
#[async_trait]
pub trait EPSRankingRepositoryPort: Send + Sync {
    /// Find ranking by ID
    async fn find_by_id(&self, ranking_id: &str) -> AppResult<Option<EPSRanking>>;

    /// List rankings with optional filtering
    async fn find_all(&self, criteria: EPSRankingSearchCriteria) -> AppResult<Vec<EPSRanking>>;

    /// Save (create or update) a ranking
    async fn save(&self, ranking: &EPSRanking) -> AppResult<()>;

    /// Delete a ranking
    async fn delete(&self, ranking_id: &str) -> AppResult<()>;

    /// Count rankings matching criteria
    async fn count(&self, criteria: EPSRankingSearchCriteria) -> AppResult<i64>;

    /// Check if ranking ID exists
    async fn ranking_exists(&self, ranking_id: &str) -> AppResult<bool>;

    /// Find rankings by type
    async fn find_by_type(&self, ranking_type: &RankingType) -> AppResult<Vec<EPSRanking>>;

    /// Find rankings by period
    async fn find_by_period(&self, time_period: &RankingPeriod) -> AppResult<Vec<EPSRanking>>;
}
