use chrono::{DateTime, Utc};
use crate::domain::shared_kernel::aggregate_root::{AggregateRoot, AggregateBase};
use crate::domain::shared_kernel::domain_event::{DomainEvent, EventMetadata};
use crate::domain::trading_analytics::value_objects::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// EPS Ranking Aggregate Root
/// Represents rankings and comparisons across multiple stocks based on EPS metrics
#[derive(Debug, Clone)]
pub struct EPSRanking {
    ranking_id: String,
    ranking_type: RankingType,
    time_period: RankingPeriod,
    sector_filter: Option<SectorCategory>,
    country_filter: Option<Country>,
    entries: BTreeMap<u32, RankingEntry>, // rank -> entry
    total_entries: u32,
    last_updated: DateTime<Utc>,
    statistics: RankingStatistics,
    base: AggregateBase,
}

impl EPSRanking {
    /// Create new EPS ranking
    pub fn new(
        ranking_type: RankingType,
        time_period: RankingPeriod,
        sector_filter: Option<SectorCategory>,
        country_filter: Option<Country>,
    ) -> Self {
        let ranking_id = Self::generate_ranking_id(&ranking_type, &time_period, &sector_filter, &country_filter);
        
        let mut ranking = Self {
            ranking_id: ranking_id.clone(),
            ranking_type: ranking_type.clone(),
            time_period,
            sector_filter: sector_filter.clone(),
            country_filter: country_filter.clone(),
            entries: BTreeMap::new(),
            total_entries: 0,
            last_updated: Utc::now(),
            statistics: RankingStatistics::default(),
            base: AggregateBase::new(),
        };

        // Publish creation event
        ranking.base.add_event(Box::new(EPSRankingCreated::new(
            ranking_id,
            ranking.base.version,
            ranking_type,
            time_period,
            sector_filter,
            country_filter,
        )));

        ranking
    }

    /// Add or update stock entry in the ranking
    pub fn add_entry(
        &mut self,
        symbol: StockSymbol,
        company_name: String,
        eps_value: EPSValue,
        growth_factor: GrowthFactor,
        sector: MarketSector,
        country: Country,
    ) -> Result<u32, String> {
        // Validate filters
        if let Some(ref sector_filter) = self.sector_filter {
            if sector.category() != sector_filter {
                return Err(format!("Stock sector {} does not match ranking filter {:?}", sector.category().as_str(), sector_filter));
            }
        }

        if let Some(ref country_filter) = self.country_filter {
            if &country != country_filter {
                return Err(format!("Stock country {} does not match ranking filter {}", country.name(), country_filter.name()));
            }
        }

        let entry = RankingEntry {
            symbol: symbol.clone(),
            company_name,
            eps_value,
            growth_factor,
            sector,
            country,
            score: self.calculate_entry_score(&eps_value, &growth_factor),
            added_at: Utc::now(),
        };

        // Remove existing entry if updating
        if let Some(existing_rank) = self.find_entry_rank(&symbol) {
            self.entries.remove(&existing_rank);
        }

        // Find correct position based on score
        let rank = self.calculate_rank(&entry);
        
        // Shift existing entries down if necessary
        self.shift_entries_from_rank(rank);
        
        // Insert new entry
        self.entries.insert(rank, entry);
        self.total_entries = self.entries.len() as u32;
        self.last_updated = Utc::now();

        // Recalculate statistics
        self.update_statistics();

        // Publish event
        self.base.add_event(Box::new(StockAddedToRanking::new(
            self.ranking_id.clone(),
            self.base.version,
            symbol.clone(),
            rank,
            self.total_entries,
        )));

        Ok(rank)
    }

    /// Remove stock from ranking
    pub fn remove_entry(&mut self, symbol: &StockSymbol) -> Result<(), String> {
        let rank = self.find_entry_rank(symbol)
            .ok_or_else(|| format!("Stock {} not found in ranking", symbol.as_str()))?;

        self.entries.remove(&rank);
        
        // Shift remaining entries up
        self.compact_rankings();
        
        self.total_entries = self.entries.len() as u32;
        self.last_updated = Utc::now();
        
        // Update statistics
        self.update_statistics();

        // Publish event
        self.base.add_event(Box::new(StockRemovedFromRanking::new(
            self.ranking_id.clone(),
            self.base.version,
            symbol.clone(),
            rank,
        )));

        Ok(())
    }

