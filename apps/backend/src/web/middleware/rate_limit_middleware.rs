use axum::{
  extract::{ Request, State },
  http::{ HeaderMap, StatusCode },
  middleware::Next,
  response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{ DateTime, Utc, Duration };
use tracing::{ debug, warn };
use crate::infrastructure::container::DomainContainer;
use crate::infrastructure::adapters::services::web3_permission_service_adapter::Web3PermissionServiceAdapter;

/// Rate limit tier based on token holdings
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitTier {
  Free, // No tokens: 10 requests/minute
  Basic, // Some tokens: 60 requests/minute
  Premium, // High-value tokens: 300 requests/minute
  Elite, // Whale-tier holdings: 1000 requests/minute
}

impl RateLimitTier {
  /// Get requests per minute for this tier
  pub fn requests_per_minute(&self) -> u32 {
    match self {
      RateLimitTier::Free => 10,
      RateLimitTier::Basic => 60,
      RateLimitTier::Premium => 300,
      RateLimitTier::Elite => 1000,
    }
  }

  /// Get burst capacity (max requests in a single second)
  pub fn burst_capacity(&self) -> u32 {
    match self {
      RateLimitTier::Free => 2,
      RateLimitTier::Basic => 10,
      RateLimitTier::Premium => 50,
      RateLimitTier::Elite => 100,
    }
  }
}

/// Token-based rate limiting configuration
#[derive(Debug, Clone)]
pub struct TokenRateLimitConfig {
  /// Ethereum-based tokens (value in USD)
  pub eth_threshold_basic: f64, // $100 worth
  pub eth_threshold_premium: f64, // $1000 worth
  pub eth_threshold_elite: f64, // $10000 worth

  /// Polygon-based tokens
  pub matic_threshold_basic: f64, // $50 worth
  pub matic_threshold_premium: f64, // $500 worth
  pub matic_threshold_elite: f64, // $5000 worth

  /// BSC-based tokens
  pub bnb_threshold_basic: f64, // $50 worth
  pub bnb_threshold_premium: f64, // $500 worth
  pub bnb_threshold_elite: f64, // $5000 worth

  /// Cache duration for token holdings (minutes)
  pub cache_duration_minutes: i64,
}

impl Default for TokenRateLimitConfig {
  fn default() -> Self {
    Self {
      eth_threshold_basic: 100.0,
      eth_threshold_premium: 1000.0,
      eth_threshold_elite: 10000.0,
      matic_threshold_basic: 50.0,
      matic_threshold_premium: 500.0,
      matic_threshold_elite: 5000.0,
      bnb_threshold_basic: 50.0,
      bnb_threshold_premium: 500.0,
      bnb_threshold_elite: 5000.0,
      cache_duration_minutes: 60, // Cache for 1 hour
    }
  }
}

/// Rate limit bucket for token bucket algorithm
#[derive(Debug, Clone)]
pub struct RateLimitBucket {
  tokens: u32,
  last_refill: DateTime<Utc>,
  tier: RateLimitTier,
}

impl RateLimitBucket {
  pub fn new(tier: RateLimitTier) -> Self {
    Self {
      tokens: tier.burst_capacity(),
      last_refill: Utc::now(),
      tier,
    }
  }

  /// Try to consume tokens, returns true if allowed
  pub fn try_consume(&mut self, tokens_needed: u32) -> bool {
    self.refill();

    if self.tokens >= tokens_needed {
      self.tokens -= tokens_needed;
      true
    } else {
      false
    }
  }

  /// Refill tokens based on time elapsed
  fn refill(&mut self) {
    let now = Utc::now();
    let elapsed = now.signed_duration_since(self.last_refill);
    let elapsed_minutes = elapsed.num_minutes() as f64;

    if elapsed_minutes > 0.0 {
      let tokens_to_add = (((self.tier.requests_per_minute() as f64) *
        elapsed_minutes) /
        60.0) as u32;
      self.tokens = (self.tokens + tokens_to_add).min(
        self.tier.burst_capacity()
      );
      self.last_refill = now;
    }
  }

  /// Update the tier if token holdings changed
  pub fn update_tier(&mut self, new_tier: RateLimitTier) {
    if self.tier != new_tier {
      debug!("Rate limit tier updated: {:?} -> {:?}", self.tier, new_tier);
      // Clone new_tier before moving it
      let tier_capacity = new_tier.burst_capacity();
      self.tier = new_tier;
      // Adjust tokens to new capacity
      self.tokens = self.tokens.min(tier_capacity);
    }
  }
}

/// Cross-chain token holdings cache
#[derive(Debug, Clone)]
pub struct TokenHoldingsCache {
  #[allow(dead_code)]
  wallet_address: String,
  #[allow(dead_code)]
  total_value_usd: f64,
  tier: RateLimitTier,
  #[allow(dead_code)]
  cached_at: DateTime<Utc>,
  expires_at: DateTime<Utc>,
}

/// Web3 rate limiting service
pub struct Web3RateLimitService {
  buckets: Arc<RwLock<HashMap<String, RateLimitBucket>>>,
  holdings_cache: Arc<RwLock<HashMap<String, TokenHoldingsCache>>>,
  config: TokenRateLimitConfig,
  #[allow(dead_code)]
  web3_service: Arc<Web3PermissionServiceAdapter>,
}

impl Web3RateLimitService {
  pub fn new(web3_service: Arc<Web3PermissionServiceAdapter>) -> Self {
    Self {
      buckets: Arc::new(RwLock::new(HashMap::new())),
      holdings_cache: Arc::new(RwLock::new(HashMap::new())),
      config: TokenRateLimitConfig::default(),
      web3_service,
    }
  }

  /// Check if request is allowed for wallet
  pub async fn is_allowed(
    &self,
    wallet_address: &str
  ) -> Result<bool, Box<dyn std::error::Error>> {
    // Get current tier for this wallet
    let tier = self.get_wallet_tier(wallet_address).await?;

    // Get or create bucket for this wallet
    let mut buckets = self.buckets.write().await;
    let bucket = buckets
      .entry(wallet_address.to_string())
      .or_insert_with(|| RateLimitBucket::new(tier.clone()));

    // Update tier if needed
    bucket.update_tier(tier);

    // Try to consume one token
    Ok(bucket.try_consume(1))
  }

  /// Get rate limit tier based on cross-chain token holdings
  async fn get_wallet_tier(
    &self,
    wallet_address: &str
  ) -> Result<RateLimitTier, Box<dyn std::error::Error>> {
    // Check cache first
    {
      let cache = self.holdings_cache.read().await;
      if let Some(cached) = cache.get(wallet_address) {
        if Utc::now() < cached.expires_at {
          debug!(
            "Using cached tier {:?} for wallet {}",
            cached.tier,
            wallet_address
          );
          return Ok(cached.tier.clone());
        }
      }
    }

    // Calculate tier based on cross-chain holdings
    let total_value = self.calculate_cross_chain_value(wallet_address).await?;
    let tier = self.determine_tier(total_value);

    // Cache the result
    {
      let mut cache = self.holdings_cache.write().await;
      cache.insert(wallet_address.to_string(), TokenHoldingsCache {
        wallet_address: wallet_address.to_string(),
        total_value_usd: total_value,
        tier: tier.clone(),
        cached_at: Utc::now(),
        expires_at: Utc::now() +
        Duration::minutes(self.config.cache_duration_minutes),
      });
    }

    debug!(
      "Calculated tier {:?} for wallet {} (${:.2} total value)",
      tier,
      wallet_address,
      total_value
    );
    Ok(tier)
  }

  /// Calculate total USD value across all supported chains
  async fn calculate_cross_chain_value(
    &self,
    wallet_address: &str
  ) -> Result<f64, Box<dyn std::error::Error>> {
    let mut total_value = 0.0;

    // Check major tokens on each network
    let networks = vec![
      (
        "ethereum",
        vec![
          ("0xa0b86a33e6e4f1e8c2b1e1a4b8e8c8e8c8e8c8e8", "ETH", 2000.0), // Example ETH price
          ("0xa0b73e1ff0b80914ab6fe0444e65848c4c34450b", "USDC", 1.0) // USDC
        ],
      ),
      (
        "polygon",
        vec![
          ("0x0000000000000000000000000000000000001010", "MATIC", 0.8), // MATIC price
          ("0x2791bca1f2de4661ed88a30c5b9c3c7ad0e4be39", "USDC", 1.0) // USDC on Polygon
        ],
      ),
      (
        "bsc",
        vec![
          ("0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82", "CAKE", 2.5), // CAKE price
          ("0x55d398326f99059ff775485246999027b3197955", "USDT", 1.0), // USDT on BSC
          ("0xe9e7cea3dedca5984780bafc599bd69add087d56", "BUSD", 1.0) // BUSD
        ],
      )
    ];

    for (network, tokens) in networks {
      for (contract_address, _symbol, price_usd) in tokens {
        match
          self.get_token_balance_value(
            wallet_address,
            contract_address,
            network,
            price_usd
          ).await
        {
          Ok(value) => {
            total_value += value;
            debug!(
              "Found ${:.2} worth of tokens on {} for {}",
              value,
              network,
              wallet_address
            );
          }
          Err(e) => {
            warn!("Failed to get token balance on {}: {}", network, e);
          }
        }
      }
    }

    // Add NFT holdings (simplified estimation)
    if
      let Ok(nft_value) =
        self.estimate_nft_portfolio_value(wallet_address).await
    {
      total_value += nft_value;
    }

    Ok(total_value)
  }

  /// Get token balance value in USD
  async fn get_token_balance_value(
    &self,
    wallet_address: &str,
    _contract_address: &str,
    _network: &str,
    price_usd: f64
  ) -> Result<f64, Box<dyn std::error::Error>> {
    // This is a simplified implementation
    // In production, you'd use the Web3PermissionService to get actual balances
    // and integrate with price feeds like Chainlink or CoinGecko

    // For demo purposes, return a simulated value
    let simulated_balance = if wallet_address.to_lowercase().contains("742d35") {
      1000.0 // Demo wallet has some tokens
    } else {
      0.0
    };

    Ok(simulated_balance * price_usd)
  }

  /// Estimate NFT portfolio value
  async fn estimate_nft_portfolio_value(
    &self,
    _wallet_address: &str
  ) -> Result<f64, Box<dyn std::error::Error>> {
    // Simplified NFT valuation
    // In production, integrate with OpenSea API, floor price feeds, etc.

    let _nft_collections = [("0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d", "BAYC", 30000.0), // Bored Ape floor
      ("0x60e4d786628fea6478f785a6d7e704777c86a7c6", "MAYC", 5000.0)];

    // For demo, return 0 for now
    Ok(0.0)
  }

  /// Determine rate limit tier based on total value
  fn determine_tier(&self, total_value_usd: f64) -> RateLimitTier {
    if total_value_usd >= self.config.eth_threshold_elite {
      RateLimitTier::Elite
    } else if total_value_usd >= self.config.eth_threshold_premium {
      RateLimitTier::Premium
    } else if total_value_usd >= self.config.eth_threshold_basic {
      RateLimitTier::Basic
    } else {
      RateLimitTier::Free
    }
  }

  /// Get remaining tokens for a wallet
  pub async fn get_remaining_tokens(&self, wallet_address: &str) -> u32 {
    let buckets = self.buckets.read().await;
    buckets
      .get(wallet_address)
      .map(|bucket| bucket.tokens)
      .unwrap_or(0)
  }

  /// Get tier info for a wallet
  pub async fn get_tier_info(
    &self,
    wallet_address: &str
  ) -> (RateLimitTier, u32, u32) {
    let tier = self
      .get_wallet_tier(wallet_address).await
      .unwrap_or(RateLimitTier::Free);
    let remaining = self.get_remaining_tokens(wallet_address).await;
    (tier.clone(), remaining, tier.requests_per_minute())
  }
}

/// Middleware function for Web3 rate limiting
pub async fn web3_rate_limit_middleware(
  State(_container): State<Arc<DomainContainer>>,
  headers: HeaderMap,
  request: Request,
  next: Next
) -> Result<Response, StatusCode> {
  // Extract wallet address from Authorization header or X-Wallet-Address
  let wallet_address = extract_wallet_address(&headers);

  if let Some(wallet_address) = wallet_address {
    // Get rate limit service (this would be added to DomainContainer)
    // For now, we'll create a simplified check

    let is_premium = wallet_address.to_lowercase().contains("742d35"); // Demo logic
    let allowed = if is_premium {
      true
    } else {
      // Simple rate limiting for non-premium users
      true // Always allow for demo
    };

    if !allowed {
      warn!("Rate limit exceeded for wallet: {}", wallet_address);
      return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    debug!("Rate limit check passed for wallet: {}", wallet_address);
  }

  // Continue with request
  Ok(next.run(request).await)
}

/// Extract wallet address from request headers
fn extract_wallet_address(headers: &HeaderMap) -> Option<String> {
  // Try X-Wallet-Address header first (standardized header name)
  if let Some(wallet) = headers.get("X-Wallet-Address") {
    if let Ok(wallet_str) = wallet.to_str() {
      return Some(wallet_str.to_string());
    }
  }

  // Pure Web3 authentication - no Bearer token support
  // Rate limiting based on wallet address only

  None
}

/// Unified rate limit middleware - alias for web3_rate_limit_middleware
/// Provides a unified entry point for rate limiting across the application
pub async fn unified_rate_limit_middleware(
  State(container): State<Arc<DomainContainer>>,
  headers: HeaderMap,
  request: Request,
  next: Next
) -> Result<Response, StatusCode> {
  // Delegate to web3_rate_limit_middleware for unified behavior
  web3_rate_limit_middleware(State(container), headers, request, next).await
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rate_limit_tiers_correct_limits() {
    assert_eq!(RateLimitTier::Free.requests_per_minute(), 10);
    assert_eq!(RateLimitTier::Basic.requests_per_minute(), 60);
    assert_eq!(RateLimitTier::Premium.requests_per_minute(), 300);
    assert_eq!(RateLimitTier::Elite.requests_per_minute(), 1000);
  }

  #[test]
  fn bucket_refill_works() {
    let mut bucket = RateLimitBucket::new(RateLimitTier::Basic);

    // Consume all tokens
    for _ in 0..bucket.tier.burst_capacity() {
      assert!(bucket.try_consume(1));
    }

    // Should be empty now
    assert!(!bucket.try_consume(1));

    // Manually advance time (in real test, would use mock time)
    bucket.last_refill = Utc::now() - Duration::seconds(60);
    bucket.refill();

    // Should have tokens again
    assert!(bucket.try_consume(1));
  }

  #[test]
  fn tier_determination_works() {
    let config = TokenRateLimitConfig::default();
    let service = Web3RateLimitService {
      buckets: Arc::new(RwLock::new(HashMap::new())),
      holdings_cache: Arc::new(RwLock::new(HashMap::new())),
      config: config.clone(),
      web3_service: Arc::new(
        // Would need to create a mock Web3PermissionService for testing
        // Skipping for now
      ),
    };

    assert_eq!(service.determine_tier(0.0), RateLimitTier::Free);
    assert_eq!(service.determine_tier(100.0), RateLimitTier::Basic);
    assert_eq!(service.determine_tier(1000.0), RateLimitTier::Premium);
    assert_eq!(service.determine_tier(10000.0), RateLimitTier::Elite);
  }
}
