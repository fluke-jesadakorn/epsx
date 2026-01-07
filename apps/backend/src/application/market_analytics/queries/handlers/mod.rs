// Trading Analytics Query Handlers

pub mod get_stock_analysis_handler;
pub mod list_stock_analyses_handler;
pub mod get_eps_ranking_handler;
pub mod list_eps_rankings_handler;
pub mod get_top_performers_handler;
pub mod get_stocks_by_sector_handler;
pub mod get_growth_leaders_handler;
pub mod get_stock_statistics_handler;

// New handlers for web layer migration
pub mod get_portfolio_rankings_handler;
pub mod get_system_metrics_handler;
pub mod get_admin_timeseries_handler;
pub mod get_admin_modules_handler;

pub use get_stock_analysis_handler::GetStockAnalysisQueryHandler;
pub use list_stock_analyses_handler::ListStockAnalysesQueryHandler;
pub use get_eps_ranking_handler::GetEPSRankingQueryHandler;
pub use list_eps_rankings_handler::ListEPSRankingsQueryHandler;
pub use get_top_performers_handler::GetTopPerformersQueryHandler;
pub use get_stocks_by_sector_handler::GetStocksBySectorQueryHandler;
pub use get_growth_leaders_handler::GetGrowthLeadersQueryHandler;
pub use get_stock_statistics_handler::GetStockStatisticsQueryHandler;

// Re-export new handlers
pub use get_portfolio_rankings_handler::GetPortfolioRankingsQueryHandler;
pub use get_system_metrics_handler::GetSystemMetricsQueryHandler;
pub use get_admin_timeseries_handler::GetAdminTimeSeriesQueryHandler;
pub use get_admin_modules_handler::GetAdminModulesQueryHandler;