    /// Get top N entries
    pub fn top_entries(&self, limit: usize) -> Vec<(u32, &RankingEntry)> {
        self.entries.iter()
            .take(limit)
            .map(|(rank, entry)| (*rank, entry))
            .collect()
    }

    /// Get entry by rank
    pub fn get_entry_by_rank(&self, rank: u32) -> Option<&RankingEntry> {
        self.entries.get(&rank)
    }

    /// Get entry by symbol
    pub fn get_entry_by_symbol(&self, symbol: &StockSymbol) -> Option<(u32, &RankingEntry)> {
        self.entries.iter()
            .find(|(_, entry)| &entry.symbol == symbol)
            .map(|(rank, entry)| (*rank, entry))
    }

    /// Get entries within percentile range
    pub fn entries_in_percentile_range(&self, min_percentile: f64, max_percentile: f64) -> Vec<(u32, &RankingEntry)> {
        if self.total_entries == 0 {
            return vec![];
        }

        let min_rank = ((min_percentile / 100.0) * self.total_entries as f64).ceil() as u32;
        let max_rank = ((max_percentile / 100.0) * self.total_entries as f64).floor() as u32;

        self.entries.range(min_rank..=max_rank)
            .map(|(rank, entry)| (*rank, entry))
            .collect()
    }

    /// Calculate entry score based on ranking type
    fn calculate_entry_score(&self, eps_value: &EPSValue, growth_factor: &GrowthFactor) -> f64 {
        match self.ranking_type {
            RankingType::EPSValue => eps_value.value(),
            RankingType::EPSGrowth => growth_factor.percentage(),
            RankingType::Combined => {
                // Weighted combination: 60% EPS value, 40% growth
                let eps_score = eps_value.value().max(0.0) * 0.6;
                let growth_score = growth_factor.percentage() * 0.004; // Scale growth to similar range
                eps_score + growth_score
            }
        }
    }

    /// Calculate rank for new entry
    fn calculate_rank(&self, new_entry: &RankingEntry) -> u32 {
        let mut rank = 1;
        
        for (_, existing_entry) in &self.entries {
            if new_entry.score > existing_entry.score {
                break;
            }
            rank += 1;
        }
        
        rank
    }

    /// Find rank of existing entry by symbol
    fn find_entry_rank(&self, symbol: &StockSymbol) -> Option<u32> {
        self.entries.iter()
            .find(|(_, entry)| &entry.symbol == symbol)
            .map(|(rank, _)| *rank)
    }

    /// Shift entries from given rank downward
    fn shift_entries_from_rank(&mut self, from_rank: u32) {
        let entries_to_shift: Vec<_> = self.entries.range(from_rank..).map(|(rank, _)| *rank).collect();
        let mut shifted_entries = BTreeMap::new();

        for old_rank in entries_to_shift {
            if let Some(entry) = self.entries.remove(&old_rank) {
                shifted_entries.insert(old_rank + 1, entry);
            }
        }

        for (rank, entry) in shifted_entries {
            self.entries.insert(rank, entry);
        }
    }

    /// Compact rankings after removal
    fn compact_rankings(&mut self) {
        let entries: Vec<_> = self.entries.values().cloned().collect();
        self.entries.clear();

        for (index, entry) in entries.into_iter().enumerate() {
            self.entries.insert((index + 1) as u32, entry);
        }
    }

    /// Update ranking statistics
    fn update_statistics(&mut self) {
        if self.entries.is_empty() {
            self.statistics = RankingStatistics::default();
            return;
        }

        let scores: Vec<f64> = self.entries.values().map(|e| e.score).collect();
        let eps_values: Vec<f64> = self.entries.values().map(|e| e.eps_value.value()).collect();
        let growth_values: Vec<f64> = self.entries.values().map(|e| e.growth_factor.percentage()).collect();

        self.statistics = RankingStatistics {
            total_entries: self.total_entries,
            avg_eps: eps_values.iter().sum::<f64>() / eps_values.len() as f64,
            median_eps: Self::calculate_median(&mut eps_values.clone()),
            avg_growth: growth_values.iter().sum::<f64>() / growth_values.len() as f64,
            median_growth: Self::calculate_median(&mut growth_values.clone()),
            top_score: scores.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            bottom_score: scores.iter().copied().fold(f64::INFINITY, f64::min),
        };
    }

