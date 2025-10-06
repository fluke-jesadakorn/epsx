// Trading Analytics Application Layer
// Commands and queries for stock analysis and EPS ranking operations

pub mod commands;
pub mod queries;
pub mod services;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos; // Request/Response DTOs

// Re-export command models
pub use commands::{
    CreateStockAnalysisCommand,
    CreateStockAnalysisResponse,
    UpdateStockAnalysisCommand,
    UpdateStockAnalysisResponse,
    DeleteStockAnalysisCommand,
    DeleteStockAnalysisResponse,
    CreateEPSRankingCommand,
    CreateEPSRankingResponse,
    AddStockToRankingCommand,
    AddStockToRankingResponse,
};

// Re-export command handlers
pub use commands::{
    CreateStockAnalysisCommandHandler,
    UpdateStockAnalysisCommandHandler,
    DeleteStockAnalysisCommandHandler,
    CreateEPSRankingCommandHandler,
    AddStockToRankingCommandHandler,
};

// Re-export query models
pub use queries::{
    GetStockAnalysisQuery,
    GetStockAnalysisResponse,
    ListStockAnalysesQuery,
    ListStockAnalysesResponse,
    GetEPSRankingQuery,
    GetEPSRankingResponse,
    ListEPSRankingsQuery,
    ListEPSRankingsResponse,
    GetTopPerformersQuery,
    GetTopPerformersResponse,
    GetStocksBySectorQuery,
    GetStocksBySectorResponse,
    GetGrowthLeadersQuery,
    GetGrowthLeadersResponse,
    GetStockStatisticsQuery,
    GetStockStatisticsResponse,
    StockAnalysisSummary,
    EPSRankingSummary,
    RankingEntryDTO,
    RankingStatisticsDTO,
};

// Re-export query handlers
pub use queries::{
    GetStockAnalysisQueryHandler,
    ListStockAnalysesQueryHandler,
    GetEPSRankingQueryHandler,
    ListEPSRankingsQueryHandler,
    GetTopPerformersQueryHandler,
    GetStocksBySectorQueryHandler,
    GetGrowthLeadersQueryHandler,
    GetStockStatisticsQueryHandler,
};
