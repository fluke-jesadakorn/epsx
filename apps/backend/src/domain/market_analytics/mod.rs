/// Market Analytics Bounded Context
/// 
/// This bounded context handles all aspects of stock analysis, EPS tracking, rankings,
/// and investment recommendations. It provides comprehensive analytical capabilities
/// for the EPSX analytics platform.
/// 
/// ## Core Concepts
/// 
/// - **StockAnalysis**: Comprehensive analysis of individual stocks with EPS data,
///   growth metrics, sector analysis, and investment scoring
/// 
/// - **EPSRanking**: Comparative rankings across multiple stocks based on EPS metrics,
///   supporting filtering by sector and country
/// 
/// - **Value Objects**: Core analytics concepts like StockSymbol, EPSValue, GrowthFactor,
///   MarketSector, and Country provide type safety and business rules
/// 
/// ## Domain Events
/// 
/// The context publishes events for stock analysis updates, ranking changes,
/// and significant score movements to enable reactive processing
/// 
/// ## Integration
/// 
/// This bounded context integrates with:
/// - User Management (for permissions and access control)
/// - Notification (for alerting on significant changes)
/// - External data providers (for real-time stock data)
pub mod value_objects;
pub mod aggregates;
pub mod repository_ports;
pub mod domain_services;
// wave 9 (R5) — moved from `domain::shared_kernel::services::eps_ranking_service`
// to here because the EPS-ranking logic is used only by analytics (8 call
// sites) and was only sitting in the shared kernel by historical accident.
pub mod services;

// Public exports from value objects
pub use value_objects::{
    StockSymbol, EPSValue, EPSQuality, GrowthFactor, GrowthClassification, GrowthComparison,
    MarketSector, SectorCategory, GrowthPotential, VolatilityLevel,
    Country, MarketRegion, MarketComplexity, MarketCharacteristics,
    LiquidityLevel, RegulationLevel, TransparencyLevel
};

// Public exports from aggregates
pub use aggregates::{
    StockAnalysis, AnalysisScore, Ranking, RankingCategory, InvestmentRecommendation,
    StockAnalysisCreated, StockAnalysisUpdated, StockRankingUpdated,
    EPSRanking, RankingEntry, RankingType, RankingPeriod, RankingStatistics,
    EPSRankingCreated, StockAddedToRanking, StockRemovedFromRanking
};

// Public exports from repository ports
pub use repository_ports::{
    StockAnalysisRepositoryPort, StockAnalysisSearchCriteria, StockAnalysisStatistics,
    EPSRankingRepositoryPort, EPSRankingSearchCriteria
};