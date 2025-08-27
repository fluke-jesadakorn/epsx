use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use tokio::sync::RwLock;

use crate::dom::entities::eps_growth::{EPSGrowthData, EPSRanking, EPSRankingsResponse, EPSPagination};
use crate::core::errors::AppError;
use crate::infra::services::tradingview::TradingViewApiService;
use crate::dom::services::eps_ranking_service::EPSRepository;

/// Cache-based EPS service for live data fetching
pub struct EPSCacheService {
    cache: Arc<RwLock<EPSCache>>,
    tradingview_service: Arc<TradingViewApiService>,
    eps_repository: Arc<dyn EPSRepository + Send + Sync>,
    config: EPSCacheConfig,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct EPSCacheConfig {
    pub ttl_seconds: u64,
    pub max_entries: usize,
    pub enable_background_refresh: bool,
    pub batch_size: usize,
}

impl Default for EPSCacheConfig {
    fn default() -> Self {
        Self {
            ttl_seconds: 600, // 10 minutes
            max_entries: 1000,
            enable_background_refresh: false, // Disabled for serverless compatibility
            batch_size: 50,
        }
    }
}

/// In-memory cache for EPS data
#[derive(Debug)]
struct EPSCache {
    entries: HashMap<String, CacheEntry>,
    last_cleanup: std::time::Instant,
    hits: u64,
    misses: u64,
}

/// Individual cache entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CacheEntry {
    data: EPSGrowthData,
    created_at: std::time::Instant,
    access_count: u32,
}

/// Cache statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub active_entries: usize,
    pub expired_entries: usize,
    pub hit_ratio: f64,
    pub miss_ratio: f64,
    pub cache_size_mb: f64,
}

/// Parameters for EPS rankings with caching support
#[derive(Debug, Clone)]
pub struct EPSCacheParams {
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: Option<String>,
    pub page: i32,
    pub limit: i32,
    pub min_eps: Option<f64>,
    pub min_growth: Option<f64>,
    pub force_refresh: bool, // Force cache refresh
}

impl Default for EPSCacheParams {
    fn default() -> Self {
        Self {
            country: None,
            sector: None,
            sort_by: Some("qoq_growth".to_string()),
            page: 1,
            limit: 50,
            min_eps: None,
            min_growth: None,
            force_refresh: false,
        }
    }
}

impl EPSCacheService {
    pub fn new(
        tradingview_service: Arc<TradingViewApiService>,
        eps_repository: Arc<dyn EPSRepository + Send + Sync>,
        config: Option<EPSCacheConfig>,
    ) -> Self {
        let config = config.unwrap_or_default();
        let cache = Arc::new(RwLock::new(EPSCache {
            entries: HashMap::new(),
            last_cleanup: std::time::Instant::now(),
            hits: 0,
            misses: 0,
        }));

        Self {
            cache,
            tradingview_service,
            eps_repository,
            config,
        }
    }

    /// Get EPS rankings with cache-first approach
    pub async fn get_eps_rankings(&self, params: EPSCacheParams) -> Result<EPSRankingsResponse, AppError> {
        debug!("Getting EPS rankings with cache params: {:?}", params);

        // Check cache first (unless force refresh)
        if !params.force_refresh {
            if let Some(cached_data) = self.get_from_cache(&params).await {
                debug!("Cache hit for EPS rankings request");
                return Ok(cached_data);
            }
        }

        // Cache miss - fetch fresh data
        debug!("Cache miss - fetching fresh EPS data from TradingView");
        self.fetch_and_cache_data(&params).await
    }

    /// Check cache for existing data
    async fn get_from_cache(&self, params: &EPSCacheParams) -> Option<EPSRankingsResponse> {
        let cache = self.cache.read().await;
        
        // Get all non-expired entries
        let valid_entries: Vec<EPSRanking> = cache.entries
            .values()
            .filter(|entry| !self.is_expired(entry))
            .map(|entry| self.convert_to_ranking(&entry.data))
            .filter(|ranking| self.matches_filters(ranking, params))
            .collect();

        if valid_entries.is_empty() {
            return None;
        }

        // Update hit statistics
        drop(cache);
        let mut cache = self.cache.write().await;
        cache.hits += 1;

        // Apply sorting and pagination
        let sorted_rankings = self.sort_rankings(valid_entries, &params.sort_by);
        let paginated_result = self.paginate_rankings(sorted_rankings, params.page, params.limit);

        Some(paginated_result)
    }

