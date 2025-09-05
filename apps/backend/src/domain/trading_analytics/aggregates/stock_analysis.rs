use crate::domain::shared_kernel::aggregate_root::{AggregateRoot, AggregateBase};
use crate::domain::shared_kernel::domain_event::{DomainEvent, EventMetadata};
use crate::domain::trading_analytics::value_objects::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Stock Analysis Aggregate Root
/// Represents comprehensive analytical data for a single stock
#[derive(Debug, Clone)]
pub struct StockAnalysis {
    id: String,
    symbol: StockSymbol,
    company_name: String,
    current_eps: EPSValue,
    previous_eps: EPSValue,
    eps_growth: GrowthFactor,
    sector: MarketSector,
    country: Country,
    analysis_score: AnalysisScore,
    rankings: HashMap<RankingCategory, Ranking>,
    last_updated: DateTime<Utc>,
    base: AggregateBase,
}

impl StockAnalysis {
    /// Create new stock analysis
    pub fn new(
        symbol: StockSymbol,
        company_name: String,
        current_eps: EPSValue,
        previous_eps: EPSValue,
        sector: MarketSector,
        country: Country,
    ) -> Result<Self, String> {
        if company_name.trim().is_empty() {
            return Err("Company name cannot be empty".to_string());
        }

        if company_name.len() > 200 {
            return Err("Company name cannot exceed 200 characters".to_string());
        }

        // Calculate EPS growth
        let eps_growth = match previous_eps.percentage_change_to(current_eps) {
            Some(change) => GrowthFactor::new(change)?,
            None => GrowthFactor::new(0.0)?, // Default to zero growth if previous EPS was zero
        };

        // Calculate initial analysis score
        let analysis_score = Self::calculate_analysis_score(&current_eps, &eps_growth, &sector, &country);

        let id = symbol.to_string();
        let mut stock_analysis = Self {
            id: id.clone(),
            symbol: symbol.clone(),
            company_name,
            current_eps,
            previous_eps,
            eps_growth,
            sector,
            country,
            analysis_score,
            rankings: HashMap::new(),
            last_updated: Utc::now(),
            base: AggregateBase::new(),
        };

        // Publish domain event
        stock_analysis.base.add_event(Box::new(StockAnalysisCreated::new(
            id,
            stock_analysis.base.version,
            symbol,
            stock_analysis.company_name.clone(),
            stock_analysis.analysis_score.clone(),
        )));

        Ok(stock_analysis)
    }

    /// Update EPS data and recalculate metrics
    pub fn update_eps(&mut self, current_eps: EPSValue, previous_eps: EPSValue) -> Result<(), String> {
        let old_score = self.analysis_score.clone();
        
        self.previous_eps = previous_eps;
        self.current_eps = current_eps;
        
        // Recalculate growth
        self.eps_growth = match previous_eps.percentage_change_to(current_eps) {
            Some(change) => GrowthFactor::new(change)?,
            None => GrowthFactor::new(0.0)?,
        };

        // Recalculate analysis score
        self.analysis_score = Self::calculate_analysis_score(&current_eps, &self.eps_growth, &self.sector, &self.country);
        self.last_updated = Utc::now();

        // Publish event if score changed significantly
        if (self.analysis_score.overall_score as i32 - old_score.overall_score as i32).abs() >= 5 {
            self.base.add_event(Box::new(StockAnalysisUpdated::new(
                self.id.clone(),
                self.base.version,
                self.symbol.clone(),
                old_score.overall_score,
                self.analysis_score.overall_score,
            )));
        }

        Ok(())
    }

