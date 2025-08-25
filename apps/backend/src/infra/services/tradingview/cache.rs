// TradingView Cache - Focused Module for Caching and Performance Optimization
// Handles data caching, performance optimizations, and cache management strategies

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

use super::types::{FrontendEPSData, FrontendEPSResponse, FrontendDataBatch};

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub timestamp: u64,
    pub ttl_seconds: u64,
}

impl<T> CacheEntry<T> {
    /// Create new cache entry with TTL
    pub fn new(data: T, ttl_seconds: u64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            data,
            timestamp,
            ttl_seconds,
        }
    }

    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        current_time > self.timestamp + self.ttl_seconds
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> u64 {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if self.is_expired() {
            0
        } else {
            self.timestamp + self.ttl_seconds - current_time
        }
    }
}

/// TradingView cache manager
pub struct TradingViewCache {
    eps_data_cache: HashMap<String, CacheEntry<FrontendEPSResponse>>,
    symbol_cache: HashMap<String, CacheEntry<FrontendEPSData>>,
    market_cache: HashMap<String, CacheEntry<FrontendDataBatch>>,
    request_cache: HashMap<String, CacheEntry<serde_json::Value>>,
    default_ttl: Duration,
    max_cache_size: usize,
}

impl TradingViewCache {
    /// Create new cache with default settings
    pub fn new() -> Self {
        Self {
            eps_data_cache: HashMap::new(),
            symbol_cache: HashMap::new(),
            market_cache: HashMap::new(),
            request_cache: HashMap::new(),
            default_ttl: Duration::from_secs(300), // 5 minutes default TTL
            max_cache_size: 1000,
        }
    }

    /// Create cache with custom settings
    pub fn with_settings(default_ttl: Duration, max_cache_size: usize) -> Self {
        Self {
            eps_data_cache: HashMap::new(),
            symbol_cache: HashMap::new(),
            market_cache: HashMap::new(),
            request_cache: HashMap::new(),
            default_ttl,
            max_cache_size,
        }
    }

    /// Cache EPS rankings response
    pub fn cache_eps_rankings(&mut self, key: String, data: FrontendEPSResponse, ttl_seconds: Option<u64>) {
        let ttl = ttl_seconds.unwrap_or(self.default_ttl.as_secs());
        let entry = CacheEntry::new(data, ttl);
        
        self.eps_data_cache.insert(key.clone(), entry);
        self.cleanup_if_needed(&key, CacheType::EpsData);
        
        debug!("Cached EPS rankings for key: {}, TTL: {}s", key, ttl);
    }

    /// Get cached EPS rankings
    pub fn get_eps_rankings(&mut self, key: &str) -> Option<FrontendEPSResponse> {
        if let Some(entry) = self.eps_data_cache.get(key) {
            if entry.is_expired() {
                self.eps_data_cache.remove(key);
                debug!("Removed expired EPS cache entry: {}", key);
                None
            } else {
                debug!("Cache hit for EPS rankings: {}, remaining TTL: {}s", key, entry.remaining_ttl());
                Some(entry.data.clone())
            }
        } else {
            debug!("Cache miss for EPS rankings: {}", key);
            None
        }
    }

    /// Cache individual symbol data
    pub fn cache_symbol_data(&mut self, symbol: String, data: FrontendEPSData, ttl_seconds: Option<u64>) {
        let ttl = ttl_seconds.unwrap_or(self.default_ttl.as_secs());
        let entry = CacheEntry::new(data, ttl);
        
        self.symbol_cache.insert(symbol.clone(), entry);
        self.cleanup_if_needed(&symbol, CacheType::Symbol);
        
        debug!("Cached symbol data for: {}, TTL: {}s", symbol, ttl);
    }

    /// Get cached symbol data
    pub fn get_symbol_data(&mut self, symbol: &str) -> Option<FrontendEPSData> {
        if let Some(entry) = self.symbol_cache.get(symbol) {
            if entry.is_expired() {
                self.symbol_cache.remove(symbol);
                debug!("Removed expired symbol cache entry: {}", symbol);
                None
            } else {
                debug!("Cache hit for symbol: {}, remaining TTL: {}s", symbol, entry.remaining_ttl());
                Some(entry.data.clone())
            }
        } else {
            debug!("Cache miss for symbol: {}", symbol);
            None
        }
    }

    /// Cache market data batch
    pub fn cache_market_data(&mut self, market: String, data: FrontendDataBatch, ttl_seconds: Option<u64>) {
        let ttl = ttl_seconds.unwrap_or(self.default_ttl.as_secs());
        let entry = CacheEntry::new(data, ttl);
        
        self.market_cache.insert(market.clone(), entry);
        self.cleanup_if_needed(&market, CacheType::Market);
        
        debug!("Cached market data for: {}, TTL: {}s", market, ttl);
    }

