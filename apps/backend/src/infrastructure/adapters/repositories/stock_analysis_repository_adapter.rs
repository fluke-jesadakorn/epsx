use std::sync::Arc;
use tracing::{debug, error, info};
use chrono::Utc;

use crate::domain::trading_analytics::aggregates::eps_ranking::{EPSRanking as DDDEPSRanking, RankingEntry, RankingType, RankingPeriod};
use crate::domain::trading_analytics::value_objects::*;
use crate::domain::shared_kernel::entities::eps_growth::{EPSRanking as LegacyEPSRanking};
use crate::domain::shared_kernel::services::eps_ranking_service::{EPSRankingService, EPSRankingParams};

/// Repository adapter that bridges legacy EPS ranking system with DDD Trading Analytics
pub struct StockAnalysisRepositoryAdapter {
    eps_service: Arc<EPSRankingService>,
}

unsafe impl Send for StockAnalysisRepositoryAdapter {}
unsafe impl Sync for StockAnalysisRepositoryAdapter {}

impl StockAnalysisRepositoryAdapter {
    pub fn new(eps_service: Arc<EPSRankingService>) -> Self {
        Self { eps_service }
    }

    /// Convert legacy EPSRanking to DDD RankingEntry
    fn convert_legacy_to_ddd_entry(&self, legacy_ranking: &LegacyEPSRanking) -> Result<RankingEntry, String> {
        let symbol = StockSymbol::new(legacy_ranking.symbol.clone())
            .map_err(|e| format!("Invalid symbol: {}", e))?;
        
        let eps_value = EPSValue::new(legacy_ranking.eps_current)
            .map_err(|e| format!("Invalid EPS value: {}", e))?;
        
        let growth_factor = GrowthFactor::new(legacy_ranking.growth_rate)
            .map_err(|e| format!("Invalid growth factor: {}", e))?;
        
        let sector = MarketSector::new(legacy_ranking.sector.clone())
            .map_err(|e| format!("Invalid sector: {}", e))?;
        
        let country = Country::new("US".to_string()) // Default country as field not available
            .map_err(|e| format!("Invalid country: {}", e))?;

        Ok(RankingEntry {
            symbol,
            company_name: legacy_ranking.company_name.clone(),
            eps_value,
            growth_factor,
            sector,
            country,
            score: 0.0, // Will be calculated by DDD aggregate
            added_at: Utc::now(),
        })
    }

    /// Convert DDD RankingEntry to legacy EPSRanking
    fn convert_ddd_to_legacy_ranking(&self, entry: &RankingEntry, rank: u32) -> LegacyEPSRanking {
        LegacyEPSRanking {
            symbol: entry.symbol.as_str().to_string(),
            company_name: entry.company_name.clone(),
            eps_current: entry.eps_value.value(),
            eps_previous: 0.0, // Not available in DDD model
            growth_rate: entry.growth_factor.percentage(),
            rank,
            sector: entry.sector.name().to_string(),
            market_cap: None,    // Not available in DDD model
            price_current: None, // Not available in DDD model
            last_updated: chrono::Utc::now(),
        }
    }

