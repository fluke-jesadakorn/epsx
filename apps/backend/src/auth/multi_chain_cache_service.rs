use anyhow::{ anyhow, Result };
use chrono::{ DateTime, Duration, Utc };
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{ debug, info };

/// Network-specific caching configuration
#[derive(Debug, Clone)]
pub struct NetworkCacheConfig {
  /// Time-to-live for cached results
  pub ttl_minutes: i64,
  /// Maximum entries in cache for this network
  pub max_entries: usize,
  /// Whether to use aggressive caching (cache failures too)
  pub cache_failures: bool,
  /// Network cost multiplier for cache priority
  pub cost_multiplier: f64,
}

impl NetworkCacheConfig {
  /// Ethereum mainnet: expensive, long cache
  pub fn ethereum() -> Self {
    Self {
      ttl_minutes: 60, // 1 hour - expensive calls
      max_entries: 10000,
      cache_failures: true, // Cache failures to avoid expensive retries
      cost_multiplier: 1.0, // Base cost
    }
  }

  /// Polygon: cheaper, moderate cache
  pub fn polygon() -> Self {
    Self {
      ttl_minutes: 30, // 30 minutes
      max_entries: 15000,
      cache_failures: false, // Don't cache failures - cheaper to retry
      cost_multiplier: 0.1, // Much cheaper than Ethereum
    }
  }

  /// Arbitrum: fast, short cache
  pub fn arbitrum() -> Self {
    Self {
      ttl_minutes: 15, // 15 minutes - fast finality
      max_entries: 12000,
      cache_failures: false,
      cost_multiplier: 0.05, // Very cheap
    }
  }

  /// Optimism: fast, short cache
  pub fn optimism() -> Self {
    Self {
      ttl_minutes: 15, // 15 minutes
      max_entries: 12000,
      cache_failures: false,
      cost_multiplier: 0.05,
    }
  }

  /// Base: fast, short cache
  pub fn base() -> Self {
    Self {
      ttl_minutes: 15, // 15 minutes
      max_entries: 12000,
      cache_failures: false,
      cost_multiplier: 0.05,
    }
  }

  /// BSC: moderate cost, moderate cache
  pub fn bsc() -> Self {
    Self {
      ttl_minutes: 20, // 20 minutes
      max_entries: 8000,
      cache_failures: false,
      cost_multiplier: 0.02, // Very cheap
    }
  }
}

/// Cached verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedVerification {
  /// Wallet address
  pub wallet_address: String,
  /// Contract address being verified
  pub contract_address: String,
  /// Network name
  pub network: String,
  /// Type of verification (nft_balance, token_balance, delegation)
  pub verification_type: String,
  /// The verification result
  pub result: bool,
  /// Detailed verification data
  pub verification_data: serde_json::Value,
  /// When this was cached
  pub cached_at: DateTime<Utc>,
  /// When this expires
  pub expires_at: DateTime<Utc>,
  /// Number of times this cache entry was accessed
  pub access_count: u64,
  /// Last time this was accessed
  pub last_accessed: DateTime<Utc>,
}

impl CachedVerification {
  /// Check if this cache entry is still valid
  pub fn is_valid(&self) -> bool {
    Utc::now() < self.expires_at
  }

  /// Mark as accessed (for LRU tracking)
  pub fn mark_accessed(&mut self) {
    self.access_count += 1;
    self.last_accessed = Utc::now();
  }

  /// Get cache key for this verification
  pub fn cache_key(&self) -> String {
    format!(
      "{}:{}:{}:{}",
      self.network,
      self.verification_type,
      self.wallet_address,
      self.contract_address
    )
  }
}

/// Multi-chain caching service with network-specific optimizations
pub struct MultiChainCacheService {
  /// Per-network cache stores
  caches: HashMap<String, Arc<RwLock<HashMap<String, CachedVerification>>>>,
  /// Per-network configurations
  configs: HashMap<String, NetworkCacheConfig>,
  /// Global cache statistics
  stats: Arc<RwLock<CacheStatistics>>,
}

#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
  pub total_hits: u64,
  pub total_misses: u64,
  pub total_entries: u64,
  pub network_stats: HashMap<String, NetworkStats>,
}