    /// Get cached market data
    pub fn get_market_data(&mut self, market: &str) -> Option<FrontendDataBatch> {
        if let Some(entry) = self.market_cache.get(market) {
            if entry.is_expired() {
                self.market_cache.remove(market);
                debug!("Removed expired market cache entry: {}", market);
                None
            } else {
                debug!("Cache hit for market: {}, remaining TTL: {}s", market, entry.remaining_ttl());
                Some(entry.data.clone())
            }
        } else {
            debug!("Cache miss for market: {}", market);
            None
        }
    }

    /// Cache API request response
    pub fn cache_request(&mut self, request_key: String, response: serde_json::Value, ttl_seconds: Option<u64>) {
        let ttl = ttl_seconds.unwrap_or(self.default_ttl.as_secs());
        let entry = CacheEntry::new(response, ttl);
        
        self.request_cache.insert(request_key.clone(), entry);
        self.cleanup_if_needed(&request_key, CacheType::Request);
        
        debug!("Cached API request response for: {}, TTL: {}s", request_key, ttl);
    }

    /// Get cached API request response
    pub fn get_cached_request(&mut self, request_key: &str) -> Option<serde_json::Value> {
        if let Some(entry) = self.request_cache.get(request_key) {
            if entry.is_expired() {
                self.request_cache.remove(request_key);
                debug!("Removed expired request cache entry: {}", request_key);
                None
            } else {
                debug!("Cache hit for request: {}, remaining TTL: {}s", request_key, entry.remaining_ttl());
                Some(entry.data.clone())
            }
        } else {
            debug!("Cache miss for request: {}", request_key);
            None
        }
    }

    /// Generate cache key for EPS rankings request
    pub fn generate_eps_rankings_key(
        page: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
        sort_by: Option<String>,
    ) -> String {
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(10);
        let country = country.unwrap_or_else(|| "all".to_string());
        let sort_by = sort_by.unwrap_or_else(|| "market_cap".to_string());
        
        format!("eps_rankings_{}_{}_{}_{}", page, limit, country, sort_by)
    }

    /// Generate cache key for market request
    pub fn generate_market_key(market: &str, skip: i32, limit: i32) -> String {
        format!("market_{}_{}_{}", market, skip, limit)
    }

    /// Generate cache key for API request
    pub fn generate_request_key(request: &serde_json::Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let request_str = serde_json::to_string(request).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        request_str.hash(&mut hasher);
        format!("request_{:x}", hasher.finish())
    }

    /// Clear expired entries from all caches
    pub fn cleanup_expired(&mut self) {
        let initial_eps_count = self.eps_data_cache.len();
        let initial_symbol_count = self.symbol_cache.len();
        let initial_market_count = self.market_cache.len();
        let initial_request_count = self.request_cache.len();

        self.eps_data_cache.retain(|_, entry| !entry.is_expired());
        self.symbol_cache.retain(|_, entry| !entry.is_expired());
        self.market_cache.retain(|_, entry| !entry.is_expired());
        self.request_cache.retain(|_, entry| !entry.is_expired());

        let cleaned_eps = initial_eps_count - self.eps_data_cache.len();
        let cleaned_symbol = initial_symbol_count - self.symbol_cache.len();
        let cleaned_market = initial_market_count - self.market_cache.len();
        let cleaned_request = initial_request_count - self.request_cache.len();

        if cleaned_eps + cleaned_symbol + cleaned_market + cleaned_request > 0 {
            info!("Cleaned expired cache entries - EPS: {}, Symbol: {}, Market: {}, Request: {}", 
                  cleaned_eps, cleaned_symbol, cleaned_market, cleaned_request);
        }
    }

