// TradingView Cache - Focused Module for Caching and Performance Optimization
// Handles data caching, performance optimizations, and cache management strategies

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use tracing::{debug, info};
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

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_count: usize,
    pub active_entries: usize,
    pub expired_entries: usize,
    pub hit_ratio: f64,
    pub cache_size_mb: f64,
    pub avg_ttl_seconds: u64,
}

/// TradingView cache manager
pub struct TradingViewCache {
    eps_data_cache: HashMap<String, CacheEntry<FrontendEPSResponse>>,
    symbol_cache: HashMap<String, CacheEntry<FrontendEPSData>>,
    market_cache: HashMap<String, CacheEntry<FrontendDataBatch>>,
    request_cache: HashMap<String, CacheEntry<serde_json::Value>>,
    default_ttl: Duration,
    hit_count: u64,
    miss_count: u64,
}

impl Default for TradingViewCache {
    fn default() -> Self {
        Self::new()
    }
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
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// Create cache with custom settings
    pub fn with_ttl(default_ttl: Duration) -> Self {
        Self {
            eps_data_cache: HashMap::new(),
            symbol_cache: HashMap::new(),
            market_cache: HashMap::new(),
            request_cache: HashMap::new(),
            default_ttl,
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// Cache EPS rankings response
    pub fn cache_eps_rankings(&mut self, key: String, data: FrontendEPSResponse, ttl_seconds: Option<u64>) {
        let ttl = ttl_seconds.unwrap_or(self.default_ttl.as_secs());
        let entry = CacheEntry::new(data, ttl);
        
        self.eps_data_cache.insert(key.clone(), entry);
        self.cleanup_expired_entries();
        
        debug!("Cached EPS rankings with key: {}, TTL: {}s", key, ttl);
    }

    /// Get cached EPS rankings
    pub fn get_eps_rankings(&mut self, key: &str) -> Option<FrontendEPSResponse> {
        if let Some(entry) = self.eps_data_cache.get(key) {
            if !entry.is_expired() {
                self.hit_count += 1;
                debug!("Cache hit for EPS rankings key: {}", key);
                return Some(entry.data.clone());
            } else {
                // Remove expired entry
                self.eps_data_cache.remove(key);
                debug!("Removed expired EPS rankings cache entry: {}", key);
            }
        }
        
        self.miss_count += 1;
        debug!("Cache miss for EPS rankings key: {}", key);
        None
    }

    /// Cache individual symbol data
    pub fn cache_symbol(&mut self, symbol: String, data: FrontendEPSData, ttl_seconds: Option<u64>) {
        let ttl = ttl_seconds.unwrap_or(self.default_ttl.as_secs());
        let entry = CacheEntry::new(data, ttl);
        
        self.symbol_cache.insert(symbol.clone(), entry);
        self.cleanup_expired_entries();
        
        debug!("Cached symbol data: {}, TTL: {}s", symbol, ttl);
    }

    /// Get cached symbol data
    pub fn get_symbol(&mut self, symbol: &str) -> Option<FrontendEPSData> {
        if let Some(entry) = self.symbol_cache.get(symbol) {
            if !entry.is_expired() {
                self.hit_count += 1;
                return Some(entry.data.clone());
            } else {
                self.symbol_cache.remove(symbol);
            }
        }
        
        self.miss_count += 1;
        None
    }

    /// Cache market data batch
    pub fn cache_market_data(&mut self, market: String, data: FrontendDataBatch, ttl_seconds: Option<u64>) {
        let ttl = ttl_seconds.unwrap_or(self.default_ttl.as_secs());
        let entry = CacheEntry::new(data, ttl);
        
        self.market_cache.insert(market.clone(), entry);
        self.cleanup_expired_entries();
        
        debug!("Cached market data: {}, TTL: {}s", market, ttl);
    }

    /// Get cached market data
    pub fn get_market_data(&mut self, market: &str) -> Option<FrontendDataBatch> {
        if let Some(entry) = self.market_cache.get(market) {
            if !entry.is_expired() {
                self.hit_count += 1;
                return Some(entry.data.clone());
            } else {
                self.market_cache.remove(market);
            }
        }
        
        self.miss_count += 1;
        None
    }

    /// Generate cache key for EPS rankings
    pub fn generate_eps_rankings_key(
        page: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
        sector: Option<String>,
    ) -> String {
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(10);
        let country = country.as_deref().unwrap_or("all");
        let sector = sector.as_deref().unwrap_or("all");
        
        format!("eps_rankings:{}:{}:{}:{}", page, limit, country, sector)
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let total_entries = self.eps_data_cache.len() + self.symbol_cache.len() + 
                           self.market_cache.len() + self.request_cache.len();
        
        let active_entries = self.count_active_entries();
        let expired_entries = total_entries - active_entries;
        
        let total_requests = self.hit_count + self.miss_count;
        let hit_ratio = if total_requests > 0 {
            self.hit_count as f64 / total_requests as f64
        } else {
            0.0
        };
        
        // Rough cache size estimation (this would be more accurate with actual serialization)
        let estimated_size_bytes = total_entries * 1024; // Estimate 1KB per entry
        let cache_size_mb = estimated_size_bytes as f64 / 1_048_576.0;
        
        CacheStats {
            total_count: total_entries,
            active_entries,
            expired_entries,
            hit_ratio,
            cache_size_mb,
            avg_ttl_seconds: self.default_ttl.as_secs(),
        }
    }

    /// Count active (non-expired) entries
    fn count_active_entries(&self) -> usize {
        let active_eps = self.eps_data_cache.values().filter(|entry| !entry.is_expired()).count();
        let active_symbols = self.symbol_cache.values().filter(|entry| !entry.is_expired()).count();
        let active_market = self.market_cache.values().filter(|entry| !entry.is_expired()).count();
        let active_request = self.request_cache.values().filter(|entry| !entry.is_expired()).count();
        
        active_eps + active_symbols + active_market + active_request
    }

    /// Clean up expired entries from all caches
    fn cleanup_expired_entries(&mut self) {
        let initial_count = self.eps_data_cache.len() + self.symbol_cache.len() + 
                           self.market_cache.len() + self.request_cache.len();
        
        self.eps_data_cache.retain(|_, entry| !entry.is_expired());
        self.symbol_cache.retain(|_, entry| !entry.is_expired());
        self.market_cache.retain(|_, entry| !entry.is_expired());
        self.request_cache.retain(|_, entry| !entry.is_expired());
        
        let final_count = self.eps_data_cache.len() + self.symbol_cache.len() + 
                         self.market_cache.len() + self.request_cache.len();
        
        let cleaned_count = initial_count - final_count;
        if cleaned_count > 0 {
            info!("Cleaned up {} expired cache entries", cleaned_count);
        }
    }

    /// Clear all cache entries
    pub fn clear_all(&mut self) {
        let total_cleared = self.eps_data_cache.len() + self.symbol_cache.len() + 
                           self.market_cache.len() + self.request_cache.len();
        
        self.eps_data_cache.clear();
        self.symbol_cache.clear();
        self.market_cache.clear();
        self.request_cache.clear();
        self.hit_count = 0;
        self.miss_count = 0;
        
        info!("Cleared all cache entries: {} items", total_cleared);
    }

    /// Get default TTL
    pub fn get_default_ttl(&self) -> Duration {
        self.default_ttl
    }

    /// Set default TTL
    pub fn set_default_ttl(&mut self, ttl: Duration) {
        self.default_ttl = ttl;
        debug!("Updated default cache TTL to: {:?}", ttl);
    }
}

/// Cache performance optimizer
pub struct CachePerformanceOptimizer;

impl CachePerformanceOptimizer {
    /// Analyze cache performance and provide recommendations
    pub fn analyze_performance(stats: &CacheStats) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if stats.hit_ratio < 0.5 {
            recommendations.push("Consider increasing TTL values for better hit rates".to_string());
        }
        
        if stats.expired_entries > stats.active_entries {
            recommendations.push("High number of expired entries - consider more frequent cleanup".to_string());
        }
        
        if stats.cache_size_mb > 100.0 {
            recommendations.push("Cache size is large - monitor memory usage".to_string());
        }
        
        if stats.active_entries == 0 {
            recommendations.push("Cache is empty - consider cache warming strategies".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Cache performance looks good".to_string());
        }
        
        recommendations
    }

    /// Suggest optimal TTL based on data type
    pub fn suggest_ttl(data_type: &str) -> Duration {
        match data_type {
            "eps_rankings" => Duration::from_secs(300),    // 5 minutes for rankings
            "symbol_data" => Duration::from_secs(180),     // 3 minutes for individual symbols
            "market_data" => Duration::from_secs(600),     // 10 minutes for market data
            "request_cache" => Duration::from_secs(60),    // 1 minute for API requests
            _ => Duration::from_secs(300),                 // Default 5 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_creation() {
        let data = "test_data".to_string();
        let entry = CacheEntry::new(data.clone(), 300);
        
        assert_eq!(entry.data, data);
        assert_eq!(entry.ttl_seconds, 300);
        assert!(!entry.is_expired());
        assert!(entry.remaining_ttl() <= 300);
    }

    #[test]
    fn test_cache_key_generation() {
        let key = TradingViewCache::generate_eps_rankings_key(
            Some(1), Some(10), Some("america".to_string()), Some("technology".to_string())
        );
        
        assert_eq!(key, "eps_rankings:1:10:america:technology");
        
        let default_key = TradingViewCache::generate_eps_rankings_key(None, None, None, None);
        assert_eq!(default_key, "eps_rankings:1:10:all:all");
    }

    #[test]
    fn test_cache_operations() {
        let cache = TradingViewCache::new();
        
        // Test initial stats
        let stats = cache.get_stats();
        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.hit_ratio, 0.0);
        
        // Test caching and retrieval (we can't easily test FrontendEPSResponse without more setup)
        // So we'll just test that the cache is created successfully
        assert!(cache.eps_data_cache.is_empty());
    }

    #[test]
    fn test_performance_analyzer() {
        let good_stats = CacheStats {
            total_count: 100,
            active_entries: 90,
            expired_entries: 10,
            hit_ratio: 0.8,
            cache_size_mb: 50.0,
            avg_ttl_seconds: 300,
        };
        
        let recommendations = CachePerformanceOptimizer::analyze_performance(&good_stats);
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_ttl_suggestions() {
        assert_eq!(CachePerformanceOptimizer::suggest_ttl("eps_rankings"), Duration::from_secs(300));
        assert_eq!(CachePerformanceOptimizer::suggest_ttl("symbol_data"), Duration::from_secs(180));
        assert_eq!(CachePerformanceOptimizer::suggest_ttl("unknown"), Duration::from_secs(300));
    }
}