#[derive(Debug, Default, Clone)]
pub struct NetworkStats {
  pub hits: u64,
  pub misses: u64,
  pub entries: u64,
  pub evictions: u64,
  pub avg_ttl_minutes: f64,
}

impl MultiChainCacheService {
  /// Create new multi-chain cache service
  pub fn new() -> Self {
    let mut service = Self {
      caches: HashMap::new(),
      configs: HashMap::new(),
      stats: Arc::new(RwLock::new(CacheStatistics::default())),
    };

    // Initialize network-specific caches
    service.init_network_cache("ethereum", NetworkCacheConfig::ethereum());
    service.init_network_cache("polygon", NetworkCacheConfig::polygon());
    service.init_network_cache("arbitrum", NetworkCacheConfig::arbitrum());
    service.init_network_cache("optimism", NetworkCacheConfig::optimism());
    service.init_network_cache("base", NetworkCacheConfig::base());
    service.init_network_cache("bsc", NetworkCacheConfig::bsc());

    service
  }

  /// Initialize cache for a specific network
  fn init_network_cache(&mut self, network: &str, config: NetworkCacheConfig) {
    self.caches.insert(
      network.to_string(),
      Arc::new(RwLock::new(HashMap::new()))
    );
    self.configs.insert(network.to_string(), config);

    info!(
      "Initialized cache for network: {} with TTL: {} minutes",
      network,
      self.configs[network].ttl_minutes
    );
  }

  /// Get cached verification result
  pub async fn get_verification(
    &self,
    wallet_address: &str,
    contract_address: &str,
    network: &str,
    verification_type: &str
  ) -> Option<bool> {
    let cache_key = format!(
      "{}:{}:{}:{}",
      network,
      verification_type,
      wallet_address,
      contract_address
    );

    if let Some(cache) = self.caches.get(network) {
      let mut cache_guard = cache.write().await;

      if let Some(entry) = cache_guard.get_mut(&cache_key) {
        if entry.is_valid() {
          entry.mark_accessed();
          self.record_cache_hit(network).await;
          debug!("Cache HIT for {}: {}", cache_key, entry.result);
          return Some(entry.result);
        } else {
          // Remove expired entry
          cache_guard.remove(&cache_key);
          debug!("Cache entry expired for {}", cache_key);
        }
      }
    }

    self.record_cache_miss(network).await;
    debug!("Cache MISS for {}", cache_key);
    None
  }

  /// Cache verification result
  pub async fn cache_verification(
    &self,
    wallet_address: &str,
    contract_address: &str,
    network: &str,
    verification_type: &str,
    result: bool,
    verification_data: serde_json::Value
  ) -> Result<()> {
    let config = self.configs
      .get(network)
      .ok_or_else(|| anyhow!("No cache config for network: {}", network))?;

    // Don't cache failures for networks that are cheap to retry
    if !result && !config.cache_failures {
      debug!("Skipping cache for failure on cheap network: {}", network);
      return Ok(());
    }

    let cache_key = format!(
      "{}:{}:{}:{}",
      network,
      verification_type,
      wallet_address,
      contract_address
    );
    let now = Utc::now();

    let cached_verification = CachedVerification {
      wallet_address: wallet_address.to_string(),
      contract_address: contract_address.to_string(),
      network: network.to_string(),
      verification_type: verification_type.to_string(),
      result,
      verification_data,
      cached_at: now,
      expires_at: now + Duration::minutes(config.ttl_minutes),
      access_count: 0,
      last_accessed: now,
    };

    if let Some(cache) = self.caches.get(network) {
      let mut cache_guard = cache.write().await;

      // Check if we need to evict entries
      if cache_guard.len() >= config.max_entries {
        self.evict_lru_entries(&mut cache_guard, config.max_entries / 4).await;
      }

      cache_guard.insert(cache_key.clone(), cached_verification);
      self.record_cache_entry(network).await;

      debug!(
        "Cached verification for {} (expires in {} minutes)",
        cache_key,
        config.ttl_minutes
      );
    }

    Ok(())
  }