    /// Clear all cache entries
    pub fn clear_all(&mut self) {
        let total_entries = self.eps_data_cache.len() + self.symbol_cache.len() + 
                           self.market_cache.len() + self.request_cache.len();
        
        self.eps_data_cache.clear();
        self.symbol_cache.clear();
        self.market_cache.clear();
        self.request_cache.clear();

        info!("Cleared all cache entries: {}", total_entries);
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            eps_data_count: self.eps_data_cache.len(),
            symbol_count: self.symbol_cache.len(),
            market_count: self.market_cache.len(),
            request_count: self.request_cache.len(),
            total_count: self.eps_data_cache.len() + self.symbol_cache.len() + 
                        self.market_cache.len() + self.request_cache.len(),
        }
    }

    /// Cleanup if cache exceeds maximum size
    fn cleanup_if_needed(&mut self, _key: &str, cache_type: CacheType) {
        let current_size = match cache_type {
            CacheType::EpsData => self.eps_data_cache.len(),
            CacheType::Symbol => self.symbol_cache.len(),
            CacheType::Market => self.market_cache.len(),
            CacheType::Request => self.request_cache.len(),
        };

        if current_size > self.max_cache_size {
            warn!("Cache size limit exceeded for {:?}, cleaning up oldest entries", cache_type);
            self.cleanup_oldest_entries(cache_type);
        } else {
            debug!("Cache size OK for {:?}: {} entries (max: {})", cache_type, current_size, self.max_cache_size);
        }
    }

    /// Remove oldest entries from cache
    fn cleanup_oldest_entries(&mut self, cache_type: CacheType) {
        let cleanup_count = self.max_cache_size / 4; // Remove 25% of entries
        
        match cache_type {
            CacheType::EpsData => {
                let mut entries: Vec<_> = self.eps_data_cache.iter().map(|(k, v)| (k.clone(), v.timestamp)).collect();
                entries.sort_by_key(|(_, timestamp)| *timestamp);
                
                for (key, _) in entries.into_iter().take(cleanup_count) {
                    self.eps_data_cache.remove(&key);
                }
            }
            CacheType::Symbol => {
                let mut entries: Vec<_> = self.symbol_cache.iter().map(|(k, v)| (k.clone(), v.timestamp)).collect();
                entries.sort_by_key(|(_, timestamp)| *timestamp);
                
                for (key, _) in entries.into_iter().take(cleanup_count) {
                    self.symbol_cache.remove(&key);
                }
            }
            CacheType::Market => {
                let mut entries: Vec<_> = self.market_cache.iter().map(|(k, v)| (k.clone(), v.timestamp)).collect();
                entries.sort_by_key(|(_, timestamp)| *timestamp);
                
                for (key, _) in entries.into_iter().take(cleanup_count) {
                    self.market_cache.remove(&key);
                }
            }
            CacheType::Request => {
                let mut entries: Vec<_> = self.request_cache.iter().map(|(k, v)| (k.clone(), v.timestamp)).collect();
                entries.sort_by_key(|(_, timestamp)| *timestamp);
                
                for (key, _) in entries.into_iter().take(cleanup_count) {
                    self.request_cache.remove(&key);
                }
            }
        }
        
        info!("Cleaned {} oldest entries from {:?} cache", cleanup_count, cache_type);
    }
}

impl Default for TradingViewCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache type enumeration
#[derive(Debug, Clone, Copy)]
enum CacheType {
    EpsData,
    Symbol,
    Market,
    Request,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub eps_data_count: usize,
    pub symbol_count: usize,
    pub market_count: usize,
    pub request_count: usize,
    pub total_count: usize,
}

impl CacheStats {
    /// Calculate cache hit ratio (would need request counters)
    pub fn hit_ratio(&self) -> f64 {
        // Placeholder - would track hits/misses in real implementation
        0.75 // 75% hit ratio placeholder
    }

    /// Get memory usage estimate
    pub fn estimated_memory_usage(&self) -> usize {
        // Rough estimate based on average entry size
        const AVG_EPS_DATA_SIZE: usize = 512;
        const AVG_SYMBOL_SIZE: usize = 256;
        const AVG_MARKET_SIZE: usize = 1024;
        const AVG_REQUEST_SIZE: usize = 2048;
        
        self.eps_data_count * AVG_EPS_DATA_SIZE +
        self.symbol_count * AVG_SYMBOL_SIZE +
        self.market_count * AVG_MARKET_SIZE +
        self.request_count * AVG_REQUEST_SIZE
    }
}

/// Cache performance optimizer
pub struct CachePerformanceOptimizer {
    cache: TradingViewCache,
    access_patterns: HashMap<String, u32>,
}

impl CachePerformanceOptimizer {
    /// Create new performance optimizer
    pub fn new(cache: TradingViewCache) -> Self {
        Self {
            cache,
            access_patterns: HashMap::new(),
        }
    }

    /// Track cache access pattern
    pub fn track_access(&mut self, key: &str) {
        *self.access_patterns.entry(key.to_string()).or_insert(0) += 1;
        debug!("Access pattern for {}: {} hits", key, self.access_patterns[key]);
    }