    /// Fetch fresh data from database first, then enhance with TradingView if needed
    async fn fetch_and_cache_data(&self, params: &EPSCacheParams) -> Result<EPSRankingsResponse, AppError> {
        let start_time = std::time::Instant::now();

        // Primary source: Database EPS rankings
        info!("Fetching EPS rankings from database");
        let db_rankings = self.eps_repository.get_rankings_filtered(
            params.country.clone(),
            params.sort_by.clone(),
            params.page,
            params.limit,
        ).await?;

        debug!("Fetched {} EPS rankings from database", db_rankings.len());

        // Convert database rankings to EPSGrowthData for caching
        let fresh_data: Vec<EPSGrowthData> = db_rankings.iter().map(|ranking| {
            EPSGrowthData {
                symbol: ranking.symbol.clone(),
                name: ranking.name.clone(),
                country: ranking.country.clone(),
                sector: ranking.sector.clone(),
                exchange: ranking.exchange.clone(),
                current_eps: ranking.current_eps,
                growth_factor: ranking.growth_factor,
                price_current: ranking.price_current,
                market_cap: ranking.market_cap,
                volume: ranking.volume,
                ranking_score: None, // Not in EPSRanking, will be calculated
                created_at: None,
                updated_at: None,
            }
        }).collect();

        // Update cache with fresh data from database
        {
            let mut cache = self.cache.write().await;
            cache.misses += 1;
            
            // Clear expired entries during cache update
            self.cleanup_expired_entries(&mut cache);
            
            // Add fresh data to cache
            for eps_data in &fresh_data {
                let entry = CacheEntry {
                    data: eps_data.clone(),
                    created_at: std::time::Instant::now(),
                    access_count: 1,
                };
                cache.entries.insert(eps_data.symbol.clone(), entry);
            }
        }

        // Get total count for pagination
        let total_count = self.eps_repository.get_total_count(params.country.clone()).await?;
        let pagination = EPSPagination::new(params.page, params.limit, total_count);

        // Use database rankings directly (already filtered and sorted)
        let result = EPSRankingsResponse {
            rankings: db_rankings,
            pagination,
        };

        let duration = start_time.elapsed();
        info!("Live EPS data fetch and cache update completed in {:?}", duration);

        Ok(result)
    }

    /// Convert EPSGrowthData to EPSRanking
    fn convert_to_ranking(&self, eps_data: &EPSGrowthData) -> EPSRanking {
        EPSRanking {
            symbol: eps_data.symbol.clone(),
            name: eps_data.name.clone(),
            current_eps: eps_data.current_eps,
            growth_factor: eps_data.growth_factor,
            price_current: eps_data.price_current,
            market_cap: eps_data.market_cap,
            volume: eps_data.volume,
            country: eps_data.country.clone(),
            sector: eps_data.sector.clone(),
            exchange: eps_data.exchange.clone(),
            ranking_position: None, // Will be set during pagination
            quarterly_data: None, // Will be populated by WebSocket enhancement
        }
    }

    /// Check if cache entry matches filter criteria
    fn matches_filters(&self, ranking: &EPSRanking, params: &EPSCacheParams) -> bool {
        // Country filter
        if let Some(ref country) = params.country {
            if ranking.country.to_lowercase() != country.to_lowercase() {
                return false;
            }
        }

        // Sector filter
        if let Some(ref sector) = params.sector {
            if ranking.sector.to_lowercase() != sector.to_lowercase() {
                return false;
            }
        }

        // EPS filter
        if let Some(min_eps) = params.min_eps {
            if ranking.current_eps.unwrap_or(0.0) < min_eps {
                return false;
            }
        }

        // Growth filter
        if let Some(min_growth) = params.min_growth {
            if ranking.growth_factor.unwrap_or(0.0) < min_growth {
                return false;
            }
        }

        true
    }

    /// Sort rankings based on criteria
    fn sort_rankings(&self, mut rankings: Vec<EPSRanking>, sort_by: &Option<String>) -> Vec<EPSRanking> {
        let sort_field = sort_by.as_ref().map(|s| s.as_str()).unwrap_or("growth_factor");
        
        rankings.sort_by(|a, b| {
            match sort_field {
                "growth_factor" | "qoq_growth" => b.growth_factor.unwrap_or(0.0).partial_cmp(&a.growth_factor.unwrap_or(0.0)).unwrap(),
                "market_cap" => b.market_cap.unwrap_or(0).cmp(&a.market_cap.unwrap_or(0)),
                "volume" => b.volume.unwrap_or(0).cmp(&a.volume.unwrap_or(0)),
                "current_eps" => b.current_eps.unwrap_or(0.0).partial_cmp(&a.current_eps.unwrap_or(0.0)).unwrap(),
                "name" => a.name.cmp(&b.name),
                _ => b.growth_factor.unwrap_or(0.0).partial_cmp(&a.growth_factor.unwrap_or(0.0)).unwrap(),
            }
        });

        rankings
    }