    /// Build DDD EPSRanking from legacy data
    async fn build_ddd_ranking_from_legacy(&self, params: &EPSRankingParams) -> Result<DDDEPSRanking, String> {
        debug!("Building DDD ranking from legacy data with params: {:?}", params);
        
        // Get legacy data
        let legacy_result = self.eps_service.get_eps_rankings(params.clone()).await
            .map_err(|e| format!("Failed to get legacy rankings: {}", e))?;

        // Determine ranking configuration
        let ranking_type = match params.sort_by.as_deref() {
            Some("eps") => RankingType::EPSValue,
            Some("growth") | Some("qoq_growth") => RankingType::EPSGrowth,
            _ => RankingType::Combined,
        };

        let sector_filter = params.sector.as_ref()
            .and_then(|s| MarketSector::new(s.clone()).ok())
            .map(|ms| ms.category().clone());
        
        let country_filter = params.country.as_ref()
            .and_then(|c| Country::new(c.clone()).ok());

        // Create DDD ranking aggregate
        let mut ddd_ranking = DDDEPSRanking::new(
            ranking_type,
            RankingPeriod::Quarterly, // Default period
            sector_filter,
            country_filter,
        );

        // Add entries from legacy data
        for (index, legacy_ranking) in legacy_result.rankings.iter().enumerate() {
            match self.convert_legacy_to_ddd_entry(legacy_ranking) {
                Ok(entry) => {
                    match ddd_ranking.add_entry(
                        entry.symbol.clone(),
                        entry.company_name.clone(),
                        entry.eps_value,
                        entry.growth_factor,
                        entry.sector,
                        entry.country,
                    ) {
                        Ok(_rank) => {
                            debug!("Added entry {} to DDD ranking", entry.symbol.as_str());
                        }
                        Err(e) => {
                            error!("Failed to add entry {} to DDD ranking: {}", entry.symbol.as_str(), e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to convert legacy ranking to DDD entry: {}", e);
                }
            }
        }

        info!("Built DDD ranking with {} entries from legacy data", ddd_ranking.total_entries());
        Ok(ddd_ranking)
    }
}

impl StockAnalysisRepositoryAdapter {
    /// Get DDD ranking from legacy system using parameters
    pub async fn get_ddd_ranking_by_params(&self, params: &EPSRankingParams) -> Result<DDDEPSRanking, String> {
        self.build_ddd_ranking_from_legacy(params).await
    }

    /// Convert legacy ranking result to DDD format
    pub async fn convert_legacy_rankings_to_ddd(&self, params: EPSRankingParams) -> Result<Vec<LegacyEPSRanking>, String> {
        debug!("Converting legacy rankings to DDD format with params: {:?}", params);

        // Get DDD ranking from legacy system
        let ddd_ranking = self.build_ddd_ranking_from_legacy(&params).await?;

        // Convert back to legacy format for API compatibility
        let entries = ddd_ranking.top_entries(params.limit as usize);
        let rankings: Vec<LegacyEPSRanking> = entries.iter()
            .map(|(rank, entry)| self.convert_ddd_to_legacy_ranking(entry, *rank))
            .collect();

        info!("Converted {} DDD entries back to legacy format", rankings.len());
        Ok(rankings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared_kernel::entities::eps_growth::EPSRanking as LegacyEPSRanking;

    #[test]
    fn test_legacy_to_ddd_conversion() {
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

        let eps_service = Arc::new(EPSRankingService::new());
        let adapter = StockAnalysisRepositoryAdapter::new(eps_service);
        
        match adapter.convert_legacy_to_ddd_entry(&legacy_ranking) {
            Ok(entry) => {
                assert_eq!(entry.symbol.as_str(), "AAPL");
                assert_eq!(entry.company_name, "Apple Inc.");
                assert_eq!(entry.eps_value.value(), 1.52);
                assert_eq!(entry.growth_factor.percentage(), 15.2);
            }
            Err(e) => panic!("Conversion failed: {}", e),
        }
    }

    #[test]
    fn test_ddd_to_legacy_conversion() {
        let symbol = StockSymbol::new("AAPL").unwrap();
        let eps_value = EPSValue::new(1.52).unwrap();
        let growth_factor = GrowthFactor::new(15.2).unwrap();
        let sector = MarketSector::from_string("Technology").unwrap();
        let country = Country::from_name("america").unwrap();

        let ddd_entry = RankingEntry {
            symbol,
            company_name: "Apple Inc.".to_string(),
            eps_value,
            growth_factor,
            sector,
            country,
            score: 85.5,
            added_at: Utc::now(),
        };

        let eps_service = Arc::new(EPSRankingService::new());
        let adapter = StockAnalysisRepositoryAdapter::new(eps_service);
        
        let legacy_ranking = adapter.convert_ddd_to_legacy_ranking(&ddd_entry, 1);
        
        assert_eq!(legacy_ranking.symbol, "AAPL");
        assert_eq!(legacy_ranking.name, "Apple Inc.");
        assert_eq!(legacy_ranking.current_eps, Some(1.52));
        assert_eq!(legacy_ranking.growth_factor, Some(15.2));
        assert_eq!(legacy_ranking.ranking_position, Some(1));
    }
}