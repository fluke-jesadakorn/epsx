use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn, error};

use crate::dom::entities::eps_growth::{EPSGrowthData, EPSRanking};
use crate::dom::services::eps_ranking_service::EPSRepository;
use crate::core::errors::AppError;
use crate::infra::db::diesel::DbPool;
use crate::infra::cache::{Cache, CacheExt};

pub struct DieselEPSRepository {
    _pool: Arc<DbPool>,
    cache: Arc<dyn Cache>,
}

impl DieselEPSRepository {
    pub fn new(pool: Arc<DbPool>, cache: Arc<dyn Cache>) -> Self {
        Self { _pool: pool, cache }
    }

    fn get_cache_key(&self, key_type: &str, params: &str) -> String {
        format!("eps:{}:{}", key_type, params)
    }
}

#[async_trait]
impl EPSRepository for DieselEPSRepository {
    async fn store_eps_data(&self, eps_data: EPSGrowthData) -> Result<(), AppError> {
        debug!("Storing EPS data for symbol: {}", eps_data.symbol);
        
        // Store in cache with 30-minute TTL (1800 seconds)
        let cache_key = self.get_cache_key("data", &eps_data.symbol);
        
        match self.cache.set(&cache_key, &eps_data, Some(1800)).await {
            Ok(_) => {
                info!("Successfully cached EPS data for {}", eps_data.symbol);
                
                // Invalidate related cache entries
                let _ = self.cache.delete(&self.get_cache_key("rankings", "all")).await;
                
                Ok(())
            }
            Err(e) => {
                error!("Failed to cache EPS data for {}: {}", eps_data.symbol, e);
                Err(AppError::cache_error(format!("Cache storage failed: {}", e)))
            }
        }
    }

    async fn get_rankings_filtered(
        &self,
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
        page: i32,
        limit: i32,
    ) -> Result<Vec<EPSRanking>, AppError> {
        let params = format!("{}:{}:{}:{}:{}", 
                           country.as_deref().unwrap_or("all"),
                           sector.as_deref().unwrap_or("all"),
                           sort_by.as_deref().unwrap_or("growth_factor"),
                           page, limit);
        let cache_key = self.get_cache_key("rankings", &params);
        
        debug!("Fetching EPS rankings from cache with key: {}", cache_key);
        
        // Try cache first
        match self.cache.get::<Vec<EPSRanking>>(&cache_key).await {
            Ok(Some(cached_rankings)) => {
                info!("Cache hit: Found {} cached EPS rankings", cached_rankings.len());
                return Ok(cached_rankings);
            }
            Ok(None) => debug!("Cache miss: No cached rankings found"),
            Err(e) => warn!("Cache error during rankings fetch: {}", e),
        }
        
        // Generate sample data for development (Phase 2 implementation)
        // This will be replaced with real TradingView data in Phase 3
        let sample_rankings = self.generate_sample_rankings(country, sector, sort_by, page, limit).await;
        
        // Cache the results with 30-minute TTL
        if let Err(e) = self.cache.set(&cache_key, &sample_rankings, Some(1800)).await {
            warn!("Failed to cache rankings: {}", e);
        } else {
            debug!("Cached {} rankings with key: {}", sample_rankings.len(), cache_key);
        }
        
        Ok(sample_rankings)
    }