  /// Evict least recently used entries
  async fn evict_lru_entries(
    &self,
    cache: &mut HashMap<String, CachedVerification>,
    evict_count: usize
  ) {
    // Clone all data to avoid borrowing conflicts
    let mut entries: Vec<(String, CachedVerification)> = cache
      .iter()
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect();
    entries.sort_by_key(|(_, entry)| entry.last_accessed);

    // Remove oldest entries
    for (key, _) in entries.iter().take(evict_count) {
      cache.remove(key);
    }

    debug!("Evicted {} cache entries", evict_count);
  }

  /// Get cache statistics for all networks
  pub async fn get_cache_stats(&self) -> CacheStatistics {
    let stats = self.stats.read().await;
    (*stats).clone()
  }

  /// Get cache statistics for specific network
  pub async fn get_network_stats(&self, network: &str) -> Option<NetworkStats> {
    let stats = self.stats.read().await;
    stats.network_stats.get(network).cloned()
  }

  /// Clear cache for specific network
  pub async fn clear_network_cache(&self, network: &str) -> Result<()> {
    if let Some(cache) = self.caches.get(network) {
      let mut cache_guard = cache.write().await;
      let entry_count = cache_guard.len();
      cache_guard.clear();
      info!("Cleared {} entries from {} cache", entry_count, network);
    }
    Ok(())
  }

  /// Clear all caches
  pub async fn clear_all_caches(&self) -> Result<()> {
    for (network, cache) in &self.caches {
      let mut cache_guard = cache.write().await;
      let entry_count = cache_guard.len();
      cache_guard.clear();
      info!("Cleared {} entries from {} cache", entry_count, network);
    }

    // Reset statistics
    let mut stats = self.stats.write().await;
    *stats = CacheStatistics::default();

    Ok(())
  }

  /// Get cache health status
  pub async fn get_cache_health(&self) -> HashMap<String, CacheHealthStatus> {
    let mut health = HashMap::new();

    for (network, cache) in &self.caches {
      let cache_guard = cache.read().await;
      let config = &self.configs[network];

      let entry_count = cache_guard.len();
      let capacity_usage =
        ((entry_count as f64) / (config.max_entries as f64)) * 100.0;

      // Count expired entries
      let expired_count = cache_guard
        .values()
        .filter(|entry| !entry.is_valid())
        .count();

      let health_status = CacheHealthStatus {
        network: network.clone(),
        entry_count,
        max_entries: config.max_entries,
        capacity_usage_percent: capacity_usage,
        expired_entries: expired_count,
        avg_ttl_minutes: config.ttl_minutes,
        status: if capacity_usage > 90.0 {
          "CRITICAL".to_string()
        } else if capacity_usage > 75.0 {
          "WARNING".to_string()
        } else {
          "HEALTHY".to_string()
        },
      };

      health.insert(network.clone(), health_status);
    }

    health
  }

  /// Background cleanup task to remove expired entries
  pub async fn cleanup_expired_entries(&self) {
    for (network, cache) in &self.caches {
      let mut cache_guard = cache.write().await;
      let initial_count = cache_guard.len();

      cache_guard.retain(|_, entry| entry.is_valid());

      let removed_count = initial_count - cache_guard.len();
      if removed_count > 0 {
        debug!(
          "Removed {} expired entries from {} cache",
          removed_count,
          network
        );
      }
    }
  }

  /// Record cache hit for statistics
  async fn record_cache_hit(&self, network: &str) {
    let mut stats = self.stats.write().await;
    stats.total_hits += 1;

    let network_stats = stats.network_stats
      .entry(network.to_string())
      .or_default();
    network_stats.hits += 1;
  }

  /// Record cache miss for statistics
  async fn record_cache_miss(&self, network: &str) {
    let mut stats = self.stats.write().await;
    stats.total_misses += 1;

    let network_stats = stats.network_stats
      .entry(network.to_string())
      .or_default();
    network_stats.misses += 1;
  }

  /// Record new cache entry for statistics
  async fn record_cache_entry(&self, network: &str) {
    let mut stats = self.stats.write().await;
    stats.total_entries += 1;

    let network_stats = stats.network_stats
      .entry(network.to_string())
      .or_default();
    network_stats.entries += 1;
  }
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheHealthStatus {
  pub network: String,
  pub entry_count: usize,
  pub max_entries: usize,
  pub capacity_usage_percent: f64,
  pub expired_entries: usize,
  pub avg_ttl_minutes: i64,
  pub status: String,
}

/// Cache warming strategies for different networks
pub struct CacheWarmingService {
  cache_service: Arc<MultiChainCacheService>,
}

impl CacheWarmingService {
  pub fn new(cache_service: Arc<MultiChainCacheService>) -> Self {
    Self { cache_service }
  }