    fn calculate_median(values: &mut Vec<f64>) -> f64 {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = values.len();
        if len % 2 == 0 {
            (values[len / 2 - 1] + values[len / 2]) / 2.0
        } else {
            values[len / 2]
        }
    }

    /// Generate unique ranking ID
    fn generate_ranking_id(
        ranking_type: &RankingType,
        period: &RankingPeriod,
        sector: &Option<SectorCategory>,
        country: &Option<Country>,
    ) -> String {
        let mut parts = vec![
            ranking_type.as_str().to_string(),
            period.as_str().to_string(),
        ];

        if let Some(s) = sector {
            parts.push(format!("sector_{}", s.as_str()));
        }

        if let Some(c) = country {
            parts.push(format!("country_{}", c.code()));
        }

        parts.join("_")
    }

    // Getters
    pub fn ranking_id(&self) -> &str { &self.ranking_id }
    pub fn ranking_type(&self) -> &RankingType { &self.ranking_type }
    pub fn time_period(&self) -> RankingPeriod { self.time_period }
    pub fn sector_filter(&self) -> Option<&SectorCategory> { self.sector_filter.as_ref() }
    pub fn country_filter(&self) -> Option<&Country> { self.country_filter.as_ref() }
    pub fn total_entries(&self) -> u32 { self.total_entries }
    pub fn last_updated(&self) -> DateTime<Utc> { self.last_updated }
    pub fn statistics(&self) -> &RankingStatistics { &self.statistics }
}

impl AggregateRoot for EPSRanking {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.ranking_id
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

/// Entry in the EPS ranking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankingEntry {
    pub symbol: StockSymbol,
    pub company_name: String,
    pub eps_value: EPSValue,
    pub growth_factor: GrowthFactor,
    pub sector: MarketSector,
    pub country: Country,
    pub score: f64,
    pub added_at: DateTime<Utc>,
}

/// Types of EPS rankings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RankingType {
    EPSValue,
    EPSGrowth,
    Combined,
}

impl RankingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RankingType::EPSValue => "eps_value",
            RankingType::EPSGrowth => "eps_growth",
            RankingType::Combined => "combined",
        }
    }
}

/// Time periods for rankings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RankingPeriod {
    Quarterly,
    Yearly,
    Trailing12Months,
}

impl RankingPeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            RankingPeriod::Quarterly => "quarterly",
            RankingPeriod::Yearly => "yearly",
            RankingPeriod::Trailing12Months => "ttm",
        }
    }
}

/// Statistical summary of ranking data
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RankingStatistics {
    pub total_entries: u32,
    pub avg_eps: f64,
    pub median_eps: f64,
    pub avg_growth: f64,
    pub median_growth: f64,
    pub top_score: f64,
    pub bottom_score: f64,
}

// Domain Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPSRankingCreated {
    pub metadata: EventMetadata,
    pub ranking_type: RankingType,
    pub time_period: RankingPeriod,
    pub sector_filter: Option<SectorCategory>,
    pub country_filter: Option<Country>,
}

impl EPSRankingCreated {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        ranking_type: RankingType,
        time_period: RankingPeriod,
        sector_filter: Option<SectorCategory>,
        country_filter: Option<Country>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            ranking_type,
            time_period,
            sector_filter,
            country_filter,
        }
    }
}

impl DomainEvent for EPSRankingCreated {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "EPSRankingCreated"
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
pub struct StockAddedToRanking {
    pub metadata: EventMetadata,
    pub symbol: StockSymbol,
    pub rank: u32,
    pub total_entries: u32,
}

impl StockAddedToRanking {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        symbol: StockSymbol,
        rank: u32,
        total_entries: u32,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            symbol,
            rank,
            total_entries,
        }
    }
}

impl DomainEvent for StockAddedToRanking {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "StockAddedToRanking"
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
pub struct StockRemovedFromRanking {
    pub metadata: EventMetadata,
    pub symbol: StockSymbol,
    pub former_rank: u32,
}

impl StockRemovedFromRanking {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        symbol: StockSymbol,
        former_rank: u32,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            symbol,
            former_rank,
        }
    }
}

impl DomainEvent for StockRemovedFromRanking {
    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn event_type(&self) -> &'static str {
        "StockRemovedFromRanking"
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