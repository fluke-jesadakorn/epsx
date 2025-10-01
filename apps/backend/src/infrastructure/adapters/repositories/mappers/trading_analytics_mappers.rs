/// Mappers for Trading Analytics domain
/// Convert between legacy EPS ranking structures and DDD Trading Analytics aggregates

use chrono::Utc;
use tracing::{debug, warn};

use crate::domain::trading_analytics::aggregates::eps_ranking::{EPSRanking as DDDEPSRanking, RankingEntry, RankingType, RankingPeriod, RankingStatistics};
use crate::domain::trading_analytics::aggregates::stock_analysis::StockAnalysis;
use crate::domain::trading_analytics::value_objects::*;
use crate::domain::shared_kernel::entities::eps_growth::{EPSRanking as LegacyEPSRanking, EPSGrowthData};
use crate::domain::shared_kernel::entities::stock::{Stock as LegacyStock};

/// Mapper for converting between legacy and DDD EPS ranking structures
pub struct EPSRankingMapper;

impl EPSRankingMapper {
    /// Convert legacy EPSRanking to DDD RankingEntry
    pub fn legacy_to_ddd_entry(legacy: &LegacyEPSRanking) -> Result<RankingEntry, String> {
        debug!("Converting legacy EPS ranking to DDD entry: {}", legacy.symbol);

        let symbol = StockSymbol::new(legacy.symbol.clone())
            .map_err(|e| format!("Invalid symbol '{}': {}", legacy.symbol, e))?;

        let eps_value = EPSValue::new(legacy.current_eps.unwrap_or(0.0))
            .map_err(|e| format!("Invalid EPS value: {}", e))?;

        let growth_factor = GrowthFactor::new(legacy.growth_factor.unwrap_or(0.0))
            .map_err(|e| format!("Invalid growth factor: {}", e))?;

        let sector = MarketSector::new(legacy.sector.clone())
            .map_err(|e| format!("Invalid sector '{}': {}", legacy.sector, e))?;

        let country = Country::new("US".to_string()) // Default country as field not available
            .map_err(|e| format!("Invalid default country: {}", e))?;

        Ok(RankingEntry {
            symbol,
            company_name: legacy.name.clone(),
            eps_value,
            growth_factor,
            sector,
            country,
            score: 0.0, // Will be calculated by DDD aggregate
            added_at: Utc::now(),
        })
    }

    /// Convert DDD RankingEntry to legacy EPSRanking
    pub fn ddd_entry_to_legacy(entry: &RankingEntry, rank: u32) -> LegacyEPSRanking {
        debug!("Converting DDD entry to legacy EPS ranking: {}", entry.symbol.as_str());

        LegacyEPSRanking {
            symbol: entry.symbol.as_str().to_string(),
            name: entry.company_name.clone(),
            country: entry.country.name().to_string(),
            sector: entry.sector.name().to_string(),
            exchange: "NASDAQ".to_string(), // Default exchange
            current_eps: Some(entry.eps_value.value()),
            growth_factor: Some(entry.growth_factor.percentage()),
            price_current: None, // Not available in DDD model
            market_cap: None,    // Not available in DDD model
            volume: None,        // Not available in DDD model
            ranking_position: Some(rank as i32),
            quarterly_data: None,
            next_earnings_date: None,
            last_earnings_date: None,
        }
    }

    /// Convert DDD RankingStatistics to legacy format
    pub fn ddd_stats_to_legacy(stats: &RankingStatistics) -> crate::domain::shared_kernel::entities::eps_growth::EPSPagination {
        // Since legacy EPSPagination is actually pagination, not statistics,
        // we create a minimal pagination structure
        crate::domain::shared_kernel::entities::eps_growth::EPSPagination::new(1, stats.total_entries as i32, stats.total_entries as i64)
    }

    /// Convert legacy EPSGrowthData to DDD RankingEntry
    pub fn legacy_growth_data_to_ddd_entry(growth_data: &EPSGrowthData) -> Result<RankingEntry, String> {
        debug!("Converting legacy EPS growth data to DDD entry: {}", growth_data.symbol);

        let symbol = StockSymbol::new(growth_data.symbol.clone())
            .map_err(|e| format!("Invalid symbol '{}': {}", growth_data.symbol, e))?;

        let eps_value = EPSValue::new(growth_data.current_eps.unwrap_or(0.0))
            .map_err(|e| format!("Invalid EPS value: {}", e))?;

        let growth_factor = GrowthFactor::new(growth_data.growth_factor.unwrap_or(0.0))
            .map_err(|e| format!("Invalid growth factor: {}", e))?;

        let sector = MarketSector::new("Technology".to_string())  // Default sector
            .map_err(|e| format!("Invalid sector: {}", e))?;

        let country = Country::new("US".to_string()) // Default country as field not available
            .map_err(|e| format!("Invalid default country: {}", e))?;

        Ok(RankingEntry {
            symbol,
            company_name: growth_data.name.clone(),
            eps_value,
            growth_factor,
            sector,
            country,
            score: 0.0,  // Default score
            added_at: growth_data.created_at.unwrap_or_else(|| Utc::now()),
        })
    }