    /// Set ranking for a specific category
    pub fn set_ranking(&mut self, category: RankingCategory, rank: u32, total_stocks: u32, percentile: f64) -> Result<(), String> {
        if rank == 0 || total_stocks == 0 {
            return Err("Rank and total stocks must be greater than zero".to_string());
        }

        if rank > total_stocks {
            return Err("Rank cannot be greater than total stocks".to_string());
        }

        if !(0.0..=100.0).contains(&percentile) {
            return Err("Percentile must be between 0 and 100".to_string());
        }

        let ranking = Ranking {
            category: category.clone(),
            rank,
            total_stocks,
            percentile,
            updated_at: Utc::now(),
        };

        self.rankings.insert(category.clone(), ranking);
        self.last_updated = Utc::now();

        // Publish ranking update event
        self.base.add_event(Box::new(StockRankingUpdated::new(
            self.id.clone(),
            self.base.version,
            self.symbol.clone(),
            category,
            rank,
            percentile,
        )));

        Ok(())
    }

    /// Get ranking for a specific category
    pub fn get_ranking(&self, category: &RankingCategory) -> Option<&Ranking> {
        self.rankings.get(category)
    }

    /// Calculate comprehensive analysis score
    fn calculate_analysis_score(eps: &EPSValue, growth: &GrowthFactor, sector: &MarketSector, country: &Country) -> AnalysisScore {
        let eps_score = match eps.quality_rating() {
            EPSQuality::Excellent => 25,
            EPSQuality::Good => 20,
            EPSQuality::Fair => 15,
            EPSQuality::Poor => 10,
            EPSQuality::Negative => 0,
        };

        let growth_score = (growth.investment_score() as f64 * 0.25) as u8;

        let sector_score = match sector.growth_potential() {
            GrowthPotential::High => 20,
            GrowthPotential::Medium => 15,
            GrowthPotential::Low => 10,
            GrowthPotential::Unknown => 5,
        };

        let country_score = if country.is_developed_market() { 15 } else { 10 };

        let volatility_adjustment = match sector.typical_volatility() {
            VolatilityLevel::High => -5i8,
            VolatilityLevel::Medium => 0i8,
            VolatilityLevel::Low => 5i8,
            VolatilityLevel::Unknown => 0i8,
        };

        let overall_score = ((eps_score + growth_score + sector_score + country_score) as i8 + volatility_adjustment).clamp(0, 100) as u8;

        AnalysisScore {
            overall_score,
            eps_score,
            growth_score,
            sector_score,
            country_score,
            volatility_adjustment,
            last_calculated: Utc::now(),
        }
    }

    /// Check if analysis needs updating (based on age)
    pub fn needs_update(&self, max_age_hours: i64) -> bool {
        let age = Utc::now().signed_duration_since(self.last_updated);
        age.num_hours() > max_age_hours
    }

    /// Get investment recommendation based on analysis
    pub fn investment_recommendation(&self) -> InvestmentRecommendation {
        match self.analysis_score.overall_score {
            90..=100 => InvestmentRecommendation::StrongBuy,
            80..=89 => InvestmentRecommendation::Buy,
            70..=79 => InvestmentRecommendation::WeakBuy,
            60..=69 => InvestmentRecommendation::Hold,
            50..=59 => InvestmentRecommendation::WeakSell,
            40..=49 => InvestmentRecommendation::Sell,
            _ => InvestmentRecommendation::StrongSell,
        }
    }

    // Getters
    pub fn symbol(&self) -> &StockSymbol { &self.symbol }
    pub fn company_name(&self) -> &str { &self.company_name }
    pub fn current_eps(&self) -> EPSValue { self.current_eps }
    pub fn previous_eps(&self) -> EPSValue { self.previous_eps }
    pub fn eps_growth(&self) -> GrowthFactor { self.eps_growth }
    pub fn sector(&self) -> &MarketSector { &self.sector }
    pub fn country(&self) -> &Country { &self.country }
    pub fn analysis_score(&self) -> &AnalysisScore { &self.analysis_score }
    pub fn rankings(&self) -> &HashMap<RankingCategory, Ranking> { &self.rankings }
    pub fn last_updated(&self) -> DateTime<Utc> { self.last_updated }
}

impl AggregateRoot for StockAnalysis {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.base.version
    }

    fn increment_version(&mut self) {
        self.base.increment_version();
    }

    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.base.events
    }

    fn mark_events_as_committed(&mut self) {
        self.base.clear_events();
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }

    fn touch(&mut self) {
        self.base.touch();
    }
}