    /// Apply pagination to rankings
    fn paginate_rankings(&self, rankings: Vec<EPSRanking>, page: i32, limit: i32) -> EPSRankingsResponse {
        let total_count = rankings.len() as i64;
        let _total_pages = ((total_count as f64) / (limit as f64)).ceil() as i32;
        
        let start_idx = ((page - 1) * limit) as usize;
        let end_idx = (start_idx + limit as usize).min(rankings.len());
        
        let paginated_rankings = if start_idx < rankings.len() {
            rankings[start_idx..end_idx].to_vec()
        } else {
            Vec::new()
        };

        let pagination = EPSPagination::new(page, limit, total_count);

        EPSRankingsResponse {
            rankings: paginated_rankings,
            pagination,
        }
    }

    /// Check if cache entry is expired
    fn is_expired(&self, entry: &CacheEntry) -> bool {
        entry.created_at.elapsed().as_secs() > self.config.ttl_seconds
    }

    /// Clean up expired cache entries
    fn cleanup_expired_entries(&self, cache: &mut EPSCache) {
        let before_count = cache.entries.len();
        cache.entries.retain(|_, entry| !self.is_expired(entry));
        let removed_count = before_count - cache.entries.len();
        
        if removed_count > 0 {
            debug!("Cleaned up {} expired cache entries", removed_count);
        }
        
        cache.last_cleanup = std::time::Instant::now();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        
        let active_entries = cache.entries.values()
            .filter(|entry| !self.is_expired(entry))
            .count();
        
        let expired_entries = cache.entries.len() - active_entries;
        let total_requests = cache.hits + cache.misses;
        
        CacheStats {
            total_entries: cache.entries.len(),
            active_entries,
            expired_entries,
            hit_ratio: if total_requests > 0 { cache.hits as f64 / total_requests as f64 } else { 0.0 },
            miss_ratio: if total_requests > 0 { cache.misses as f64 / total_requests as f64 } else { 0.0 },
            cache_size_mb: (cache.entries.len() * std::mem::size_of::<CacheEntry>()) as f64 / (1024.0 * 1024.0),
        }
    }

    /// Get available countries from database
    pub async fn get_available_countries(&self) -> Result<Vec<String>, AppError> {
        // Use database as source of truth for countries
        self.eps_repository.get_countries().await
    }

    /// Get available sectors from database
    pub async fn get_sectors_by_country(&self, country: Option<String>) -> Result<Vec<String>, AppError> {
        // Use database as source of truth for sectors
        self.eps_repository.get_sectors_by_country(country).await
    }

    /// Get total count for pagination validation from database
    pub async fn get_total_count_for_params(&self, params: &EPSCacheParams) -> Result<i64, AppError> {
        // Use database as source of truth for total count
        self.eps_repository.get_total_count(params.country.clone()).await
    }

    /// Force cache refresh from database
    pub async fn refresh_cache(&self) -> Result<usize, AppError> {
        info!("Forcing cache refresh from database");
        
        let params = EPSCacheParams {
            force_refresh: true,
            ..Default::default()
        };
        
        let result = self.fetch_and_cache_data(&params).await?;
        
        Ok(result.rankings.len())
    }

    // Background cache warming removed - using on-demand refresh instead
    // Use refresh_cache() method for manual cache refresh when needed
}

// Implement Clone for background tasks
impl Clone for EPSCacheService {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            tradingview_service: Arc::clone(&self.tradingview_service),
            eps_repository: Arc::clone(&self.eps_repository),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = EPSCacheConfig::default();
        assert_eq!(config.ttl_seconds, 600);
        assert_eq!(config.max_entries, 1000);
        assert!(config.enable_background_refresh);
    }

    #[test]
    fn test_cache_params_default() {
        let params = EPSCacheParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.limit, 50);
        assert_eq!(params.sort_by, Some("qoq_growth".to_string()));
        assert!(!params.force_refresh);
    }

    #[tokio::test]
    async fn test_cache_stats_calculation() {
        // Test cache statistics calculations
        let stats = CacheStats {
            total_entries: 100,
            active_entries: 80,
            expired_entries: 20,
            hit_ratio: 0.75,
            miss_ratio: 0.25,
            cache_size_mb: 1.5,
        };

        assert_eq!(stats.hit_ratio + stats.miss_ratio, 1.0);
        assert_eq!(stats.active_entries + stats.expired_entries, stats.total_entries);
    }
}