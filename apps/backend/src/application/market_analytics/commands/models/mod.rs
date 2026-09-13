// Trading Analytics Command Models

pub mod add_stock_to_ranking;
pub mod create_eps_ranking;
pub mod create_stock_analysis;
pub mod delete_stock_analysis;
pub mod update_stock_analysis;

// New commands for web layer migration
pub mod extend_assignment;
pub mod refresh_cache;
pub mod revoke_assignment;
pub mod sync_eps_data;

// Re-export commands and responses
pub use add_stock_to_ranking::{AddStockToRankingCommand, AddStockToRankingResponse};
pub use create_eps_ranking::{CreateEPSRankingCommand, CreateEPSRankingResponse, RankingFilters};
pub use create_stock_analysis::{CreateStockAnalysisCommand, CreateStockAnalysisResponse};
pub use delete_stock_analysis::{DeleteStockAnalysisCommand, DeleteStockAnalysisResponse};
pub use update_stock_analysis::{UpdateStockAnalysisCommand, UpdateStockAnalysisResponse};

// Re-export new commands
pub use extend_assignment::{ExtendAssignmentCommand, ExtendAssignmentResponse};
pub use refresh_cache::{RefreshCacheCommand, RefreshCacheResponse};
pub use revoke_assignment::{RevokeAssignmentCommand, RevokeAssignmentResponse};
pub use sync_eps_data::{SyncEPSDataCommand, SyncEPSDataResponse};
