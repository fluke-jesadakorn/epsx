// Trading Analytics Query Models

pub mod get_eps_ranking;
pub mod get_growth_leaders;
pub mod get_stock_analysis;
pub mod get_stock_statistics;
pub mod get_stocks_by_sector;
pub mod get_top_performers;
pub mod list_eps_rankings;
pub mod list_stock_analyses;

// New queries for web layer migration
pub mod get_admin_modules;
pub mod get_admin_timeseries;
pub mod get_cache_stats;
pub mod get_cached_rankings;
pub mod get_portfolio_rankings;
pub mod get_sectors_by_country;
pub mod get_stock_ranking_assignments;
pub mod get_system_metrics;

// Re-export queries and responses
pub use get_eps_ranking::{
    GetEPSRankingQuery, GetEPSRankingResponse, RankingEntryDTO, RankingStatisticsDTO,
};
pub use get_growth_leaders::{GetGrowthLeadersQuery, GetGrowthLeadersResponse};
pub use get_stock_analysis::{GetStockAnalysisQuery, GetStockAnalysisResponse, RankingSummary};
pub use get_stock_statistics::{GetStockStatisticsQuery, GetStockStatisticsResponse};
pub use get_stocks_by_sector::{GetStocksBySectorQuery, GetStocksBySectorResponse};
pub use get_top_performers::{GetTopPerformersQuery, GetTopPerformersResponse};
pub use list_eps_rankings::{EPSRankingSummary, ListEPSRankingsQuery, ListEPSRankingsResponse};
pub use list_stock_analyses::{
    ListStockAnalysesQuery, ListStockAnalysesResponse, StockAnalysisSummary,
};

// Re-export new queries
pub use get_admin_modules::{GetAdminModulesQuery, GetAdminModulesResponse, ModuleStatus};
pub use get_admin_timeseries::*;
pub use get_cache_stats::{GetCacheStatsQuery, GetCacheStatsResponse};
pub use get_cached_rankings::{GetCachedRankingsQuery, GetCachedRankingsResponse};
pub use get_portfolio_rankings::{GetPortfolioRankingsQuery, GetPortfolioRankingsResponse};
pub use get_sectors_by_country::{GetSectorsByCountryQuery, GetSectorsByCountryResponse};
pub use get_stock_ranking_assignments::{
    GetStockRankingAssignmentsQuery, GetStockRankingAssignmentsResponse, StockRankingAssignment,
};
pub use get_system_metrics::*;
