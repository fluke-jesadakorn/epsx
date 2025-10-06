// Trading Analytics Command Models

pub mod create_stock_analysis;
pub mod update_stock_analysis;
pub mod delete_stock_analysis;
pub mod create_eps_ranking;
pub mod add_stock_to_ranking;

// New commands for web layer migration
pub mod refresh_cache;
pub mod sync_eps_data;
pub mod extend_assignment;
pub mod revoke_assignment;

// Re-export commands and responses
pub use create_stock_analysis::{CreateStockAnalysisCommand, CreateStockAnalysisResponse};
pub use update_stock_analysis::{UpdateStockAnalysisCommand, UpdateStockAnalysisResponse};
pub use delete_stock_analysis::{DeleteStockAnalysisCommand, DeleteStockAnalysisResponse};
pub use create_eps_ranking::{CreateEPSRankingCommand, CreateEPSRankingResponse, RankingFilters};
pub use add_stock_to_ranking::{AddStockToRankingCommand, AddStockToRankingResponse};

// Re-export new commands
pub use refresh_cache::{RefreshCacheCommand, RefreshCacheResponse};
pub use sync_eps_data::{SyncEPSDataCommand, SyncEPSDataResponse};
pub use extend_assignment::{ExtendAssignmentCommand, ExtendAssignmentResponse};
pub use revoke_assignment::{RevokeAssignmentCommand, RevokeAssignmentResponse};