  /// Warm cache for high-priority wallets and contracts
  pub async fn warm_priority_contracts(&self) -> Result<()> {
    let priority_contracts = vec![
      // High-value NFT collections
      ("ethereum", "nft_balance", "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d"), // BAYC
      ("ethereum", "nft_balance", "0x60e4d786628fea6478f785a6d7e704777c86a7c6"), // MAYC
      ("ethereum", "nft_balance", "0xb47e3cd837ddf8e4c57f05d70ab865de6e193bbb"), // CryptoPunks

      // Major ERC-20 tokens
      (
        "ethereum",
        "token_balance",
        "0xa0b86a33e6e4f1e8c2b1e1a4b8e8c8e8c8e8c8e8",
      ), // ETH
      (
        "ethereum",
        "token_balance",
        "0xa0b73e1ff0b80914ab6fe0444e65848c4c34450b",
      ), // USDC
      (
        "polygon",
        "token_balance",
        "0x2791bca1f2de4661ed88a30c5b9c3c7ad0e4be39",
      ), // USDC on Polygon

      // BSC tokens
      ("bsc", "token_balance", "0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82"), // CAKE
      ("bsc", "token_balance", "0x55d398326f99059ff775485246999027b3197955") // USDT
    ];

    info!(
      "Starting cache warming for {} priority contracts",
      priority_contracts.len()
    );

    for (network, verification_type, contract) in priority_contracts {
      // In a real implementation, you'd query for active wallets from the database
      // and warm the cache for those specific wallet-contract combinations
      debug!(
        "Would warm cache for {} {} on {}",
        verification_type,
        contract,
        network
      );
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn cache_stores_and_retrieves_verification() {
    let service = MultiChainCacheService::new();

    let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
    let contract = "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d";
    let network = "ethereum";
    let verification_type = "nft_balance";

    // Should be cache miss initially
    assert!(
      service
        .get_verification(wallet, contract, network, verification_type).await
        .is_none()
    );

    // Cache a verification
    service
      .cache_verification(
        wallet,
        contract,
        network,
        verification_type,
        true,
        serde_json::json!({"balance": 1})
      ).await
      .unwrap();

    // Should be cache hit now
    assert_eq!(
      service.get_verification(
        wallet,
        contract,
        network,
        verification_type
      ).await,
      Some(true)
    );
  }

  #[tokio::test]
  async fn different_networks_have_different_ttls() {
    let service = MultiChainCacheService::new();

    assert_eq!(service.configs["ethereum"].ttl_minutes, 60);
    assert_eq!(service.configs["polygon"].ttl_minutes, 30);
    assert_eq!(service.configs["arbitrum"].ttl_minutes, 15);
    assert_eq!(service.configs["bsc"].ttl_minutes, 20);
  }

  #[tokio::test]
  async fn cache_health_reports_correct_status() {
    let service = MultiChainCacheService::new();

    let health = service.get_cache_health().await;

    // All networks should be healthy initially
    for (network, status) in health {
      assert_eq!(status.status, "HEALTHY");
      assert_eq!(status.entry_count, 0);
      assert!(status.capacity_usage_percent < 1.0);
    }
  }

  #[tokio::test]
  async fn cache_statistics_track_hits_and_misses() {
    let service = MultiChainCacheService::new();

    let wallet = "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6";
    let contract = "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d";

    // This should be a miss
    service.get_verification(wallet, contract, "ethereum", "nft_balance").await;

    let stats = service.get_cache_stats().await;
    assert_eq!(stats.total_misses, 1);
    assert_eq!(stats.total_hits, 0);

    // Cache and retrieve
    service
      .cache_verification(
        wallet,
        contract,
        "ethereum",
        "nft_balance",
        true,
        serde_json::json!({})
      ).await
      .unwrap();

    service.get_verification(wallet, contract, "ethereum", "nft_balance").await;

    let stats = service.get_cache_stats().await;
    assert_eq!(stats.total_misses, 1);
    assert_eq!(stats.total_hits, 1);
  }
}