    async fn get_total_count(&self, country: Option<String>, sector: Option<String>) -> Result<i64, AppError> {
        let cache_key = self.get_cache_key("count", &format!("{}:{}", 
            country.as_deref().unwrap_or("all"), 
            sector.as_deref().unwrap_or("all")));
        
        // Try cache first
        match self.cache.get::<i64>(&cache_key).await {
            Ok(Some(cached_count)) => {
                debug!("Cache hit: Found cached count {}", cached_count);
                return Ok(cached_count);
            }
            Ok(None) => debug!("Cache miss: No cached count found"),
            Err(e) => warn!("Cache error during count fetch: {}", e),
        }
        
        // Sample count for development - consider both country and sector
        let count = match (country.as_deref(), sector.as_deref()) {
            // Country + Sector combinations
            (Some("america"), Some("Technology")) => 25,
            (Some("america"), Some("Finance")) => 15,
            (Some("america"), Some("Consumer Goods")) => 10,
            (Some("germany"), Some("Technology")) => 8,
            (Some("japan"), Some("Technology")) => 12,
            // Country only
            (Some("america"), None) => 150,
            (Some("germany"), None) => 80,
            (Some("japan"), None) => 120,
            (Some("taiwan"), None) => 95,
            (Some("hongkong"), None) => 60,
            // Sector only (across all countries)
            (None, Some("Technology")) => 180,
            (None, Some("Finance")) => 120,
            (None, Some("Consumer Goods")) => 90,
            (None, Some("Healthcare")) => 85,
            // No filters
            _ => 500, // Total across all countries and sectors
        };
        
        // Cache with 30-minute TTL
        if let Err(e) = self.cache.set(&cache_key, &count, Some(1800)).await {
            warn!("Failed to cache count: {}", e);
        }
        
        Ok(count)
    }

    async fn batch_store_eps_data(&self, eps_data_list: Vec<EPSGrowthData>) -> Result<usize, AppError> {
        debug!("Batch storing {} EPS data entries", eps_data_list.len());
        
        let mut stored_count = 0;
        
        for eps_data in eps_data_list {
            match self.store_eps_data(eps_data).await {
                Ok(_) => stored_count += 1,
                Err(e) => warn!("Failed to store EPS data: {}", e),
            }
        }
        
        info!("Successfully stored {} out of {} EPS data entries", stored_count, stored_count);
        Ok(stored_count)
    }

    async fn get_countries(&self) -> Result<Vec<String>, AppError> {
        let cache_key = self.get_cache_key("countries", "all");
        
        // Try cache first
        match self.cache.get::<Vec<String>>(&cache_key).await {
            Ok(Some(cached_countries)) => {
                debug!("Cache hit: Found {} cached countries", cached_countries.len());
                return Ok(cached_countries);
            }
            Ok(None) => debug!("Cache miss: No cached countries found"),
            Err(e) => warn!("Cache error during countries fetch: {}", e),
        }
        
        // Sample countries for development
        let countries = vec![
            "america".to_string(),
            "germany".to_string(),
            "japan".to_string(),
            "united kingdom".to_string(),
            "canada".to_string(),
        ];
        
        // Cache with 4-hour TTL (14400 seconds) - countries change less frequently
        if let Err(e) = self.cache.set(&cache_key, &countries, Some(14400)).await {
            warn!("Failed to cache countries: {}", e);
        }
        
        Ok(countries)
    }

    async fn get_sectors_by_country(&self, country: Option<String>) -> Result<Vec<String>, AppError> {
        let cache_key = self.get_cache_key("sectors", country.as_deref().unwrap_or("all"));
        
        // Try cache first
        match self.cache.get::<Vec<String>>(&cache_key).await {
            Ok(Some(cached_sectors)) => {
                debug!("Cache hit: Found {} cached sectors", cached_sectors.len());
                return Ok(cached_sectors);
            }
            Ok(None) => debug!("Cache miss: No cached sectors found"),
            Err(e) => warn!("Cache error during sectors fetch: {}", e),
        }
        
        // Sample sectors for development
        let sectors = vec![
            "Technology".to_string(),
            "Healthcare".to_string(),
            "Finance".to_string(),
            "Consumer Goods".to_string(),
            "Industrial".to_string(),
            "Energy".to_string(),
        ];
        
        // Cache with 4-hour TTL
        if let Err(e) = self.cache.set(&cache_key, &sectors, Some(14400)).await {
            warn!("Failed to cache sectors: {}", e);
        }
        
        Ok(sectors)
    }
}