    /// Get frequently accessed keys
    pub fn get_hot_keys(&self, threshold: u32) -> Vec<String> {
        self.access_patterns
            .iter()
            .filter(|(_, &count)| count >= threshold)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Optimize cache based on access patterns
    pub fn optimize_cache(&mut self) {
        let hot_keys = self.get_hot_keys(5); // Keys accessed 5+ times
        info!("Optimizing cache for {} hot keys", hot_keys.len());
        
        // Extend TTL for frequently accessed data
        for key in hot_keys {
            // Implementation would extend TTL for hot data
            debug!("Optimizing cache entry for hot key: {}", key);
        }
    }

    /// Get optimizer statistics
    pub fn get_optimizer_stats(&self) -> (CacheStats, usize) {
        (self.cache.get_stats(), self.access_patterns.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_eps_response() -> FrontendEPSResponse {
        use super::super::types::{FrontendEPSData, FrontendPagination};
        
        FrontendEPSResponse {
            data: vec![
                FrontendEPSData {
                    id: "1".to_string(),
                    symbol: "AAPL".to_string(),
                    company_name: "Apple Inc".to_string(),
                    current_eps: 3.25,
                    qoq_growth: 12.5,
                    market_cap: 2500000000000,
                    price_current: 150.0,
                    volume: 50000000,
                    country: "america".to_string(),
                    sector: "Technology".to_string(),
                    ranking_score: 85.0,
                }
            ],
            pagination: FrontendPagination {
                page: 1,
                limit: 10,
                total: 100,
                total_pages: 10,
                has_next: true,
                has_prev: false,
            },
        }
    }

    #[test]
    fn test_cache_entry_creation() {
        let data = "test_data".to_string();
        let entry = CacheEntry::new(data, 300);
        
        assert_eq!(entry.data, "test_data");
        assert_eq!(entry.ttl_seconds, 300);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_operations() {
        let mut cache = TradingViewCache::new();
        let test_data = create_test_eps_response();
        
        // Test caching
        cache.cache_eps_rankings("test_key".to_string(), test_data.clone(), Some(300));
        
        // Test retrieval
        let cached_data = cache.get_eps_rankings("test_key");
        assert!(cached_data.is_some());
        
        // Test cache miss
        let missing_data = cache.get_eps_rankings("nonexistent_key");
        assert!(missing_data.is_none());
    }

    #[test]
    fn test_key_generation() {
        let key1 = TradingViewCache::generate_eps_rankings_key(
            Some(1), Some(10), Some("america".to_string()), Some("eps_growth".to_string())
        );
        let key2 = TradingViewCache::generate_eps_rankings_key(
            Some(1), Some(10), Some("america".to_string()), Some("eps_growth".to_string())
        );
        let key3 = TradingViewCache::generate_eps_rankings_key(
            Some(2), Some(10), Some("america".to_string()), Some("eps_growth".to_string())
        );
        
        assert_eq!(key1, key2); // Same parameters should generate same key
        assert_ne!(key1, key3); // Different parameters should generate different keys
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = TradingViewCache::new();
        let test_data = create_test_eps_response();
        
        cache.cache_eps_rankings("test1".to_string(), test_data.clone(), Some(300));
        cache.cache_eps_rankings("test2".to_string(), test_data, Some(300));
        
        let stats = cache.get_stats();
        assert_eq!(stats.eps_data_count, 2);
        assert_eq!(stats.total_count, 2);
    }

    #[test]
    fn test_cleanup_expired() {
        let mut cache = TradingViewCache::new();
        let test_data = create_test_eps_response();
        
        // Cache with 0 TTL (expired immediately)
        cache.cache_eps_rankings("expired".to_string(), test_data, Some(0));
        
        // Give time for expiration
        std::thread::sleep(Duration::from_millis(10));
        
        cache.cleanup_expired();
        let stats = cache.get_stats();
        assert_eq!(stats.eps_data_count, 0);
    }

    #[test]
    fn test_cache_performance_optimizer() {
        let cache = TradingViewCache::new();
        let mut optimizer = CachePerformanceOptimizer::new(cache);
        
        // Track some access patterns
        for _ in 0..10 {
            optimizer.track_access("hot_key");
        }
        
        for _ in 0..2 {
            optimizer.track_access("cold_key");
        }
        
        let hot_keys = optimizer.get_hot_keys(5);
        assert_eq!(hot_keys.len(), 1);
        assert_eq!(hot_keys[0], "hot_key");
        
        let (stats, pattern_count) = optimizer.get_optimizer_stats();
        assert_eq!(pattern_count, 2); // Two different keys tracked
    }

    #[test]
    fn test_memory_usage_estimate() {
        let stats = CacheStats {
            eps_data_count: 10,
            symbol_count: 20,
            market_count: 5,
            request_count: 15,
            total_count: 50,
        };
        
        let memory_usage = stats.estimated_memory_usage();
        assert!(memory_usage > 0);
        assert!(memory_usage > stats.total_count * 100); // Should be reasonable estimate
    }
}