/// Analysis Score breakdown
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisScore {
    pub overall_score: u8,
    pub eps_score: u8,
    pub growth_score: u8,
    pub sector_score: u8,
    pub country_score: u8,
    pub volatility_adjustment: i8,
    pub last_calculated: DateTime<Utc>,
}

/// Ranking information for different categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ranking {
    pub category: RankingCategory,
    pub rank: u32,
    pub total_stocks: u32,
    pub percentile: f64,
    pub updated_at: DateTime<Utc>,
}

/// Categories for stock rankings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RankingCategory {
    EPSGrowth,
    EPSValue,
    OverallScore,
    SectorRanking,
    MarketCapGrowth,
}

impl RankingCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            RankingCategory::EPSGrowth => "eps_growth",
            RankingCategory::EPSValue => "eps_value", 
            RankingCategory::OverallScore => "overall_score",
            RankingCategory::SectorRanking => "sector_ranking",
            RankingCategory::MarketCapGrowth => "market_cap_growth",
        }
    }
}

/// Investment recommendation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestmentRecommendation {
    StrongBuy,
    Buy,
    WeakBuy,
    Hold,
    WeakSell,
    Sell,
    StrongSell,
}

impl InvestmentRecommendation {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvestmentRecommendation::StrongBuy => "strong_buy",
            InvestmentRecommendation::Buy => "buy",
            InvestmentRecommendation::WeakBuy => "weak_buy",
            InvestmentRecommendation::Hold => "hold",
            InvestmentRecommendation::WeakSell => "weak_sell",
            InvestmentRecommendation::Sell => "sell",
            InvestmentRecommendation::StrongSell => "strong_sell",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            InvestmentRecommendation::StrongBuy => "Strong Buy - Excellent opportunity",
            InvestmentRecommendation::Buy => "Buy - Good investment potential",
            InvestmentRecommendation::WeakBuy => "Weak Buy - Modest upside potential",
            InvestmentRecommendation::Hold => "Hold - Maintain current position",
            InvestmentRecommendation::WeakSell => "Weak Sell - Consider reducing position",
            InvestmentRecommendation::Sell => "Sell - Significant downside risk",
            InvestmentRecommendation::StrongSell => "Strong Sell - High risk of losses",
        }
    }
}

// Domain Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAnalysisCreated {
    pub metadata: EventMetadata,
    pub symbol: StockSymbol,
    pub company_name: String,
    pub analysis_score: AnalysisScore,
}

impl StockAnalysisCreated {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        symbol: StockSymbol,
        company_name: String,
        analysis_score: AnalysisScore,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            symbol,
            company_name,
            analysis_score,
        }
    }
}

impl DomainEvent for StockAnalysisCreated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "StockAnalysisCreated"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAnalysisUpdated {
    pub metadata: EventMetadata,
    pub symbol: StockSymbol,
    pub old_score: u8,
    pub new_score: u8,
}

impl StockAnalysisUpdated {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        symbol: StockSymbol,
        old_score: u8,
        new_score: u8,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            symbol,
            old_score,
            new_score,
        }
    }
}

impl DomainEvent for StockAnalysisUpdated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "StockAnalysisUpdated"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockRankingUpdated {
    pub metadata: EventMetadata,
    pub symbol: StockSymbol,
    pub category: RankingCategory,
    pub rank: u32,
    pub percentile: f64,
}

impl StockRankingUpdated {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        symbol: StockSymbol,
        category: RankingCategory,
        rank: u32,
        percentile: f64,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            symbol,
            category,
            rank,
            percentile,
        }
    }
}

impl DomainEvent for StockRankingUpdated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "StockRankingUpdated"
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.metadata.occurred_at
    }

    fn aggregate_version(&self) -> u64 {
        self.metadata.aggregate_version
    }

    fn aggregate_id(&self) -> String {
        self.metadata.aggregate_id.clone()
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}