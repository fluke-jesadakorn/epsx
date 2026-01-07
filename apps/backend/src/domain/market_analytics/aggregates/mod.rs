pub mod stock_analysis;
pub mod eps_ranking;

// Re-export aggregates and their types
pub use stock_analysis::{
    StockAnalysis, AnalysisScore, Ranking, RankingCategory,
    InvestmentRecommendation, StockAnalysisCreated, StockAnalysisUpdated, StockRankingUpdated
};

pub use eps_ranking::{
    EPSRanking, RankingEntry, RankingType, RankingPeriod, RankingStatistics,
    EPSRankingCreated, StockAddedToRanking, StockRemovedFromRanking
};