    /// Build DDD EPSRanking from collection of legacy rankings
    pub fn build_ddd_ranking_from_legacy_collection(
        legacy_rankings: &[LegacyEPSRanking],
        ranking_type: RankingType,
        sector_filter: Option<SectorCategory>,
        country_filter: Option<Country>,
    ) -> Result<DDDEPSRanking, String> {
        debug!("Building DDD ranking from {} legacy rankings", legacy_rankings.len());

        // Create new DDD ranking
        let mut ddd_ranking = DDDEPSRanking::new(
            ranking_type,
            RankingPeriod::Quarterly, // Default to quarterly
            sector_filter,
            country_filter,
        );

        // Add each legacy ranking as an entry
        for legacy_ranking in legacy_rankings {
            match Self::legacy_to_ddd_entry(legacy_ranking) {
                Ok(entry) => {
                    // Add entry to DDD ranking
                    match ddd_ranking.add_entry(
                        entry.symbol.clone(),
                        entry.company_name.clone(),
                        entry.eps_value,
                        entry.growth_factor,
                        entry.sector,
                        entry.country,
                    ) {
                        Ok(_rank) => {
                            debug!("Added {} to DDD ranking", entry.symbol.as_str());
                        }
                        Err(e) => {
                            warn!("Failed to add {} to DDD ranking: {}", entry.symbol.as_str(), e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to convert legacy ranking to DDD entry: {}", e);
                }
            }
        }

        debug!("Built DDD ranking with {} entries", ddd_ranking.total_entries());
        Ok(ddd_ranking)
    }
}

/// Mapper for converting between legacy and DDD Stock Analysis structures
pub struct StockAnalysisMapper;

impl StockAnalysisMapper {
    /// Convert legacy Stock to DDD StockAnalysis
    pub fn legacy_to_ddd_stock(legacy_stock: &LegacyStock) -> Result<StockAnalysis, String> {
        debug!("Converting legacy stock to DDD: {}", legacy_stock.symbol);

        let symbol = StockSymbol::new(legacy_stock.symbol.clone())
            .map_err(|e| format!("Invalid symbol: {}", e))?;

        // Create basic EPS values (unknown from legacy stock)
        let current_eps = EPSValue::new(0.0)
            .map_err(|e| format!("Invalid current EPS: {}", e))?;
        let previous_eps = EPSValue::new(0.0)
            .map_err(|e| format!("Invalid previous EPS: {}", e))?;

        // Create basic stock analysis (sector and country unknown from legacy stock)
        let stock_analysis = StockAnalysis::new(
            symbol,
            "Unknown Company".to_string(),
            current_eps,
            previous_eps,
            MarketSector::new("Unknown".to_string()).unwrap(),
            Country::new("unknown".to_string()).unwrap(),
        );

        stock_analysis
    }

    /// Convert legacy EPSRanking to DDD StockAnalysis
    pub fn legacy_ranking_to_ddd_stock(legacy_ranking: &LegacyEPSRanking) -> Result<StockAnalysis, String> {
        debug!("Converting legacy EPS ranking to DDD stock analysis: {}", legacy_ranking.symbol);

        let symbol = StockSymbol::new(legacy_ranking.symbol.clone())
            .map_err(|e| format!("Invalid symbol: {}", e))?;

        // Use EPS values from legacy ranking
        let current_eps = EPSValue::new(legacy_ranking.current_eps.unwrap_or(0.0))
            .map_err(|e| format!("Invalid current EPS: {}", e))?;
            
        // Calculate previous EPS from growth factor
        let growth_rate = legacy_ranking.growth_factor.unwrap_or(0.0);
        let previous_eps_value = if growth_rate != 0.0 {
            current_eps.value() / (1.0 + growth_rate / 100.0)
        } else {
            current_eps.value()
        };
        let previous_eps = EPSValue::new(previous_eps_value.max(0.0))
            .map_err(|e| format!("Invalid previous EPS: {}", e))?;

        let sector = MarketSector::new(legacy_ranking.sector.clone())
            .map_err(|e| format!("Invalid sector: {}", e))?;

        let country = Country::new("US".to_string()) // Default country as field not available
            .map_err(|e| format!("Invalid default country: {}", e))?;

        // Create stock analysis
        let stock_analysis = StockAnalysis::new(
            symbol,
            legacy_ranking.name.clone(),
            current_eps,
            previous_eps,
            sector,
            country,
        );

        stock_analysis
    }

    /// Convert DDD StockAnalysis back to legacy format (for API compatibility)
    pub fn ddd_stock_to_legacy_ranking(stock_analysis: &StockAnalysis, rank: Option<u32>) -> LegacyEPSRanking {
        debug!("Converting DDD stock analysis to legacy ranking: {}", stock_analysis.symbol().as_str());

        // Calculate growth rate from current and previous EPS
        let growth_rate = if stock_analysis.previous_eps().value() > 0.0 {
            ((stock_analysis.current_eps().value() - stock_analysis.previous_eps().value()) 
                / stock_analysis.previous_eps().value()) * 100.0
        } else {
            0.0
        };

        LegacyEPSRanking {
            symbol: stock_analysis.symbol().as_str().to_string(),
            name: stock_analysis.company_name().to_string(),
            country: stock_analysis.country().name().to_string(),
            sector: stock_analysis.sector().name().to_string(),
            exchange: "NASDAQ".to_string(), // Default exchange
            current_eps: Some(stock_analysis.current_eps().value()),
            growth_factor: Some(growth_rate),
            price_current: None, // Not available in DDD model
            market_cap: None,    // Not available in DDD model
            volume: None,        // Not available in DDD model
            ranking_position: rank.map(|r| r as i32),
            quarterly_data: None,
            next_earnings_date: None,
            last_earnings_date: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared_kernel::entities::eps_growth::EPSRanking as LegacyEPSRanking;

    #[test]
    fn test_legacy_to_ddd_entry_conversion() {
        let legacy_ranking = LegacyEPSRanking {
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            country: "america".to_string(),
            sector: "Technology".to_string(),
            exchange: "NASDAQ".to_string(),
            current_eps: Some(1.52),
            growth_factor: Some(15.2),
            price_current: Some(150.25),
            market_cap: Some(2500000000000),
            volume: Some(45678900),
            ranking_position: Some(1),
            quarterly_data: None,
            next_earnings_date: None,
            last_earnings_date: None,
        };

        match EPSRankingMapper::legacy_to_ddd_entry(&legacy_ranking) {
            Ok(entry) => {
                assert_eq!(entry.symbol.as_str(), "AAPL");
                assert_eq!(entry.company_name, "Apple Inc.");
                assert_eq!(entry.eps_value.value(), 1.52);
                assert_eq!(entry.growth_factor.percentage(), 15.2);
                assert_eq!(entry.sector.name(), "Technology");
                assert_eq!(entry.country.name(), "america");
            }
            Err(e) => panic!("Conversion failed: {}", e),
        }
    }

    #[test]
    fn test_ddd_entry_to_legacy_conversion() {
        let symbol = StockSymbol::new("AAPL".to_string()).unwrap();
        let eps_value = EPSValue::new(1.52).unwrap();
        let growth_factor = GrowthFactor::new(15.2).unwrap();
        let sector = MarketSector::new("Technology".to_string()).unwrap();
        let country = Country::new("america".to_string()).unwrap();

        let entry = RankingEntry {
            symbol,
            company_name: "Apple Inc.".to_string(),
            eps_value,
            growth_factor,
            sector,
            country,
            score: 85.5,
            added_at: Utc::now(),
        };

        let legacy_ranking = EPSRankingMapper::ddd_entry_to_legacy(&entry, 1);
        
        assert_eq!(legacy_ranking.symbol, "AAPL");
        assert_eq!(legacy_ranking.name, "Apple Inc.");
        assert_eq!(legacy_ranking.current_eps, Some(1.52));
        assert_eq!(legacy_ranking.growth_factor, Some(15.2));
        assert_eq!(legacy_ranking.ranking_position, Some(1));
        assert_eq!(legacy_ranking.sector, "Technology");
        assert_eq!(legacy_ranking.country, "america");
    }

    #[test]
    fn test_build_ddd_ranking_from_collection() {
        let legacy_rankings = vec![
            LegacyEPSRanking {
                symbol: "AAPL".to_string(),
                name: "Apple Inc.".to_string(),
                country: "america".to_string(),
                sector: "Technology".to_string(),
                exchange: "NASDAQ".to_string(),
                current_eps: Some(1.52),
                growth_factor: Some(15.2),
                price_current: Some(150.25),
                market_cap: Some(2500000000000),
                volume: Some(45678900),
                ranking_position: Some(1),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            LegacyEPSRanking {
                symbol: "GOOGL".to_string(),
                name: "Alphabet Inc.".to_string(),
                country: "america".to_string(),
                sector: "Technology".to_string(),
                exchange: "NASDAQ".to_string(),
                current_eps: Some(2.15),
                growth_factor: Some(12.8),
                price_current: Some(140.50),
                market_cap: Some(1800000000000),
                volume: Some(32145600),
                ranking_position: Some(2),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
        ];

        match EPSRankingMapper::build_ddd_ranking_from_legacy_collection(
            &legacy_rankings,
            RankingType::Combined,
            Some(SectorCategory::Technology),
            Some(Country::new("america".to_string()).unwrap()),
        ) {
            Ok(ddd_ranking) => {
                assert_eq!(ddd_ranking.total_entries(), 2);
                assert_eq!(ddd_ranking.ranking_type(), &RankingType::Combined);
                assert_eq!(ddd_ranking.sector_filter(), Some(&SectorCategory::Technology));
                
                let top_entries = ddd_ranking.top_entries(10);
                assert_eq!(top_entries.len(), 2);
            }
            Err(e) => panic!("Collection conversion failed: {}", e),
        }
    }
}