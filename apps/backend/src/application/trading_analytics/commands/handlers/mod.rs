// Trading Analytics Command Handlers

pub mod create_stock_analysis_handler;
pub mod update_stock_analysis_handler;
pub mod delete_stock_analysis_handler;
pub mod create_eps_ranking_handler;
pub mod add_stock_to_ranking_handler;

// New handlers for web layer migration
pub mod refresh_cache_handler;
pub mod sync_eps_data_handler;

pub use create_stock_analysis_handler::CreateStockAnalysisCommandHandler;
pub use update_stock_analysis_handler::UpdateStockAnalysisCommandHandler;
pub use delete_stock_analysis_handler::DeleteStockAnalysisCommandHandler;
pub use create_eps_ranking_handler::CreateEPSRankingCommandHandler;
pub use add_stock_to_ranking_handler::AddStockToRankingCommandHandler;

// Re-export new handlers
pub use refresh_cache_handler::RefreshCacheCommandHandler;
pub use sync_eps_data_handler::SyncEPSDataCommandHandler;
