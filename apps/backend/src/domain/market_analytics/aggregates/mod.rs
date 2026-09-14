pub mod eps_ranking;
pub mod stock_analysis;

// Re-export aggregates and their types
pub use stock_analysis::{
    AnalysisScore, InvestmentRecommendation, Ranking, RankingCategory, StockAnalysis,
    StockAnalysisCreated, StockAnalysisUpdated, StockRankingUpdated,
};

pub use eps_ranking::{
    EPSRanking, EPSRankingCreated, RankingEntry, RankingPeriod, RankingStatistics, RankingType,
    StockAddedToRanking, StockRemovedFromRanking,
};