impl DieselEPSRepository {
    async fn generate_sample_rankings(
        &self,
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
        page: i32,
        limit: i32,
    ) -> Vec<EPSRanking> {
        let start_idx = ((page - 1) * limit) as usize;
        
        // Sample EPS rankings data matching frontend display symbols
        let mut sample_data = vec![
            EPSRanking {
                symbol: "AAPL".to_string(),
                name: "Apple Inc".to_string(),
                country: "america".to_string(),
                sector: "Technology".to_string(),
                exchange: "NASDAQ".to_string(),
                current_eps: Some(6.15),
                growth_factor: Some(25.4),
                price_current: Some(185.64),
                market_cap: Some(2900000000000),
                volume: Some(47200000),
                ranking_position: Some(1),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            EPSRanking {
                symbol: "MSFT".to_string(),
                name: "Microsoft Corp".to_string(),
                country: "america".to_string(),
                sector: "Technology".to_string(),
                exchange: "NASDAQ".to_string(),
                current_eps: Some(12.93),
                growth_factor: Some(18.7),
                price_current: Some(424.12),
                market_cap: Some(3150000000000),
                volume: Some(18500000),
                ranking_position: Some(2),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            EPSRanking {
                symbol: "TSLA".to_string(),
                name: "Tesla Inc".to_string(),
                country: "america".to_string(),
                sector: "Consumer Goods".to_string(),
                exchange: "NASDAQ".to_string(),
                current_eps: Some(3.65),
                growth_factor: Some(15.2),
                price_current: Some(351.67),
                market_cap: Some(1118000000000),
                volume: Some(89200000),
                ranking_position: Some(3),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            EPSRanking {
                symbol: "BRK.A".to_string(),
                name: "Berkshire Hathaway".to_string(),
                country: "america".to_string(),
                sector: "Finance".to_string(),
                exchange: "NYSE".to_string(),
                current_eps: Some(10111.26),
                growth_factor: Some(44.08),
                price_current: Some(737180.0),
                market_cap: Some(850000000000),
                volume: Some(12000),
                ranking_position: Some(4),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            EPSRanking {
                symbol: "2330".to_string(),
                name: "Taiwan Semiconductor".to_string(),
                country: "taiwan".to_string(),
                sector: "Technology".to_string(),
                exchange: "TPE".to_string(),
                current_eps: Some(0.53),
                growth_factor: Some(1.28),
                price_current: Some(1190.0),
                market_cap: Some(6160000000000),
                volume: Some(28500000),
                ranking_position: Some(5),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            EPSRanking {
                symbol: "JPM".to_string(),
                name: "JPMorgan Chase".to_string(),
                country: "america".to_string(),
                sector: "Finance".to_string(),
                exchange: "NYSE".to_string(),
                current_eps: Some(5.24),
                growth_factor: Some(3.35),
                price_current: Some(298.57),
                market_cap: Some(869000000000),
                volume: Some(8900000),
                ranking_position: Some(6),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            EPSRanking {
                symbol: "WMT".to_string(),
                name: "Walmart Inc".to_string(),
                country: "america".to_string(),
                sector: "Consumer Goods".to_string(),
                exchange: "NYSE".to_string(),
                current_eps: Some(0.68),
                growth_factor: Some(11.48),
                price_current: Some(96.05),
                market_cap: Some(669000000000),
                volume: Some(21800000),
                ranking_position: Some(7),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            EPSRanking {
                symbol: "700".to_string(),
                name: "Tencent Holdings".to_string(),
                country: "hongkong".to_string(),
                sector: "Technology".to_string(),
                exchange: "HKEX".to_string(),
                current_eps: Some(7.44),
                growth_factor: Some(5.47),
                price_current: Some(596.5),
                market_cap: Some(5697000000000),
                volume: Some(15200000),
                ranking_position: Some(8),
                quarterly_data: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
        ];
        
        // Filter by country if specified
        if let Some(country_filter) = &country {
            sample_data.retain(|r| r.country == *country_filter);
        }
        
        // Filter by sector if specified
        if let Some(sector_filter) = &sector {
            sample_data.retain(|r| r.sector == *sector_filter);
        }
        
        // Sort by specified field
        match sort_by.as_deref() {
            Some("growth_factor") => {
                sample_data.sort_by(|a, b| {
                    b.growth_factor.unwrap_or(0.0).partial_cmp(&a.growth_factor.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            Some("current_eps") => {
                sample_data.sort_by(|a, b| {
                    b.current_eps.unwrap_or(0.0).partial_cmp(&a.current_eps.unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            _ => {} // Keep default order
        }
        
        // Apply pagination
        sample_data.into_iter().skip(start_idx).take(limit as usize).collect()
    }
}