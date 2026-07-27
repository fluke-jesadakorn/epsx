// Trading Analytics Application Layer
// Commands and queries for stock analysis and EPS ranking operations

pub mod commands;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos;
pub mod queries;
pub mod services; // Request/Response DTOs

// Re-export command models
pub use commands::{
    AddStockToRankingCommand, AddStockToRankingResponse, CreateEPSRankingCommand,
    CreateEPSRankingResponse, CreateStockAnalysisCommand, CreateStockAnalysisResponse,
    DeleteStockAnalysisCommand, DeleteStockAnalysisResponse, UpdateStockAnalysisCommand,
    UpdateStockAnalysisResponse,
};

// Re-export command handlers
pub use commands::{
    AddStockToRankingCommandHandler, CreateEPSRankingCommandHandler,
    CreateStockAnalysisCommandHandler, DeleteStockAnalysisCommandHandler,
    UpdateStockAnalysisCommandHandler,
};

// Re-export query models
pub use queries::{
    EPSRankingSummary, GetEPSRankingQuery, GetEPSRankingResponse, GetGrowthLeadersQuery,
    GetGrowthLeadersResponse, GetStockAnalysisQuery, GetStockAnalysisResponse,
    GetStockStatisticsQuery, GetStockStatisticsResponse, GetStocksBySectorQuery,
    GetStocksBySectorResponse, GetTopPerformersQuery, GetTopPerformersResponse,
    ListEPSRankingsQuery, ListEPSRankingsResponse, ListStockAnalysesQuery,
    ListStockAnalysesResponse, RankingEntryDTO, RankingStatisticsDTO, StockAnalysisSummary,
};

// Re-export query handlers
pub use queries::{
    GetEPSRankingQueryHandler, GetGrowthLeadersQueryHandler, GetStockAnalysisQueryHandler,
    GetStockStatisticsQueryHandler, GetStocksBySectorQueryHandler, GetTopPerformersQueryHandler,
    ListEPSRankingsQueryHandler, ListStockAnalysesQueryHandler,
};
